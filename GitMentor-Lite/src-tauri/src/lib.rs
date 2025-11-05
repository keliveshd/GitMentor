mod commands;
// 应用后端入口：初始化组件并注册 Tauri 命令
// Author: Evilek, Date: 2025-08-11
// 这个憨批入口别乱加打印，日志走 write_startup_log，调试走 debug_log!

mod core;
mod types;
mod utils;

/// 调试日志宏
/// 只有在调试开关启用时才输出日志
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::commands::debug_commands::is_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// 警告日志宏（始终显示）
#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

/// 错误日志宏（始终显示）
#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

/// 信息日志宏（始终显示）
#[macro_export]
macro_rules! info_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

use chrono::Local;
use commands::{
    ai_analysis_commands, ai_commands, daily_report_commands, debug_commands, git_commands,
    git_config_commands, gitflow_commands, repository_commands, system_commands, template_commands,
    unified_template_commands, update_commands,
};
use core::{
    ai_manager::AIManager,
    git_config::GitConfigManager,
    git_engine::GitEngine,
    llm_client::{LLMClient, LLMConfig},
};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

fn resolve_log_file_path() -> PathBuf {
    if cfg!(debug_assertions) {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("startup.log")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("startup.log")
    }
}

/// 写入启动日志到文件
/// Author: Evilek, Date: 2025-01-09
fn write_startup_log(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_message = format!("[{}] {}\n", timestamp, message);

    let log_path = resolve_log_file_path();
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = file.write_all(log_message.as_bytes());
        let _ = file.flush();
    }

    // 同时输出到控制台
    println!("{}", log_message.trim());
}

/// 写入错误日志到文件
/// Author: Evilek, Date: 2025-01-09
fn write_error_log(error: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_message = format!("[{}] ERROR: {}\n", timestamp, error);

    let log_path = resolve_log_file_path();
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = file.write_all(log_message.as_bytes());
        let _ = file.flush();
    }

    // 同时输出到控制台
    eprintln!("{}", log_message.trim());
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 检查并处理延迟更新
async fn handle_pending_updates() {
    let current_exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(_) => return,
    };

    let app_dir = match current_exe.parent() {
        Some(dir) => dir.to_path_buf(),
        None => return,
    };

    let pending_dir = app_dir.join("pending-update");
    if !pending_dir.exists() {
        return;
    }

    let marker_file = pending_dir.join(".update-pending");
    if !marker_file.exists() {
        return;
    }

    println!("[STARTUP] 检测到待更新文件，开始应用更新...");

    // 查找 ZIP 文件
    let mut zip_files: Vec<_> = fs::read_dir(&pending_dir)
        .unwrap_or_else(|_| Vec::new())
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name()
                .to_string_lossy()
                .to_lowercase()
                .ends_with(".zip")
        })
        .collect();

    if zip_files.is_empty() {
        println!("[STARTUP] 未找到 ZIP 文件");
        let _ = fs::remove_dir_all(&pending_dir);
        return;
    }

    // 取最新的 ZIP 文件（假设文件名包含版本号，按字母顺序排序）
    zip_files.sort_by(|a, b| {
        a.file_name()
            .cmp(&b.file_name())
    });
    let zip_path = zip_files.last().unwrap().path();

    println!("[STARTUP] 应用延迟更新: {:?}", zip_path);

    let update_manager = gitmentor_lite_lib::core::update_manager::UpdateManager::new(
        env!("CARGO_PKG_VERSION").to_string()
    );

    match update_manager.install_portable_zip(&zip_path).await {
        Ok(_) => {
            println!("[STARTUP] 延迟更新成功");
            // 清理待更新目录
            let _ = fs::remove_dir_all(&pending_dir);
        }
        Err(e) => {
            println!("[STARTUP] 延迟更新失败: {}", e);
            // 不清理待更新目录，保留以便下次重试
        }
    }
}

/// 检查并处理更新器进程
fn handle_updater_mode() -> bool {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "updater" {
        println!("[UPDATER] 进入更新器模式");
        println!("[UPDATER] 参数: {:?}", args);

        // 解析命令行参数
        let mut installer_path = None;
        let mut app_dir = None;
        let mut exe_name = None;

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--installer" if i + 1 < args.len() => {
                    installer_path = Some(args[i + 1].clone());
                    i += 2;
                }
                "--app-dir" if i + 1 < args.len() => {
                    app_dir = Some(args[i + 1].clone());
                    i += 2;
                }
                "--exe-name" if i + 1 < args.len() => {
                    exe_name = Some(args[i + 1].clone());
                    i += 2;
                }
                _ => i += 1,
            }
        }

        // 验证参数
        if installer_path.is_none() || app_dir.is_none() {
            eprintln!("[UPDATER] 错误：缺少必要参数");
            std::process::exit(1);
        }

        let installer_path = PathBuf::from(installer_path.unwrap());
        let app_dir = PathBuf::from(app_dir.unwrap());
        let exe_name = exe_name.unwrap_or_else(|| "GitMentorLite.exe".to_string());

        println!("[UPDATER] 安装包路径: {:?}", installer_path);
        println!("[UPDATER] 应用目录: {:?}", app_dir);
        println!("[UPDATER] 可执行文件名: {}", exe_name);

        // 执行更新
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let update_manager = gitmentor_lite_lib::core::update_manager::UpdateManager::new(
                env!("CARGO_PKG_VERSION").to_string()
            );

            if let Err(e) = update_manager.run_updater_process(&installer_path, &app_dir, &exe_name).await {
                eprintln!("[UPDATER] 更新失败: {}", e);
                std::process::exit(1);
            }
        });

        println!("[UPDATER] 更新完成，重新启动应用...");

        // 重新启动应用
        let app_exe = app_dir.join(&exe_name);
        std::process::Command::new(&app_exe)
            .spawn()
            .expect("重新启动应用失败");

        println!("[UPDATER] 应用已重启，更新器退出");
        std::process::exit(0);
    }

    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    write_startup_log("=== GitMentor-Lite 启动开始 ===");
    write_startup_log("Author: Evilek, Date: 2025-01-09");

    // 检查是否为更新器模式
    if handle_updater_mode() {
        return; // 更新器模式已处理并退出
    }

    // 检查并处理待更新文件（异步）
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        handle_pending_updates().await;
    });

    // 记录当前工作目录
    match std::env::current_dir() {
        Ok(dir) => write_startup_log(&format!("当前工作目录: {}", dir.display())),
        Err(e) => write_error_log(&format!("无法获取当前工作目录: {}", e)),
    }

    // 过滤PNG警告和其他不必要的日志
    //std::env::set_var("RUST_LOG", "warn,libpng=off,image=off");
    std::env::set_var("RUST_LOG", "error"); // 只显示错误日志
    write_startup_log("设置日志级别为 ERROR");

    // Initialize configuration directory
    write_startup_log("初始化配置目录...");
    let config_dir = match std::env::current_dir() {
        Ok(dir) => {
            let config_path = dir.join(".config");
            write_startup_log(&format!("配置目录路径: {}", config_path.display()));
            config_path
        }
        Err(e) => {
            write_error_log(&format!("获取当前目录失败: {}", e));
            panic!("无法获取当前目录");
        }
    };

    // Initialize Git configuration
    write_startup_log("初始化Git配置管理器...");
    let git_config_path = config_dir.join("git_config.json");
    write_startup_log(&format!("Git配置文件路径: {}", git_config_path.display()));

    let git_config_manager = match GitConfigManager::new(git_config_path) {
        Ok(manager) => {
            write_startup_log("Git配置管理器初始化成功");
            manager
        }
        Err(e) => {
            write_error_log(&format!("Git配置管理器初始化失败: {}", e));
            panic!("Failed to initialize Git Config Manager: {}", e);
        }
    };
    let git_config = git_config_manager.get_config().clone();

    // Initialize components
    write_startup_log("初始化核心组件...");
    let git_engine = Mutex::new(GitEngine::new_with_config(git_config));
    write_startup_log("Git引擎初始化完成");

    let git_config_manager = Mutex::new(git_config_manager);
    let llm_config = LLMConfig::default();
    let llm_client = LLMClient::new(llm_config);
    write_startup_log("LLM客户端初始化完成");

    // Initialize AI Manager
    write_startup_log("初始化AI管理器...");
    let ai_config_path = config_dir.join("ai_config.json");
    write_startup_log(&format!("AI配置文件路径: {}", ai_config_path.display()));

    let ai_manager = match AIManager::new(ai_config_path) {
        Ok(manager) => {
            write_startup_log("AI管理器初始化成功");
            Arc::new(RwLock::new(manager))
        }
        Err(e) => {
            write_error_log(&format!("AI管理器初始化失败: {}", e));
            panic!("Failed to initialize AI Manager: {}", e);
        }
    };

    write_startup_log("构建Tauri应用...");
    let app_result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .manage(git_engine)
        .manage(git_config_manager)
        .manage(llm_client)
        .manage(ai_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            git_commands::select_repository,
            git_commands::close_repository,
            git_commands::stop_repo_watcher,
            git_commands::get_git_status,
            git_commands::get_remote_configuration,
            git_commands::add_remote,
            git_commands::update_remote,
            git_commands::remove_remote,
            git_commands::set_branch_upstream,
            repository_commands::clone_repository,
            repository_commands::configure_remote,
            repository_commands::validate_remote_connection,
            repository_commands::generate_initial_commit_message,
            git_commands::generate_commit_message,
            git_commands::stage_files,
            git_commands::commit_changes,
            git_commands::revert_files,
            debug_commands::get_debug_settings,
            debug_commands::set_debug_logs_enabled,
            debug_commands::update_debug_settings,
            // Git config commands
            git_config_commands::get_git_config,
            git_config_commands::update_git_config,
            git_config_commands::get_available_git_modes,
            git_config_commands::test_git_execution_mode,
            git_config_commands::reset_git_config,
            git_commands::get_commit_history,
            git_commands::get_branches,
            git_commands::checkout_branch,
            git_commands::pull_current_branch,
            git_commands::push_current_branch,
            git_commands::fetch_remote,
            git_commands::discard_all_changes,
            git_commands::stage_all_changes,
            git_commands::unstage_all_changes,
            git_commands::open_folder_dialog,
            git_commands::get_file_diff,
            git_commands::get_staged_diff_summary,
            git_commands::add_to_gitignore,
            git_commands::delete_untracked_files,
            git_commands::delete_tracked_files,
            git_commands::get_file_stats,
            gitflow_commands::list_gitflow_branches,
            gitflow_commands::create_gitflow_branch,
            gitflow_commands::execute_gitflow_action,
            // Daily report commands
            daily_report_commands::get_available_repositories,
            daily_report_commands::get_repo_contributors,
            daily_report_commands::analyze_commits,
            daily_report_commands::generate_daily_report,
            daily_report_commands::save_report,
            daily_report_commands::get_history_reports,
            daily_report_commands::delete_report,
            // Enhanced daily report commands
            daily_report_commands::analyze_and_cache_commit,
            daily_report_commands::generate_enhanced_daily_report,
            daily_report_commands::get_commit_cache_status,
            daily_report_commands::cleanup_cache,
            // AI commands
            ai_commands::get_ai_config,
            ai_commands::update_ai_config,
            ai_commands::get_providers_info,
            ai_commands::get_models_for_provider,
            ai_commands::get_models_with_temp_config,
            ai_commands::test_provider_connection,
            ai_commands::test_connection_with_temp_config,
            ai_commands::refresh_provider_models,
            ai_commands::generate_commit_message_ai,
            ai_commands::generate_commit_with_template,
            ai_commands::get_prompt_templates,
            ai_commands::add_prompt_template,
            ai_commands::create_custom_template,
            ai_commands::update_template,
            ai_commands::delete_template,
            ai_commands::get_custom_templates,
            ai_commands::get_default_templates,
            ai_commands::check_template_two_phase_support,
            ai_commands::get_template_two_phase_status,
            ai_commands::get_conversation_history,
            ai_commands::clear_conversation_history,
            ai_commands::get_conversation_history_by_repository,
            ai_commands::get_repository_paths,
            ai_commands::should_use_layered_commit,
            ai_commands::execute_layered_commit,
            ai_commands::get_layered_sessions,
            ai_commands::get_conversation_records_by_session,
            ai_commands::check_and_process_file_tokens,
            ai_commands::cancel_layered_commit,
            ai_commands::check_first_time_setup,
            ai_commands::test_ai_connection,
            // Update commands
            update_commands::test_network_connection,
            update_commands::check_for_updates,
            update_commands::download_update,
            update_commands::install_update,
            update_commands::get_current_version,
            update_commands::cleanup_update_files,
            update_commands::check_update_file_exists,
            update_commands::get_update_settings,
            update_commands::save_update_settings,
            // System commands
            system_commands::open_browser_url,
            system_commands::get_app_info,
            // AI analysis commands
            ai_analysis_commands::analyze_commit_with_ai,
            ai_analysis_commands::batch_analyze_commits,
            ai_analysis_commands::generate_ai_enhanced_report,
            ai_analysis_commands::get_ai_analysis_templates,
            ai_analysis_commands::get_ai_analysis_config,
            ai_analysis_commands::set_ai_analysis_config,
            ai_analysis_commands::clear_analysis_cache,
            // Template management commands
            ai_analysis_commands::get_ai_templates,
            ai_analysis_commands::update_ai_template,
            ai_analysis_commands::reset_ai_template,
            // Template version management commands
            template_commands::get_all_templates,
            template_commands::get_template_details,
            template_commands::get_template_versions,
            template_commands::update_template_content,
            template_commands::switch_template_version,
            template_commands::get_system_template_updates,
            template_commands::apply_system_template_update,
            template_commands::create_versioned_custom_template,
            template_commands::delete_custom_template_versioned,
            template_commands::get_template_content_versioned,
            template_commands::revert_to_builtin_version,
            // Unified template management commands
            unified_template_commands::get_all_commit_templates,
            unified_template_commands::get_commit_template,
            unified_template_commands::update_commit_template_with_version,
            unified_template_commands::switch_commit_template_version,
            unified_template_commands::get_commit_template_version_history,
            unified_template_commands::check_commit_template_updates,
            unified_template_commands::apply_commit_template_update,
            unified_template_commands::get_all_unified_templates,
            unified_template_commands::get_unified_template,
            unified_template_commands::get_unified_template_version_history,
            unified_template_commands::update_unified_template,
            unified_template_commands::update_template_version,
            unified_template_commands::switch_unified_template_version,
            unified_template_commands::create_unified_custom_template,
            unified_template_commands::delete_unified_custom_template,
            unified_template_commands::check_unified_system_updates,
            unified_template_commands::apply_unified_system_update,
            unified_template_commands::get_all_templates_unified,
            unified_template_commands::batch_update_system_templates,
            unified_template_commands::reset_all_system_templates,
        ])
        .run(tauri::generate_context!());

    match app_result {
        Ok(_) => {
            write_startup_log("=== GitMentor-Lite 正常退出 ===");
        }
        Err(e) => {
            write_error_log(&format!("Tauri应用运行失败: {}", e));
            panic!("error while running tauri application: {}", e);
        }
    }
}
