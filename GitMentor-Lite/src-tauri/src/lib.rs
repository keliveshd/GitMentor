mod commands;
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

use commands::{ai_commands, debug_commands, git_commands, git_config_commands};
use core::{
    ai_manager::AIManager,
    git_config::GitConfigManager,
    git_engine::GitEngine,
    llm_client::{LLMClient, LLMConfig},
};
use tokio::sync::Mutex;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 过滤PNG警告和其他不必要的日志
    std::env::set_var("RUST_LOG", "warn,libpng=off,image=off");

    // Initialize configuration directory
    let config_dir = std::env::current_dir().unwrap().join(".config");

    // Initialize Git configuration
    let git_config_path = config_dir.join("git_config.json");
    let git_config_manager =
        GitConfigManager::new(git_config_path).expect("Failed to initialize Git Config Manager");
    let git_config = git_config_manager.get_config().clone();

    // Initialize components
    let git_engine = Mutex::new(GitEngine::new_with_config(git_config));
    let git_config_manager = Mutex::new(git_config_manager);
    let llm_config = LLMConfig::default();
    let llm_client = LLMClient::new(llm_config);

    // Initialize AI Manager
    let ai_config_path = config_dir.join("ai_config.json");
    let ai_manager =
        Mutex::new(AIManager::new(ai_config_path).expect("Failed to initialize AI Manager"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
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
            git_commands::discard_all_changes,
            git_commands::stage_all_changes,
            git_commands::unstage_all_changes,
            git_commands::open_folder_dialog,
            git_commands::get_file_diff,
            git_commands::get_staged_diff_summary,
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
            ai_commands::should_use_layered_commit,
            ai_commands::execute_layered_commit,
            ai_commands::get_layered_sessions,
            ai_commands::get_conversation_records_by_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
