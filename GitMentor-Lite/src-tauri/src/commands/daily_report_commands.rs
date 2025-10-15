use crate::core::git_engine::GitEngine;
use crate::core::report_engine::{AnalysisEngine, CacheManager};
use crate::types::git_types::{
    AnalysisConfig, CommitAnalysis, CommitDetailAnalysis, Contributor, ImpactLevel, Report,
    ReportMeta, Repository,
};
use chrono::Utc;
use std::collections::HashMap;
use tauri::{Manager, State};
use tokio::sync::Mutex;
use uuid::Uuid;

// 日报生成相关命令 - Author: Evilek, Date: 2025-08-21

/// 获取可用仓库列表
#[tauri::command]
pub async fn get_available_repositories(
    repoPaths: Vec<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<Vec<Repository>, String> {
    let engine = git_engine.lock().await;
    engine
        .get_available_repositories(repoPaths)
        .map_err(|e| format!("Failed to get repositories: {}", e))
}

/// 获取仓库贡献者列表
#[tauri::command]
pub async fn get_repo_contributors(
    repoPaths: Vec<String>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<Vec<Contributor>, String> {
    let engine = git_engine.lock().await;
    engine
        .get_repo_contributors(repoPaths)
        .map_err(|e| format!("Failed to get contributors: {}", e))
}

/// 分析提交记录
#[tauri::command]
pub async fn analyze_commits(
    config: AnalysisConfig,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<CommitAnalysis, String> {
    let engine = git_engine.lock().await;
    engine
        .analyze_commits(config)
        .map_err(|e| format!("Failed to analyze commits: {}", e))
}

/// 分析单个提交并缓存结果
#[tauri::command]
pub async fn analyze_and_cache_commit(
    repo_path: String,
    commit_id: String,
    app_handle: tauri::AppHandle,
) -> Result<CommitDetailAnalysis, String> {
    // 获取应用数据目录
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // 创建分析引擎
    let analysis_engine = AnalysisEngine::new(&app_dir)
        .map_err(|e| format!("Failed to create analysis engine: {}", e))?;

    // 获取 Git 引擎
    let git_engine_state = app_handle.state::<Mutex<GitEngine>>();
    let git_engine = git_engine_state.lock().await;

    // 获取提交信息
    let commit_info = git_engine
        .get_commit_info(&repo_path, &commit_id)
        .map_err(|e| format!("Failed to get commit info: {}", e))?;

    // 获取文件差异信息
    let diff_info = git_engine
        .get_commit_diff(&repo_path, &commit_id)
        .map_err(|e| format!("Failed to get diff info: {}", e))?;

    // 分析提交
    let analysis = analysis_engine
        .analyze_commit(&repo_path, &commit_id, &commit_info, Some(&diff_info))
        .await
        .map_err(|e| format!("Failed to analyze commit: {}", e))?;

    Ok(analysis)
}

/// 生成增强版日报
#[tauri::command]
pub async fn generate_enhanced_daily_report(
    config: AnalysisConfig,
    app_handle: tauri::AppHandle,
) -> Result<Report, String> {
    println!("开始生成增强版日报...");
    println!(
        "配置信息: 仓库数量={}, 用户数量={}, 开始日期={}, 结束日期={}",
        config.repoPaths.len(),
        config.userEmails.len(),
        config.startDate,
        config.endDate
    );

    let report_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    // 获取应用数据目录
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    println!("应用数据目录: {:?}", app_dir);

    // 创建分析引擎
    let analysis_engine = AnalysisEngine::new(&app_dir)
        .map_err(|e| format!("Failed to create analysis engine: {}", e))?;
    println!("分析引擎创建成功");

    // 获取 Git 引擎
    let git_engine_state = app_handle.state::<Mutex<GitEngine>>();
    let git_engine = git_engine_state.lock().await;
    println!("Git 引擎获取成功");

    // 分析所有提交
    let mut all_analyses = Vec::new();
    let mut total_commits_found = 0;

    println!("开始分析 {} 个仓库", config.repoPaths.len());

    for repo_path in &config.repoPaths {
        println!("正在处理仓库: {}", repo_path);

        // 获取仓库在指定时间范围内的提交
        let commits = git_engine
            .get_commits_in_date_range(repo_path, &config.startDate, &config.endDate)
            .map_err(|e| format!("Failed to get commits: {}", e))?;

        total_commits_found += commits.len();
        println!(
            "仓库 {} 在 {} 至 {} 期间找到 {} 个提交",
            repo_path,
            config.startDate,
            config.endDate,
            commits.len()
        );

        // 过滤指定用户的提交
        let filtered_commits: Vec<_> = if config.userEmails.is_empty() {
            println!("未指定用户，包含所有提交");
            commits
        } else {
            let user_list = config.userEmails.join(", ");
            println!("指定用户: {}", user_list);
            commits
                .into_iter()
                .filter(|c| {
                    let include = config.userEmails.contains(&c.email);
                    if !include {
                        println!("跳过提交 {} (用户: {} 不在指定用户列表)", c.hash, c.email);
                    }
                    include
                })
                .collect()
        };

        println!(
            "仓库 {} 过滤后有 {} 个提交",
            repo_path,
            filtered_commits.len()
        );

        // 分析每个提交
        for commit in filtered_commits {
            println!(
                "正在分析提交: {} - {}",
                commit.hash,
                commit.message.lines().next().unwrap_or("")
            );
            match analysis_engine
                .analyze_commit(repo_path, &commit.hash, &commit, None)
                .await
            {
                Ok(analysis) => {
                    all_analyses.push(analysis);
                    println!("提交 {} 分析成功", commit.hash);
                }
                Err(e) => {
                    eprintln!("Failed to analyze commit {}: {}", commit.hash, e);
                    println!("提交 {} 分析失败: {}", commit.hash, e);
                }
            }
        }
    }

    println!(
        "总共找到 {} 个提交，成功分析 {} 个",
        total_commits_found,
        all_analyses.len()
    );

    if all_analyses.is_empty() {
        println!("警告: 没有找到任何提交分析结果");
    }

    // 生成报告
    println!("正在生成报告内容...");
    let title = format!("开发日报 - {} 至 {}", config.startDate, config.endDate);
    let content = generate_enhanced_report_content(&all_analyses, &config)?;

    println!("报告生成完成，内容长度: {} 字符", content.len());

    Ok(Report {
        id: report_id,
        title,
        content,
        format: "markdown".to_string(),
        created_at,
        config,
    })
}

/// 生成增强版报告内容
fn generate_enhanced_report_content(
    analyses: &[CommitDetailAnalysis],
    config: &AnalysisConfig,
) -> Result<String, String> {
    let mut content = String::new();

    // 报告标题
    content.push_str(&format!("# 开发日报\n\n"));
    content.push_str(&format!(
        "**报告周期**: {} 至 {}\n\n",
        config.startDate, config.endDate
    ));
    content.push_str(&format!(
        "**生成时间**: {}\n\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
    content.push_str(&format!("**总提交数**: {}\n\n", analyses.len()));

    if analyses.is_empty() {
        content.push_str("*在选定的时间范围内没有找到提交记录。*\n");
        return Ok(content);
    }

    // 统计概览
    let total_insertions: u32 = analyses.iter().map(|a| a.insertions).sum();
    let total_deletions: u32 = analyses.iter().map(|a| a.deletions).sum();
    let unique_files: std::collections::HashSet<_> = analyses
        .iter()
        .flat_map(|a| a.files_changed.iter().map(|f| &f.file_path))
        .collect();

    content.push_str("## 📊 统计概览\n\n");
    content.push_str(&format!("- **活跃仓库数**: {}\n", config.repoPaths.len()));
    content.push_str(&format!("- **文件变更**: {} 个文件\n", unique_files.len()));
    content.push_str(&format!(
        "- **代码变更**: +{} / -{}\n\n",
        total_insertions, total_deletions
    ));

    // 重要提交
    let important_commits: Vec<_> = analyses
        .iter()
        .filter(|a| a.impact_level == ImpactLevel::Critical || a.impact_level == ImpactLevel::High)
        .collect();

    if !important_commits.is_empty() {
        content.push_str("## 🎯 重要提交\n\n");
        for analysis in important_commits.iter().take(5) {
            let repo_name = std::path::Path::new(&analysis.repo_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown");

            content.push_str(&format!(
                "### [{}] {} - {}\n",
                match analysis.impact_level {
                    ImpactLevel::Critical => "Critical",
                    ImpactLevel::High => "High",
                    ImpactLevel::Medium => "Medium",
                    ImpactLevel::Low => "Low",
                },
                repo_name,
                &analysis.commit_id[..7]
            ));
            content.push_str(&format!(
                "**作者**: {} ({})\n",
                analysis.author, analysis.email
            ));
            content.push_str(&format!(
                "**时间**: {}\n",
                chrono::DateTime::from_timestamp(analysis.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            ));
            content.push_str(&format!("**摘要**: {}\n\n", analysis.summary));

            // 显示主要变更文件
            if !analysis.files_changed.is_empty() {
                content.push_str("**主要变更文件**:\n");
                for file in analysis.files_changed.iter().take(3) {
                    content.push_str(&format!(
                        "- {} ({})\n",
                        file.file_path,
                        match file.change_type {
                            crate::types::git_types::FileChangeType::Added => "新增",
                            crate::types::git_types::FileChangeType::Modified => "修改",
                            crate::types::git_types::FileChangeType::Deleted => "删除",
                            crate::types::git_types::FileChangeType::Renamed => "重命名",
                            crate::types::git_types::FileChangeType::Copied => "复制",
                        }
                    ));
                }
                content.push_str("\n");
            }
        }
    }

    // 按仓库统计
    content.push_str("## 📁 各仓库提交情况\n\n");
    let mut repo_stats: HashMap<String, Vec<&CommitDetailAnalysis>> = HashMap::new();

    for analysis in analyses {
        let repo_name = std::path::Path::new(&analysis.repo_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        repo_stats.entry(repo_name).or_default().push(analysis);
    }

    for (repo_name, repo_analyses) in repo_stats {
        content.push_str(&format!("### {}\n", repo_name));
        content.push_str(&format!("- **提交数**: {}\n", repo_analyses.len()));

        // 统计贡献者
        let mut contributors: HashMap<String, u32> = HashMap::new();
        for analysis in &repo_analyses {
            *contributors.entry(analysis.author.clone()).or_insert(0) += 1;
        }

        content.push_str("- **贡献者**: ");
        for (name, count) in contributors {
            content.push_str(&format!("{}({}) ", name, count));
        }
        content.push_str("\n\n");
    }

    // 按贡献者统计
    content.push_str("## 👥 贡献者详情\n\n");
    let mut contributor_stats: HashMap<String, Vec<&CommitDetailAnalysis>> = HashMap::new();

    for analysis in analyses {
        contributor_stats
            .entry(analysis.email.clone())
            .or_default()
            .push(analysis);
    }

    for (email, user_analyses) in contributor_stats {
        if let Some(first) = user_analyses.first() {
            content.push_str(&format!("### {} ({})\n", first.author, email));
            content.push_str(&format!("- **提交数**: {}\n", user_analyses.len()));

            // 统计影响级别
            let mut impact_counts = HashMap::new();
            for analysis in &user_analyses {
                *impact_counts
                    .entry(format!("{:?}", analysis.impact_level))
                    .or_insert(0) += 1;
            }

            content.push_str("- **影响级别分布**: ");
            for (level, count) in impact_counts {
                content.push_str(&format!("{}({}) ", level, count));
            }
            content.push_str("\n\n");
        }
    }

    // 标签统计
    content.push_str("## 🏷️ 标签统计\n\n");
    let mut tag_counts: HashMap<String, u32> = HashMap::new();
    for analysis in analyses {
        for tag in &analysis.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));

    for (tag, count) in sorted_tags.iter().take(10) {
        content.push_str(&format!("- **{}**: {} 次\n", tag, count));
    }

    content.push_str("\n---\n*报告由 GitMentor 自动生成*\n");

    Ok(content)
}

/// 获取提交缓存状态
#[tauri::command]
pub async fn get_commit_cache_status(
    repo_path: String,
    commit_ids: Vec<String>,
    app_handle: tauri::AppHandle,
) -> Result<HashMap<String, bool>, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let cache_manager = CacheManager::new(&app_dir);
    let mut status = HashMap::new();

    for commit_id in commit_ids {
        status.insert(
            commit_id.clone(),
            cache_manager.cache_exists(&repo_path, &commit_id),
        );
    }

    Ok(status)
}

/// 清理过期缓存
#[tauri::command]
pub async fn cleanup_cache(days_old: u64, app_handle: tauri::AppHandle) -> Result<bool, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let cache_manager = CacheManager::new(&app_dir);
    cache_manager
        .cleanup_old_cache(days_old)
        .map_err(|e| format!("Failed to cleanup cache: {}", e))?;

    Ok(true)
}

/// 生成日报（保持向后兼容）
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
fn generate_report_content(
    analysis: &CommitAnalysis,
    _template: Option<&str>,
) -> Result<String, String> {
    let mut content = String::new();

    // 报告标题
    content.push_str(&format!("# 开发日报\n\n"));
    content.push_str(&format!("**分析周期**: {}\n\n", analysis.analysis_period));
    content.push_str(&format!("**总提交数**: {}\n\n", analysis.total_commits));

    // 按用户统计
    content.push_str("## 👥 用户提交统计\n\n");
    for (email, commits) in &analysis.commits_by_user {
        let user_name = commits
            .first()
            .map(|c| c.author.as_str())
            .unwrap_or("Unknown");
        content.push_str(&format!("### {} ({})\n", user_name, email));
        content.push_str(&format!("- 提交数量: {}\n", commits.len()));
        content.push_str("- 主要提交:\n");

        for (i, commit) in commits.iter().take(5).enumerate() {
            content.push_str(&format!(
                "  {}. `{}` {}\n",
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
    content.push_str(&format!(
        "---\n*报告生成时间: {}*\n",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

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
