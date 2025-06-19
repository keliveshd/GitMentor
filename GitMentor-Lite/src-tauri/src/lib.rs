mod commands;
mod core;
mod types;

use commands::git_commands;
use core::{git_engine::GitEngine, llm_client::{LLMClient, LLMConfig}};
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(git_engine)
        .manage(llm_client)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
