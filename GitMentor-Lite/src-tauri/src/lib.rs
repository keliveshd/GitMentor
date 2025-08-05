mod commands;
mod core;
mod types;
mod utils;

use commands::{ai_commands, git_commands};
use core::{
    ai_manager::AIManager,
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
    // Initialize components
    let git_engine = Mutex::new(GitEngine::new());
    let llm_config = LLMConfig::default();
    let llm_client = LLMClient::new(llm_config);

    // Initialize AI Manager
    let config_dir = std::env::current_dir().unwrap().join(".config");
    let ai_config_path = config_dir.join("ai_config.json");
    let ai_manager =
        Mutex::new(AIManager::new(ai_config_path).expect("Failed to initialize AI Manager"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(git_engine)
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
