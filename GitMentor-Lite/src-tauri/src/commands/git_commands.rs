use tauri::State;
// Git 命令集合：仅做参数校验与调度，核心逻辑在 GitEngine
// Author: Evilek, Date: 2025-08-11
// 返回值统一走 Result<.., String>，错误别往上抛屎山，格式化清楚点

use crate::core::git_engine::GitEngine;
use crate::core::llm_client::LLMClient;
use crate::types::git_types::{
    BranchInfo, CommitInfo, CommitMessageResult, CommitRequest, FileDiffRequest, FileDiffResult,
    GitOperationResult, GitStatusResult, RemoteConfiguration, RevertRequest, StageRequest,
};
use std::time::Instant;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn select_repository(
    path: String,
    app_handle: tauri::AppHandle,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<String, String> {
    let mut engine = git_engine.lock().await;
    engine
        .open_repository(&path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;
    engine
        .start_repo_watcher(app_handle)
        .map_err(|e| format!("Failed to start repository watcher: {}", e))?;
    Ok("Repository opened successfully".to_string())
}

#[tauri::command]
pub async fn close_repository(git_engine: State<'_, Mutex<GitEngine>>) -> Result<(), String> {
    let mut engine = git_engine.lock().await;
    engine.close_repository();
    Ok(())
}

#[tauri::command]
pub async fn stop_repo_watcher(git_engine: State<'_, Mutex<GitEngine>>) -> Result<(), String> {
    let mut engine = git_engine.lock().await;
    engine.stop_repo_watcher();
    Ok(())
}

#[tauri::command]
pub async fn get_git_status(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitStatusResult, String> {
    println!("[DEBUG] 前端请求获取Git状态（智能方式）");
    let start_time = Instant::now();

    let engine = git_engine.lock().await;
    let result = engine
        .get_status()
        .map_err(|e| format!("Failed to get git status: {}", e));

    println!("[DEBUG] Git状态命令完成，耗时: {:?}", start_time.elapsed());
    result
}

#[tauri::command]
pub async fn get_remote_configuration(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<RemoteConfiguration, String> {
    let engine = git_engine.lock().await;
    engine
        .get_remote_configuration()
        .map_err(|e| format!("Failed to get remote configuration: {}", e))
}

#[tauri::command]
pub async fn add_remote(
    name: String,
    url: String,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .add_remote(&name, &url)
        .map_err(|e| format!("Failed to add remote: {}", e))
}

#[tauri::command]
pub async fn update_remote(
    name: String,
    url: String,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .update_remote(&name, &url)
        .map_err(|e| format!("Failed to update remote: {}", e))
}

#[tauri::command]
pub async fn remove_remote(
    name: String,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .remove_remote(&name)
        .map_err(|e| format!("Failed to remove remote: {}", e))
}

#[tauri::command]
pub async fn set_branch_upstream(
    branch: String,
    remote: String,
    remote_branch: String,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .set_branch_upstream(&branch, &remote, &remote_branch)
        .map_err(|e| format!("Failed to set branch upstream: {}", e))
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
        engine
            .get_status()
            .map_err(|e| format!("Failed to get git status: {}", e))?
    };

    let diff_summary = {
        let engine = git_engine.lock().await;
        engine
            .get_diff_summary(&request.selected_files)
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
    let commit_message = llm_client
        .generate_commit_message(&prompt)
        .await
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
    engine
        .stage_files(&request)
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
    engine
        .commit(&request)
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
    engine
        .revert_files(&request)
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
    engine
        .get_commit_history(limit)
        .map_err(|e| format!("Failed to get commit history: {}", e))
}

/// 获取分支列表
/// 作者：Evilek
#[tauri::command]
pub async fn get_branches(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<Vec<BranchInfo>, String> {
    let engine = git_engine.lock().await;
    engine
        .get_branches()
        .map_err(|e| format!("Failed to get branches: {}", e))
}

/// 切换分支
/// 作者：Evilek
/// 编写日期：2025-08-12
#[tauri::command]
pub async fn checkout_branch(
    branch_name: String,
    is_remote: bool,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .checkout_branch(&branch_name, is_remote)
        .map_err(|e| format!("Failed to checkout branch: {}", e))
}

/// 拉取当前分支
/// 作者：Evilek
/// 编写日期：2025-08-12
#[tauri::command]
pub async fn pull_current_branch(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .pull_current_branch()
        .map_err(|e| format!("Failed to pull: {}", e))
}

/// 推送当前分支
/// 作者：Evilek
/// 编写日期：2025-08-12
#[tauri::command]
pub async fn push_current_branch(
    force: bool,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .push_current_branch(force)
        .map_err(|e| format!("Failed to push: {}", e))
}

/// 获取远程更新
/// 作者：Evilek
/// 编写日期：2025-08-12
#[tauri::command]
pub async fn fetch_remote(
    remote_name: Option<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .fetch_remote(remote_name.as_deref())
        .map_err(|e| format!("Failed to fetch: {}", e))
}

/// 丢弃所有工作区更改
/// 作者：Evilek
#[tauri::command]
pub async fn discard_all_changes(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .discard_all_changes()
        .map_err(|e| format!("Failed to discard changes: {}", e))
}

/// 暂存所有更改
/// 作者：Evilek
#[tauri::command]
pub async fn stage_all_changes(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .stage_all_changes()
        .map_err(|e| format!("Failed to stage all changes: {}", e))
}

/// 取消暂存所有更改
/// 作者：Evilek
#[tauri::command]
pub async fn unstage_all_changes(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .unstage_all_changes()
        .map_err(|e| format!("Failed to unstage all changes: {}", e))
}

#[tauri::command]
pub async fn open_folder_dialog(
    app_handle: tauri::AppHandle,
    require_git_repo: Option<bool>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

    let require_git_repo = require_git_repo.unwrap_or(true);

    // 打开文件夹选择对话框
    let folder_path = app_handle.dialog().file().blocking_pick_folder();

    match folder_path {
        Some(path) => {
            let path_str = path.to_string();

            if !require_git_repo {
                return Ok(Some(path_str));
            }

            // 验证选择的路径是否是一个有效的Git仓库
            match git2::Repository::open(&path_str) {
                Ok(_) => Ok(Some(path_str)),
                Err(_) => {
                    // 如果不是Git仓库，显示错误消息
                    app_handle
                        .dialog()
                        .message("所选文件夹不是一个有效的Git仓库")
                        .kind(MessageDialogKind::Error)
                        .blocking_show();
                    Ok(None)
                }
            }
        }
        None => Ok(None), // 用户取消了选择
    }
}

/// 获取文件差异
/// 作者：Evilek
/// 编写日期：2025-01-18
#[tauri::command]
pub async fn get_file_diff(
    request: FileDiffRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<FileDiffResult, String> {
    let engine = git_engine.lock().await;
    engine
        .get_file_diff_detailed(&request)
        .map_err(|e| format!("Failed to get file diff: {}", e))
}

/// 添加文件到 .gitignore
/// 作者：Evilek
/// 编写日期：2025-08-11
#[tauri::command]
pub async fn add_to_gitignore(
    file_paths: Vec<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .add_to_gitignore(&file_paths)
        .map_err(|e| format!("Failed to add to gitignore: {}", e))
}

/// 删除未跟踪文件
/// 作者：Evilek
/// 编写日期：2025-08-11
#[tauri::command]
pub async fn delete_untracked_files(
    file_paths: Vec<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .delete_untracked_files(&file_paths)
        .map_err(|e| format!("Failed to delete untracked files: {}", e))
}

/// 删除已跟踪文件
/// 作者：Evilek
/// 编写日期：2025-08-11
#[tauri::command]
pub async fn delete_tracked_files(
    file_paths: Vec<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .delete_tracked_files(&file_paths)
        .map_err(|e| format!("Failed to delete tracked files: {}", e))
}

/// 获取暂存文件的差异摘要（用于AI生成）
/// 作者：Evilek
/// 编写日期：2025-07-27
#[tauri::command]
pub async fn get_staged_diff_summary(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<String, String> {
    let engine = git_engine.lock().await;

    // 获取Git状态
    let git_status = engine
        .get_status()
        .map_err(|e| format!("Failed to get git status: {}", e))?;

    // 获取暂存文件列表
    let staged_files: Vec<String> = git_status
        .staged_files
        .iter()
        .map(|f| f.path.clone())
        .collect();

    if staged_files.is_empty() {
        return Ok("No staged files found.".to_string());
    }

    // 获取差异摘要
    engine
        .get_diff_summary(&staged_files)
        .map_err(|e| format!("Failed to get diff summary: {}", e))
}

/// 获取文件统计信息（用于文件监控）
/// 作者：Evilek
/// 编写日期：2025-01-15
#[derive(serde::Serialize)]
pub struct FileStats {
    pub modified: Option<String>,
    pub size: u64,
    pub exists: bool,
}

#[tauri::command]
pub async fn get_file_stats(path: String) -> Result<FileStats, String> {
    use std::fs;
    use std::time::SystemTime;

    match fs::metadata(&path) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|duration| {
                    let timestamp = duration.as_secs();
                    chrono::DateTime::from_timestamp(timestamp as i64, 0)
                        .unwrap_or_default()
                        .to_rfc3339()
                });

            Ok(FileStats {
                modified,
                size: metadata.len(),
                exists: true,
            })
        }
        Err(_) => Ok(FileStats {
            modified: None,
            size: 0,
            exists: false,
        }),
    }
}
