use tauri::State;

use crate::core::git_engine::GitEngine;
use crate::types::git_types::{
    GitOperationResult, GitflowActionRequest, GitflowCreateRequest, GitflowSummary,
};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn list_gitflow_branches(
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitflowSummary, String> {
    let engine = git_engine.lock().await;
    engine
        .list_gitflow_branches()
        .map_err(|e| format!("获取 Gitflow 分支失败: {}", e))
}

#[tauri::command]
pub async fn create_gitflow_branch(
    request: GitflowCreateRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .create_gitflow_branch(&request)
        .map_err(|e| format!("创建 Gitflow 分支失败: {}", e))
}

#[tauri::command]
pub async fn execute_gitflow_action(
    request: GitflowActionRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .execute_gitflow_action(&request)
        .map_err(|e| format!("执行 Gitflow 操作失败: {}", e))
}
