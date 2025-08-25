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
    ai_commands, daily_report_commands, debug_commands, git_commands, git_config_commands,
    system_commands, update_commands,
};
use core::{
    ai_manager::AIManager,
    git_config::GitConfigManager,
    git_engine::GitEngine,
    llm_client::{LLMClient, LLMConfig},
};
use std::fs::OpenOptions;
use std::io::Write;
use tokio::sync::Mutex;

/// 写入启动日志到文件
/// Author: Evilek, Date: 2025-01-09
fn write_startup_log(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_message = format!("[{}] {}\n", timestamp, message);

    // 写入到当前目录的startup.log文件
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("startup.log")
    {
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

    // 写入到当前目录的startup.log文件
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("startup.log")
    {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    write_startup_log("=== GitMentor-Lite 启动开始 ===");
    write_startup_log("Author: Evilek, Date: 2025-01-09");

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
            Mutex::new(manager)
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
            git_commands::get_git_status,
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
            // Daily report commands
            daily_report_commands::get_available_repositories,
            daily_report_commands::get_repo_contributors,
            daily_report_commands::analyze_commits,
            daily_report_commands::generate_daily_report,
            daily_report_commands::save_report,
            daily_report_commands::get_history_reports,
            daily_report_commands::delete_report,
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
