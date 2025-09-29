/// AI分析相关命令
/// 作者：Evilek
/// 编写日期：2025-09-16
/// 
/// 此模块提供了AI增强分析的相关命令

use tauri::{Manager, Emitter};
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
use crate::core::git_engine::GitEngine;
use crate::core::report_engine::AnalysisEngine;
use crate::core::ai_manager::AIManager;
use crate::core::ai_analysis_prompts::PromptTemplateManager;
use crate::types::git_types::{
    CommitDetailAnalysis, AnalysisDepth, AIAnalysisConfig,
    AnalysisProgress, AnalysisConfig, Report
};
use crate::core::ai_analysis_prompts::PromptTemplate;
use uuid::Uuid;
use chrono::Utc;

/// 分析单个提交（使用AI增强）
#[tauri::command]
pub async fn analyze_commit_with_ai(
    repo_path: String,
    commit_id: String,
    app_handle: tauri::AppHandle,
    depth: Option<AnalysisDepth>,
    include_code_review: Option<bool>,
    force_refresh: Option<bool>,
) -> Result<CommitDetailAnalysis, String> {
    println!("开始AI分析提交: {} in {}", commit_id, repo_path);
    
    // 获取应用数据目录
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    // 创建分析引擎
    let mut analysis_engine = AnalysisEngine::new(&app_dir)
        .map_err(|e| format!("Failed to create analysis engine: {}", e))?;
    
    // 获取AI管理器和配置
    let ai_manager_state = app_handle.state::<Arc<RwLock<AIManager>>>();
    let ai_manager = ai_manager_state.read().await;
    let ai_config_data = ai_manager.get_config().await;
    analysis_engine = analysis_engine.with_ai_manager(ai_manager_state.inner().clone());
    
    // 应用AI配置（使用实际配置的模型）
    let ai_config = AIAnalysisConfig {
        model: ai_config_data.base.model.clone(),
        depth: depth.unwrap_or(AnalysisDepth::Detailed),
        enable_code_review: include_code_review.unwrap_or(true),
        ..AIAnalysisConfig::default()
    };
    analysis_engine = analysis_engine.with_ai_config(ai_config);
    
    // 获取Git引擎
    let git_engine_state = app_handle.state::<Mutex<GitEngine>>();
    let git_engine = git_engine_state.lock().await;
    
    // 如果强制刷新，清除缓存
    if force_refresh.unwrap_or(false) {
        let cache_path = analysis_engine.cache_manager()
            .get_cache_path(&repo_path, &commit_id);
        if cache_path.exists() {
            std::fs::remove_file(&cache_path)
                .map_err(|e| format!("Failed to remove cache: {}", e))?;
            println!("已清除缓存: {:?}", cache_path);
        }
    }
    
    // 获取提交信息
    let commit_info = git_engine.get_commit_info(&repo_path, &commit_id)
        .map_err(|e| format!("Failed to get commit info: {}", e))?;
    
    // 获取文件差异信息
    let diff_info = git_engine.get_commit_diff(&repo_path, &commit_id)
        .map_err(|e| format!("Failed to get diff info: {}", e))?;
    
    // 分析提交（包含AI分析）
    let analysis = analysis_engine.analyze_commit(&repo_path, &commit_id, &commit_info, Some(&diff_info))
        .await
        .map_err(|e| format!("Failed to analyze commit: {}", e))?;
    
    println!("AI分析完成: {}", commit_id);
    
    Ok(analysis)
}

/// 批量分析提交（带进度反馈）
#[tauri::command]
pub async fn batch_analyze_commits(
    _repo_paths: Vec<String>,
    commit_ids: Vec<(String, String)>, // (repo_path, commit_id)
    app_handle: tauri::AppHandle,
    depth: Option<AnalysisDepth>,
    include_code_review: Option<bool>,
    session_id: Option<String>,
) -> Result<AnalysisProgress, String> {
    let session_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let total_steps = commit_ids.len() as u32;
    
    println!("开始批量分析，会话ID: {}, 总提交数: {}", session_id, total_steps);
    
    // 创建进度对象
    let progress = AnalysisProgress {
        session_id: session_id.clone(),
        total_steps,
        current_step: 0,
        current_status: "初始化分析引擎...".to_string(),
        progress_percentage: 0,
        current_file: None,
    };
    
    // 发送初始进度
    app_handle.emit("analysis-progress", &progress)
        .map_err(|e| format!("Failed to emit progress: {}", e))?;
    
    // 获取应用数据目录
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    // 创建分析引擎
    let mut analysis_engine = AnalysisEngine::new(&app_dir)
        .map_err(|e| format!("Failed to create analysis engine: {}", e))?;
    
    // 获取AI管理器和配置
    let ai_manager_state = app_handle.state::<Arc<RwLock<AIManager>>>();
    let ai_manager = ai_manager_state.read().await;
    let ai_config_data = ai_manager.get_config().await;
    analysis_engine = analysis_engine.with_ai_manager(ai_manager_state.inner().clone());
    
    // 应用AI配置（使用实际配置的模型）
    let ai_config = AIAnalysisConfig {
        model: ai_config_data.base.model.clone(),
        depth: depth.unwrap_or(AnalysisDepth::Detailed),
        enable_code_review: include_code_review.unwrap_or(true),
        ..AIAnalysisConfig::default()
    };
    analysis_engine = analysis_engine.with_ai_config(ai_config);
    
    // 获取Git引擎
    let git_engine_state = app_handle.state::<Mutex<GitEngine>>();
    let git_engine = git_engine_state.lock().await;
    
    // 分析每个提交
    let mut successful_analyses = 0;
    let mut failed_analyses = 0;
    
    for (i, (repo_path, commit_id)) in commit_ids.iter().enumerate() {
        let step = i as u32 + 1;
        let percentage = (step * 100) / total_steps;
        
        // 更新进度
        let progress = AnalysisProgress {
            session_id: session_id.clone(),
            total_steps,
            current_step: step,
            current_status: format!("分析提交: {}", commit_id),
            progress_percentage: percentage,
            current_file: Some(format!("{}:{}", repo_path, commit_id)),
        };
        
        app_handle.emit("analysis-progress", &progress)
            .map_err(|e| format!("Failed to emit progress: {}", e))?;
        
        // 检查缓存是否存在
        if !analysis_engine.cache_manager().cache_exists(repo_path, commit_id) {
            // 获取提交信息
            let commit_info = match git_engine.get_commit_info(repo_path, commit_id) {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Failed to get commit info for {}: {}", commit_id, e);
                    failed_analyses += 1;
                    continue;
                }
            };
            
            // 获取文件差异信息
            let diff_info = match git_engine.get_commit_diff(repo_path, commit_id) {
                Ok(diff) => diff,
                Err(e) => {
                    eprintln!("Failed to get diff info for {}: {}", commit_id, e);
                    failed_analyses += 1;
                    continue;
                }
            };
            
            // 分析提交
            match analysis_engine.analyze_commit(repo_path, commit_id, &commit_info, Some(&diff_info)).await {
                Ok(_) => {
                    successful_analyses += 1;
                    println!("成功分析提交: {}", commit_id);
                },
                Err(e) => {
                    eprintln!("Failed to analyze commit {}: {}", commit_id, e);
                    failed_analyses += 1;
                }
            }
        } else {
            println!("提交 {} 已存在于缓存中", commit_id);
            successful_analyses += 1;
        }
    }
    
    // 最终进度
    let final_progress = AnalysisProgress {
        session_id,
        total_steps,
        current_step: total_steps,
        current_status: format!("分析完成。成功: {}, 失败: {}", successful_analyses, failed_analyses),
        progress_percentage: 100,
        current_file: None,
    };
    
    app_handle.emit("analysis-progress", &final_progress)
        .map_err(|e| format!("Failed to emit final progress: {}", e))?;
    
    Ok(final_progress)
}

/// 生成AI增强日报
#[tauri::command]
pub async fn generate_ai_enhanced_report(
    config: AnalysisConfig,
    app_handle: tauri::AppHandle,
    include_tech_analysis: Option<bool>,
    include_risk_assessment: Option<bool>,
    use_ai_summary: Option<bool>,
    report_template: Option<String>,
) -> Result<Report, String> {
    println!("开始生成AI增强日报...");
    println!("配置: 仓库数={}, 用户数={}, 日期范围={} 到 {}", 
        config.repoPaths.len(), config.userEmails.len(), config.startDate, config.endDate);
    
    let report_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    
    // 获取应用数据目录
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    // 创建分析引擎
    let mut analysis_engine = AnalysisEngine::new(&app_dir)
        .map_err(|e| format!("Failed to create analysis engine: {}", e))?;
    
    // 获取AI管理器
    let ai_manager = app_handle.state::<Arc<RwLock<AIManager>>>();
    analysis_engine = analysis_engine.with_ai_manager(ai_manager.inner().clone());
    
    // 获取Git引擎
    let git_engine_state = app_handle.state::<Mutex<GitEngine>>();
    let git_engine = git_engine_state.lock().await;
    
    // 收集所有提交分析
    let mut all_analyses = Vec::new();
    let _total_commits_found = 0;
    
    // 分析每个仓库的提交
    for repo_path in &config.repoPaths {
        println!("处理仓库: {}", repo_path);
        
        // 获取提交列表
        let commits = git_engine.get_commits_in_date_range(
            repo_path, 
            &config.startDate, 
            &config.endDate
        ).map_err(|e| format!("Failed to get commits: {}", e))?;
        
        let _total_commits_found = commits.len();
        
        // 过滤用户
        let filtered_commits: Vec<_> = if config.userEmails.is_empty() {
            commits
        } else {
            commits.into_iter()
                .filter(|c| config.userEmails.contains(&c.email))
                .collect()
        };
        
        println!("仓库 {} 过滤后提交数: {}", repo_path, filtered_commits.len());
        
        // 分析每个提交
        for commit in filtered_commits {
            // 检查缓存
            if let Ok(Some(cached_analysis)) = analysis_engine.cache_manager()
                .load_commit_analysis(repo_path, &commit.hash) {
                all_analyses.push(cached_analysis);
            } else {
                // 获取差异信息并分析
                match git_engine.get_commit_diff(repo_path, &commit.hash) {
                    Ok(diff_info) => {
                        match analysis_engine.analyze_commit(
                            repo_path,
                            &commit.hash,
                            &commit,
                            Some(&diff_info)
                        ).await {
                            Ok(analysis) => {
                                all_analyses.push(analysis);
                            },
                            Err(e) => {
                                eprintln!("分析提交失败 {}: {}", commit.hash, e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("获取差异失败 {}: {}", commit.hash, e);
                    }
                }
            }
        }
    }
    
    // 生成报告内容
    let title = format!("AI增强日报 - {} 至 {}", config.startDate, config.endDate);
    let content = if use_ai_summary.unwrap_or(true) && !all_analyses.is_empty() {
        // 使用AI生成汇总
        let template_id = report_template.unwrap_or_else(|| "daily_summary_optimized".to_string());
        match analysis_engine.generate_ai_summary_report_with_template(
            &all_analyses,
            &config.repoPaths,
            &config.startDate,
            &config.endDate,
            &config.userEmails,
            include_tech_analysis.unwrap_or(true),
            include_risk_assessment.unwrap_or(true),
            &template_id,
        ).await {
            Ok(ai_summary) => ai_summary,
            Err(e) => {
                eprintln!("AI汇总生成失败: {}", e);
                // 回退到基础报告生成
                generate_basic_report_content(&all_analyses, &config)?
            }
        }
    } else {
        // 生成基础报告
        generate_basic_report_content(&all_analyses, &config)?
    };
    
    Ok(Report {
        id: report_id,
        title,
        content,
        format: "markdown".to_string(),
        created_at,
        config,
    })
}

/// 生成基础报告内容（作为AI报告的回退方案）
fn generate_basic_report_content(
    analyses: &[CommitDetailAnalysis],
    config: &AnalysisConfig
) -> Result<String, String> {
    let mut content = String::new();
    
    content.push_str(&format!("# 开发日报\n\n"));
    content.push_str(&format!("**报告周期**: {} 至 {}\n\n", config.startDate, config.endDate));
    content.push_str(&format!("**生成时间**: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    content.push_str(&format!("**总提交数**: {}\n\n", analyses.len()));
    
    if analyses.is_empty() {
        content.push_str("*在选定的时间范围内没有找到提交记录。*\n");
        return Ok(content);
    }
    
    // 统计信息
    let total_insertions: u32 = analyses.iter().map(|a| a.insertions).sum();
    let total_deletions: u32 = analyses.iter().map(|a| a.deletions).sum();
    
    content.push_str("## 📊 统计概览\n\n");
    content.push_str(&format!("- **活跃仓库数**: {}\n", config.repoPaths.len()));
    content.push_str(&format!("- **代码变更**: +{} / -{}\n\n", total_insertions, total_deletions));
    
    // 重要提交
    let important_commits: Vec<_> = analyses.iter()
        .filter(|a| a.impact_level == crate::types::git_types::ImpactLevel::Critical || 
                a.impact_level == crate::types::git_types::ImpactLevel::High)
        .collect();
    
    if !important_commits.is_empty() {
        content.push_str("## 🎯 重要提交\n\n");
        for analysis in important_commits.iter().take(5) {
            content.push_str(&format!(
                "### {} - {}\n", 
                &analysis.commit_id[..7],
                analysis.message.lines().next().unwrap_or("")
            ));
            content.push_str(&format!("**作者**: {} ({})\n", analysis.author, analysis.email));
            content.push_str(&format!("**摘要**: {}\n\n", analysis.summary));
        }
    }
    
    content.push_str("---\n*报告由 GitMentor 自动生成*\n");
    
    Ok(content)
}

/// 获取AI分析模板列表
#[tauri::command]
pub async fn get_ai_analysis_templates() -> Result<Vec<serde_json::Value>, String> {
    let prompt_manager = PromptTemplateManager::new();
    let templates = prompt_manager.get_all_templates();
    
    let mut result = Vec::new();
    for template in templates {
        let template_json = serde_json::json!({
            "id": template.id,
            "name": template.name,
            "description": template.description,
            "template_type": template.template_type,
            "template_content": template.template_content,
            "variables": template.variables,
            "version": template.version
        });
        result.push(template_json);
    }
    
    Ok(result)
}

/// 获取AI分析配置
#[tauri::command]
pub async fn get_ai_analysis_config() -> Result<AIAnalysisConfig, String> {
    Ok(AIAnalysisConfig::default())
}

/// 设置AI分析配置
#[tauri::command]
pub async fn set_ai_analysis_config(
    config: AIAnalysisConfig,
) -> Result<bool, String> {
    // TODO: 保存配置到本地
    println!("更新AI分析配置: {:?}", config);
    Ok(true)
}

/// 清理分析缓存
#[tauri::command]
pub async fn clear_analysis_cache(
    repo_path: Option<String>,
    commit_id: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<bool, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    let cache_manager = crate::core::report_engine::CacheManager::new(&app_dir);
    
    match (repo_path, commit_id) {
        (Some(repo), Some(commit)) => {
            // 清除特定提交的缓存
            let cache_path = cache_manager.get_cache_path(&repo, &commit);
            if cache_path.exists() {
                std::fs::remove_file(&cache_path)
                    .map_err(|e| format!("Failed to remove cache file: {}", e))?;
            }
        },
        (Some(repo), None) => {
            // 清除指定仓库的所有缓存
            let repo_hash = cache_manager.hash_path(&repo);
            let pattern = format!("commit_{}_", repo_hash);
            
            if let Ok(entries) = std::fs::read_dir(&cache_manager.cache_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if entry.file_name().to_string_lossy().starts_with(&pattern) {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        },
        (None, None) => {
            // 清除所有缓存
            if let Ok(entries) = std::fs::read_dir(&cache_manager.cache_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        },
        _ => {
            return Err("无效的参数组合".to_string());
        }
    }
    
    Ok(true)
}

/// 获取所有AI分析模板
#[tauri::command]
pub async fn get_ai_templates(app_handle: tauri::AppHandle) -> Result<Vec<PromptTemplate>, String> {
    use crate::core::versioned_template_manager::*;
    use crate::types::git_types::{AIAnalysisTemplate, AnalysisDepth};
    use std::sync::Mutex;
    use once_cell::sync::Lazy;

    // 获取或初始化版本化模板管理器
    static TEMPLATE_MANAGER: Lazy<Mutex<Option<VersionedTemplateManager>>> = Lazy::new(|| Mutex::new(None));

    let mut manager = TEMPLATE_MANAGER.lock().unwrap();
    if manager.is_none() {
        let app_dir = app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        *manager = Some(VersionedTemplateManager::new(&app_dir)
            .map_err(|e| format!("Failed to create template manager: {}", e))?);
    }

    let manager = manager.as_ref().unwrap();

    // 获取所有模板并转换为 PromptTemplate 格式
    let versioned_templates = manager.get_all_templates();
    let mut templates = Vec::new();

    for template in versioned_templates {
        // 确定模板类型
        let template_type = match template.template_type.as_str() {
            "commit_analysis" => AIAnalysisTemplate::CommitAnalysis {
                depth: AnalysisDepth::Detailed,
                include_code_review: false,
            },
            "daily_summary" => AIAnalysisTemplate::DailySummary {
                include_tech_analysis: false,
                include_risk_assessment: false,
            },
            _ => AIAnalysisTemplate::DailySummary {
                include_tech_analysis: false,
                include_risk_assessment: false,
            }, // 默认类型
        };

        // 获取当前内容
        let content = manager.get_template_content(&template.id)
            .unwrap_or_else(|_| "".to_string());

        // 创建 PromptTemplate
        let prompt_template = PromptTemplate {
            id: template.id.clone(),
            name: template.name.clone(),
            description: template.description.clone(),
            template_type,
            template_content: content,
            variables: vec![], // AI分析模板通常没有变量
            version: template.current_version_id.clone(),
        };

        templates.push(prompt_template);
    }

    Ok(templates)
}

/// 更新AI分析模板
#[tauri::command]
pub async fn update_ai_template(
    templateId: String,
    templateContent: String,
    app_handle: tauri::AppHandle,
) -> Result<bool, String> {
    use crate::core::versioned_template_manager::*;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;

    // 获取或初始化版本化模板管理器
    static TEMPLATE_MANAGER: Lazy<Mutex<Option<VersionedTemplateManager>>> = Lazy::new(|| Mutex::new(None));

    let mut manager = TEMPLATE_MANAGER.lock().unwrap();
    if manager.is_none() {
        let app_dir = app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        *manager = Some(VersionedTemplateManager::new(&app_dir)
            .map_err(|e| format!("Failed to create template manager: {}", e))?);
    }

    let manager = manager.as_mut().unwrap();

    // 更新模板内容并创建新版本
    manager.update_template(
        &templateId,
        templateContent,
        format!("v{}", chrono::Utc::now().timestamp()), // 版本号
        "用户编辑的版本".to_string(), // 版本描述
    ).map_err(|e| format!("Failed to update template: {}", e))?;

    Ok(true)
}

/// 重置AI分析模板为默认
#[tauri::command]
pub async fn reset_ai_template(
    templateId: String,
    app_handle: tauri::AppHandle,
) -> Result<bool, String> {
    use crate::core::versioned_template_manager::*;
    use std::sync::Mutex;
    use once_cell::sync::Lazy;

    // 获取或初始化版本化模板管理器
    static TEMPLATE_MANAGER: Lazy<Mutex<Option<VersionedTemplateManager>>> = Lazy::new(|| Mutex::new(None));

    let mut manager = TEMPLATE_MANAGER.lock().unwrap();
    if manager.is_none() {
        let app_dir = app_handle.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;
        *manager = Some(VersionedTemplateManager::new(&app_dir)
            .map_err(|e| format!("Failed to create template manager: {}", e))?);
    }

    let manager = manager.as_mut().unwrap();

    // 还原到系统内置版本
    manager.revert_to_builtin_version(&templateId)
        .map_err(|e| format!("Failed to reset template: {}", e))?;

    Ok(true)
}