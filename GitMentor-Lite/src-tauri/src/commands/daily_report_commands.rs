use tauri::State;
use tokio::sync::Mutex;
use crate::core::git_engine::GitEngine;
use crate::types::git_types::{
    Repository, Contributor, AnalysisConfig, CommitAnalysis, Report, ReportMeta
};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

// 日报生成相关命令 - Author: Evilek, Date: 2025-08-21

/// 获取可用仓库列表
#[tauri::command]
pub async fn get_available_repositories(
    repoPaths: Vec<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<Vec<Repository>, String> {
    let engine = git_engine.lock().await;
    engine.get_available_repositories(repoPaths)
        .map_err(|e| format!("Failed to get repositories: {}", e))
}

/// 获取仓库贡献者列表
#[tauri::command]
pub async fn get_repo_contributors(
    repoPaths: Vec<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<Vec<Contributor>, String> {
    let engine = git_engine.lock().await;
    engine.get_repo_contributors(repoPaths)
        .map_err(|e| format!("Failed to get contributors: {}", e))
}

/// 分析提交记录
#[tauri::command]
pub async fn analyze_commits(
    config: AnalysisConfig,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<CommitAnalysis, String> {
    let engine = git_engine.lock().await;
    engine.analyze_commits(config)
        .map_err(|e| format!("Failed to analyze commits: {}", e))
}

/// 生成日报
#[tauri::command]
pub async fn generate_daily_report(
    analysis: CommitAnalysis,
    template: Option<String>,
) -> Result<Report, String> {
    let report_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    // 生成报告标题
    let title = format!("Daily Report - {}", analysis.analysis_period);
    
    // 使用默认模板生成报告内容
    let content = generate_report_content(&analysis, template.as_deref())?;
    
    Ok(Report {
        id: report_id,
        title,
        content,
        format: "markdown".to_string(),
        created_at,
        config: AnalysisConfig {
            repoPaths: analysis.commits_by_repo.keys().cloned().collect(),
            userEmails: analysis.commits_by_user.keys().cloned().collect(),
            startDate: "".to_string(), // TODO: 从analysis中提取
            endDate: "".to_string(),   // TODO: 从analysis中提取
        },
    })
}

/// 生成报告内容
fn generate_report_content(analysis: &CommitAnalysis, _template: Option<&str>) -> Result<String, String> {
    let mut content = String::new();
    
    // 报告标题
    content.push_str(&format!("# 开发日报\n\n"));
    content.push_str(&format!("**分析周期**: {}\n\n", analysis.analysis_period));
    content.push_str(&format!("**总提交数**: {}\n\n", analysis.total_commits));
    
    // 按用户统计
    content.push_str("## 👥 用户提交统计\n\n");
    for (email, commits) in &analysis.commits_by_user {
        let user_name = commits.first()
            .map(|c| c.author.as_str())
            .unwrap_or("Unknown");
        content.push_str(&format!("### {} ({})\n", user_name, email));
        content.push_str(&format!("- 提交数量: {}\n", commits.len()));
        content.push_str("- 主要提交:\n");
        
        for (i, commit) in commits.iter().take(5).enumerate() {
            content.push_str(&format!("  {}. `{}` {}\n", 
                i + 1, 
                commit.short_hash, 
                commit.message.lines().next().unwrap_or("")
            ));
        }
        content.push_str("\n");
    }
    
    // 按仓库统计
    content.push_str("## 📁 仓库提交统计\n\n");
    for (repo_name, commits) in &analysis.commits_by_repo {
        content.push_str(&format!("### {}\n", repo_name));
        content.push_str(&format!("- 提交数量: {}\n", commits.len()));
        
        // 统计该仓库的用户贡献
        let mut user_commits: HashMap<String, usize> = HashMap::new();
        for commit in commits {
            *user_commits.entry(commit.author.clone()).or_insert(0) += 1;
        }
        
        content.push_str("- 贡献者:\n");
        for (user, count) in user_commits {
            content.push_str(&format!("  - {}: {} 次提交\n", user, count));
        }
        content.push_str("\n");
    }
    
    // 文件变更统计
    if !analysis.file_changes.is_empty() {
        content.push_str("## 📝 文件变更统计\n\n");
        let mut sorted_files: Vec<_> = analysis.file_changes.iter().collect();
        sorted_files.sort_by(|a, b| b.1.cmp(a.1));
        
        content.push_str("| 文件 | 变更次数 |\n");
        content.push_str("|------|----------|\n");
        
        for (file, count) in sorted_files.iter().take(10) {
            content.push_str(&format!("| {} | {} |\n", file, count));
        }
        content.push_str("\n");
    }
    
    // 报告生成时间
    content.push_str(&format!("---\n*报告生成时间: {}*\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    Ok(content)
}

// TODO: 实现历史报告管理功能
/// 保存报告
#[tauri::command]
pub async fn save_report(report: Report) -> Result<String, String> {
    // TODO: 实现报告保存到本地存储
    Ok(report.id)
}

/// 获取历史报告列表
#[tauri::command]
pub async fn get_history_reports() -> Result<Vec<ReportMeta>, String> {
    // TODO: 从本地存储加载历史报告
    Ok(vec![])
}

/// 删除报告
#[tauri::command]
pub async fn delete_report(_report_id: String) -> Result<bool, String> {
    // TODO: 从本地存储删除报告
    Ok(true)
}
