use tauri::State;
use tokio::sync::Mutex;
use crate::core::git_engine::GitEngine;
use crate::core::llm_client::LLMClient;
use crate::types::git_types::{
    GitStatusResult, CommitRequest, CommitMessageResult, StageRequest,
    RevertRequest, CommitInfo, BranchInfo, GitOperationResult
};
use std::time::Instant;

#[tauri::command]
pub async fn select_repository(
    path: String,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<String, String> {
    let mut engine = git_engine.lock().await;
    engine.open_repository(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    Ok("Repository opened successfully".to_string())
}

#[tauri::command]
pub async fn get_git_status(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitStatusResult, String> {
    let engine = git_engine.lock().await;
    engine.get_status()
        .map_err(|e| format!("Failed to get git status: {}", e))
}

#[tauri::command]
pub async fn generate_commit_message(
    request: CommitRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
    llm_client: State<'_, LLMClient>,
) -> Result<CommitMessageResult, String> {
    let start_time = Instant::now();
    
    // Get Git status and diff info
    let git_status = {
        let engine = git_engine.lock().await;
        engine.get_status()
            .map_err(|e| format!("Failed to get git status: {}", e))?
    };
    
    let diff_summary = {
        let engine = git_engine.lock().await;
        engine.get_diff_summary(&request.selected_files)
            .map_err(|e| format!("Failed to get diff summary: {}", e))?
    };
    
    // Create simple prompt for MVP
    let prompt = format!(
        "Generate a concise commit message for the following changes:\n\nBranch: {}\nFiles changed: {}\n\nFile details:\n{}\n\nDiff summary:\n{}\n\nPlease provide a clear, concise commit message following conventional commit format.",
        git_status.branch,
        request.selected_files.len(),
        request.selected_files.join(", "),
        diff_summary
    );
    
    // Call LLM to generate commit message
    let commit_message = llm_client.generate_commit_message(&prompt).await
        .map_err(|e| format!("Failed to generate commit message: {}", e))?;
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    Ok(CommitMessageResult {
        message: commit_message,
        confidence: 0.85, // Simplified confidence for MVP
        processing_time_ms: processing_time,
    })
}

/// 暂存或取消暂存文件
/// 作者：Evilek
#[tauri::command]
pub async fn stage_files(
    request: StageRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine.stage_files(&request)
        .map_err(|e| format!("Failed to stage files: {}", e))
}

/// 提交更改
/// 作者：Evilek
#[tauri::command]
pub async fn commit_changes(
    request: CommitRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine.commit(&request)
        .map_err(|e| format!("Failed to commit: {}", e))
}

/// 回滚文件更改
/// 作者：Evilek
#[tauri::command]
pub async fn revert_files(
    request: RevertRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine.revert_files(&request)
        .map_err(|e| format!("Failed to revert files: {}", e))
}

/// 获取提交历史
/// 作者：Evilek
#[tauri::command]
pub async fn get_commit_history(
    limit: usize,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<Vec<CommitInfo>, String> {
    let engine = git_engine.lock().await;
    engine.get_commit_history(limit)
        .map_err(|e| format!("Failed to get commit history: {}", e))
}

/// 获取分支列表
/// 作者：Evilek
#[tauri::command]
pub async fn get_branches(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<Vec<BranchInfo>, String> {
    let engine = git_engine.lock().await;
    engine.get_branches()
        .map_err(|e| format!("Failed to get branches: {}", e))
}

/// 丢弃所有工作区更改
/// 作者：Evilek
#[tauri::command]
pub async fn discard_all_changes(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine.discard_all_changes()
        .map_err(|e| format!("Failed to discard changes: {}", e))
}

/// 暂存所有更改
/// 作者：Evilek
#[tauri::command]
pub async fn stage_all_changes(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine.stage_all_changes()
        .map_err(|e| format!("Failed to stage all changes: {}", e))
}

/// 取消暂存所有更改
/// 作者：Evilek
#[tauri::command]
pub async fn unstage_all_changes(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine.unstage_all_changes()
        .map_err(|e| format!("Failed to unstage all changes: {}", e))
}

#[tauri::command]
pub async fn open_folder_dialog(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

    // 打开文件夹选择对话框
    let folder_path = app_handle.dialog()
        .file()
        .blocking_pick_folder();

    match folder_path {
        Some(path) => {
            let path_str = path.to_string();

            // 验证选择的路径是否是一个有效的Git仓库
            match git2::Repository::open(&path_str) {
                Ok(_) => Ok(Some(path_str)),
                Err(_) => {
                    // 如果不是Git仓库，显示错误消息
                    app_handle.dialog()
                        .message("所选文件夹不是一个有效的Git仓库")
                        .kind(MessageDialogKind::Error)
                        .blocking_show();
                    Ok(None)
                }
            }
        },
        None => Ok(None), // 用户取消了选择
    }
}
