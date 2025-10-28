use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use tauri::State;
use tokio::sync::Mutex;

use crate::core::git_engine::GitEngine;
use crate::core::llm_client::LLMClient;
use crate::types::git_types::{
    CheckoutRequest, CheckoutResult, CommitMessageResult, GitOperationResult, RemoteConfigRequest,
};

/// 克隆仓库
#[tauri::command]
pub async fn clone_repository(
    request: CheckoutRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<CheckoutResult, String> {
    let engine_clone = {
        let engine = git_engine.lock().await;
        engine.clone()
    };

    let request_clone = request.clone();

    tokio::task::spawn_blocking(move || engine_clone.clone_repository(&request_clone))
        .await
        .map_err(|e| format!("Failed to spawn clone task: {}", e))?
        .map_err(|e| format!("Failed to clone repository: {}", e))
}

/// 配置远程仓库
#[tauri::command]
pub async fn configure_remote(
    request: RemoteConfigRequest,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<GitOperationResult, String> {
    let engine = git_engine.lock().await;
    engine
        .configure_remote(&request)
        .map_err(|e| format!("Failed to configure remote: {}", e))
}

/// 验证远程仓库连接
#[tauri::command]
pub async fn validate_remote_connection(
    url: String,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<bool, String> {
    let engine_clone = {
        let engine = git_engine.lock().await;
        engine.clone()
    };

    tokio::task::spawn_blocking(move || engine_clone.validate_remote_connection(&url))
        .await
        .map_err(|e| format!("Failed to spawn validation task: {}", e))?
        .map_err(|e| format!("Failed to validate remote: {}", e))
}

/// 生成初始提交信息
#[tauri::command]
pub async fn generate_initial_commit_message(
    repository_path: String,
    llm_client: State<'_, LLMClient>,
) -> Result<CommitMessageResult, String> {
    if repository_path.trim().is_empty() {
        return Err("Repository path cannot be empty".to_string());
    }

    let analysis_path = repository_path.clone();
    let structure_summary =
        tokio::task::spawn_blocking(move || analyze_repository_structure(&analysis_path))
            .await
            .map_err(|e| format!("Failed to analyze repository structure: {}", e))??;

    let prompt = build_initial_commit_prompt(&structure_summary);
    let start_time = Instant::now();

    let message = llm_client
        .generate_commit_message(&prompt)
        .await
        .map_err(|e| format!("Failed to generate commit message: {}", e))?;

    Ok(CommitMessageResult {
        message,
        confidence: 0.75,
        processing_time_ms: start_time.elapsed().as_millis() as u64,
    })
}

fn analyze_repository_structure(path: &str) -> Result<String, String> {
    let root = Path::new(path);
    if !root.exists() {
        return Err("指定的仓库路径不存在".to_string());
    }

    let mut directories = Vec::new();
    let mut files = Vec::new();
    let mut extension_counts: HashMap<String, usize> = HashMap::new();

    let entries = fs::read_dir(root).map_err(|e| format!("无法读取仓库目录: {}", e))?;
    for entry_result in entries {
        let entry = entry_result.map_err(|e| format!("无法读取仓库文件: {}", e))?;
        let metadata = entry
            .metadata()
            .map_err(|e| format!("无法获取文件元数据: {}", e))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if metadata.is_dir() {
            let count = fs::read_dir(entry.path())
                .ok()
                .and_then(|iter| Some(iter.count()))
                .unwrap_or(0);
            directories.push((name.to_string(), count));
        } else if metadata.is_file() {
            let ext = entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_string();
            if !ext.is_empty() {
                *extension_counts.entry(ext.clone()).or_insert(0) += 1;
            }
            files.push(name.to_string());
        }

        if directories.len() >= 8 && files.len() >= 12 {
            break;
        }
    }

    directories.sort_by(|a, b| b.1.cmp(&a.1));
    files.sort();

    let readme_preview = find_readme_preview(root);

    let mut summary = String::new();

    if !directories.is_empty() {
        summary.push_str("主要目录:\n");
        for (name, count) in directories.iter().take(8) {
            summary.push_str(&format!("- {} (约 {} 个项目)\n", name, count));
        }
    }

    if !files.is_empty() {
        summary.push_str("\n关键文件:\n");
        for name in files.iter().take(12) {
            summary.push_str(&format!("- {}\n", name));
        }
    }

    if !extension_counts.is_empty() {
        summary.push_str("\n语言/类型分布:\n");
        let mut items: Vec<_> = extension_counts.into_iter().collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        for (ext, count) in items.into_iter().take(6) {
            summary.push_str(&format!("- {}: {} 个文件\n", ext, count));
        }
    }

    if let Some(preview) = readme_preview {
        summary.push_str("\nREADME 摘要:\n");
        summary.push_str(&preview);
        summary.push('\n');
    }

    if summary.trim().is_empty() {
        summary.push_str("仓库为空或尚未发现文件。");
    }

    Ok(summary)
}

fn find_readme_preview(root: &Path) -> Option<String> {
    const PREVIEW_LIMIT: usize = 240;

    let readme_candidates = ["README.md", "README.MD", "readme.md", "README.txt"];
    let mut preview = None;

    for name in readme_candidates {
        let candidate = root.join(name);
        if candidate.exists() && candidate.is_file() {
            preview = fs::read_to_string(&candidate)
                .ok()
                .map(|content| {
                    content
                        .trim()
                        .lines()
                        .take(10)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .map(|content| {
                    if content.len() > PREVIEW_LIMIT {
                        format!("{}...", &content[..PREVIEW_LIMIT])
                    } else {
                        content
                    }
                });
            if preview.is_some() {
                break;
            }
        }
    }

    preview
}

fn build_initial_commit_prompt(structure_summary: &str) -> String {
    format!(
        "你是一名资深的 Git 专家，请阅读以下仓库结构，并为首次提交生成规范的初始提交信息。\
\n\n仓库结构概览：\n{}\n\n请输出格式如下：\n1. 一行简洁的提交标题，概括仓库核心功能（<=50 字）。\
\n2. 一段简要描述，说明主要目录与关键文件的作用。\n3. 可选的后续工作或待办事项建议。\
\n\n请使用简体中文，保持专业语气。",
        structure_summary
    )
}
