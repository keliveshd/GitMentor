/**
 * Code Review Tauri 命令接口
 * 提供前端调用的 API
 * 作者：Evilek
 * 日期：2025-01-04
 */

use anyhow::{Result, Context, anyhow};
use tauri::State;
use std::sync::Arc;

use crate::core::codereview::{
    CodeReviewEngine, GlobalReviewStorage,
    ReviewRecord, ReviewFilters, PaginationOptions, PaginatedResult,
    ReviewConfig, ReviewDepth
};

/// 创建 AI 代码审查
#[tauri::command]
pub async fn create_ai_review(
    files: Vec<String>,
    depth: ReviewDepth,
    config: Option<ReviewConfig>,
    current_branch: Option<String>,
    creator: Option<String>,
    review_engine: State<'_, CodeReviewEngine>,
) -> Result<ReviewRecord, String> {
    let config = config.unwrap_or(ReviewConfig::default());
    let branch = current_branch.unwrap_or_else(|| "unknown".to_string());
    let creator = creator.unwrap_or_else(|| "anonymous".to_string());

    match review_engine
        .analyze_with_ai(files, depth, config, branch, creator)
        .await
    {
        Ok(record) => Ok(record),
        Err(e) => {
            eprintln!("[ERROR] AI 审查失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 获取审查记录
#[tauri::command]
pub async fn get_review_record(
    id: String,
    review_engine: State<'_, CodeReviewEngine>,
) -> Result<Option<ReviewRecord>, String> {
    match review_engine.get_review_record(&id).await {
        Ok(record) => Ok(record),
        Err(e) => {
            eprintln!("[ERROR]获取审查记录失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 获取审查历史列表
#[tauri::command]
pub async fn get_review_history(
    filters: Option<ReviewFilters>,
    pagination: Option<PaginationOptions>,
    review_engine: State<'_, CodeReviewEngine>,
) -> Result<PaginatedResult<ReviewRecord>, String> {
    let filters = filters.unwrap_or_default();
    let pagination = pagination.unwrap_or(PaginationOptions {
        page: 1,
        page_size: 20,
        sort_by: Some("timestamp".to_string()),
        sort_order: Some("desc".to_string()),
    });

    match review_engine
        .get_review_history(filters, pagination)
        .await
    {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("[ERROR]获取审查历史失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 删除审查记录
#[tauri::command]
pub async fn delete_review_record(
    id: String,
    review_engine: State<'_, CodeReviewEngine>,
) -> Result<(), String> {
    match review_engine.delete_review(&id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("[ERROR]删除审查记录失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 清除所有审查记录
#[tauri::command]
pub async fn clear_all_reviews(
    review_engine: State<'_, CodeReviewEngine>,
) -> Result<(), String> {
    match review_engine.clear_all_reviews().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("[ERROR]清除审查记录失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 获取审查统计数据
#[tauri::command]
pub async fn get_review_statistics(
    review_engine: State<'_, CodeReviewEngine>,
) -> Result<crate::core::codereview::ReviewStatistics, String> {
    match review_engine.get_review_statistics().await {
        Ok(stats) => Ok(stats),
        Err(e) => {
            eprintln!("[ERROR]获取审查统计失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 获取默认审查配置
#[tauri::command]
pub async fn get_review_config(
    storage: State<'_, GlobalReviewStorage>,
) -> Result<ReviewConfig, String> {
    let storage_guard = storage.read().await;
    Ok(storage_guard.get_default_config().await)
}

/// 获取缓存大小
#[tauri::command]
pub async fn get_review_cache_size(
    storage: State<'_, GlobalReviewStorage>,
) -> Result<u64, String> {
    let storage_guard = storage.read().await;
    match storage_guard.get_cache_size().await {
        Ok(size) => Ok(size),
        Err(e) => {
            eprintln!("[ERROR]获取缓存大小失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 清除审查缓存
#[tauri::command]
pub async fn clear_review_cache(
    storage: State<'_, GlobalReviewStorage>,
) -> Result<(), String> {
    let storage_guard = storage.read().await;
    match storage_guard.clear_cache().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("[ERROR]清除缓存失败: {:?}", e);
            Err(e.to_string())
        }
    }
}

/// 获取存储路径信息
#[tauri::command]
pub async fn get_review_storage_paths(
    storage: State<'_, GlobalReviewStorage>,
) -> Result<ReviewStoragePaths, String> {
    let storage_guard = storage.read().await;
    Ok(ReviewStoragePaths {
        config_dir: storage_guard.get_config_dir().to_string_lossy().to_string(),
        data_path: storage_guard.get_data_path().to_string_lossy().to_string(),
        cache_dir: storage_guard.get_cache_dir().to_string_lossy().to_string(),
    })
}

/// 存储路径信息
#[derive(serde::Serialize)]
pub struct ReviewStoragePaths {
    pub config_dir: String,
    pub data_path: String,
    pub cache_dir: String,
}

/// 初始化 Code Review 存储
pub async fn init_review_storage(
    config_dir: std::path::PathBuf,
) -> Result<GlobalReviewStorage> {
    println!("[INIT] 初始化 Code Review 存储: {:?}", config_dir);
    crate::core::codereview::create_global_review_storage(config_dir)
        .await
        .context("初始化 Code Review 存储失败")
}

/// 初始化 Code Review 引擎
pub fn init_review_engine(
    ai_manager: Arc<tokio::sync::RwLock<crate::core::ai_manager::AIManager>>,
    storage: GlobalReviewStorage,
) -> CodeReviewEngine {
    println!("[INIT] 初始化 Code Review 引擎");
    CodeReviewEngine::new(ai_manager, storage)
}
