/**
 * Code Review 存储模块
 * 负责审查记录的持久化存储和管理
 * 作者：Evilek
 * 日期：2025-01-04
 */

use anyhow::{Result, Context, anyhow};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::codereview::{
    ReviewRecord, ReviewFilters, PaginationOptions, PaginatedResult,
    ReviewConfig, ReviewError, ReviewErrorInfo
};

/// 审查存储管理器
pub struct ReviewStorage {
    config_dir: PathBuf,
    data_path: PathBuf,
    cache_dir: PathBuf,
    config_path: PathBuf,
}

/// 存储缓存结构
#[derive(Debug, Clone)]
struct StorageCache {
    reviews: Vec<ReviewRecord>,
    index: ReviewIndex,
}

/// 审查索引结构
#[derive(Debug, Clone, Default)]
struct ReviewIndex {
    reviews_by_id: HashMap<String, usize>,
    reviews_by_date: HashMap<String, Vec<String>>,
    reviews_by_type: HashMap<String, Vec<String>>,
    reviews_by_status: HashMap<String, Vec<String>>,
    reviews_by_file: HashMap<String, Vec<String>>,
    reviews_by_commit: HashMap<String, String>,
}

/// 默认审查配置
impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            ai: crate::types::codereview::AIConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                default_depth: crate::types::codereview::ReviewDepth::Standard,
                timeout: 30,
                retries: 3,
                streaming: true,
                max_tokens: 4000,
                temperature: 0.7,
                custom_prompts: HashMap::new(),
            },
        }
    }
}

impl ReviewStorage {
    /// 创建新的存储管理器
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        // 确保目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("创建配置目录失败")?;
        }

        let data_path = config_dir.join("code_review_history.json");
        let cache_dir = config_dir.join("review_cache");
        let config_path = config_dir.join("code_review_config.json");

        // 创建缓存目录
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .context("创建缓存目录失败")?;
        }

        Ok(Self {
            config_dir,
            data_path,
            cache_dir,
            config_path,
        })
    }

    /// 获取存储的根目录路径
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// 获取数据文件路径
    pub fn get_data_path(&self) -> &PathBuf {
        &self.data_path
    }

    /// 获取缓存目录路径
    pub fn get_cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// 获取配置文件路径
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// 保存审查记录
    pub async fn save_review(&self, review: &ReviewRecord) -> Result<()> {
        // 读取现有记录
        let mut reviews = self.load_all_reviews().await?;

        // 查找是否已存在
        if let Some(existing_idx) = reviews.iter().position(|r| r.id == review.id) {
            // 更新现有记录
            reviews[existing_idx] = review.clone();
        } else {
            // 添加新记录
            reviews.push(review.clone());
        }

        // 写入主文件
        let json = serde_json::to_string_pretty(&reviews)
            .context("序列化审查记录失败")?;
        fs::write(&self.data_path, json)
            .context("写入审查记录失败")?;

        // 写入缓存文件
        self.write_cache_file(review).await?;

        Ok(())
    }

    /// 读取单个审查记录
    pub async fn load_review(&self, id: &str) -> Result<Option<ReviewRecord>> {
        // 先尝试从缓存读取
        if let Some(review) = self.load_review_from_cache(id).await? {
            return Ok(Some(review));
        }

        // 从主文件读取
        let reviews = self.load_all_reviews().await?;
        Ok(reviews.into_iter().find(|r| r.id == id))
    }

    /// 加载所有审查记录
    pub async fn load_all_reviews(&self) -> Result<Vec<ReviewRecord>> {
        if !self.data_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.data_path)
            .context("读取审查记录文件失败")?;

        let reviews: Vec<ReviewRecord> = serde_json::from_str(&content)
            .context("解析审查记录失败")?;

        Ok(reviews)
    }

    /// 根据筛选条件搜索审查记录
    pub async fn search_reviews(
        &self,
        filters: &ReviewFilters,
        pagination: &PaginationOptions,
    ) -> Result<PaginatedResult<ReviewRecord>> {
        let reviews = self.load_all_reviews().await?;

        // 应用筛选条件
        let filtered: Vec<ReviewRecord> = reviews
            .into_iter()
            .filter(|r| self.matches_filters(r, filters))
            .collect();

        // 排序
        let mut sorted = filtered;
        if let (Some(sort_by), Some(sort_order)) = (&pagination.sort_by, &pagination.sort_order) {
            sorted.sort_by(|a, b| {
                let comparison = match sort_by.as_str() {
                    "timestamp" => a.timestamp.cmp(&b.timestamp),
                    "duration" => a.duration.cmp(&b.duration),
                    _ => a.timestamp.cmp(&b.timestamp),
                };
                match sort_order.as_str() {
                    "desc" => comparison.reverse(),
                    _ => comparison,
                }
            });
        }

        // 分页
        let total = sorted.len() as u32;
        let start = ((pagination.page - 1) * pagination.page_size) as usize;
        let end = (start + pagination.page_size as usize).min(sorted.len());
        let page_data = if start < sorted.len() {
            sorted[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(PaginatedResult {
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            data: page_data,
        })
    }

    /// 删除审查记录
    pub async fn delete_review(&self, id: &str) -> Result<()> {
        // 读取现有记录
        let mut reviews = self.load_all_reviews().await?;

        // 查找并移除
        let original_len = reviews.len();
        reviews.retain(|r| r.id != id);

        if reviews.len() == original_len {
            return Err(anyhow!("未找到审查记录: {}", id));
        }

        // 写入主文件
        let json = serde_json::to_string_pretty(&reviews)
            .context("序列化审查记录失败")?;
        fs::write(&self.data_path, json)
            .context("写入审查记录失败")?;

        // 删除缓存文件
        let cache_path = self.cache_dir.join(format!("{}.json", id));
        if cache_path.exists() {
            fs::remove_file(cache_path)
                .context("删除缓存文件失败")?;
        }

        Ok(())
    }

    /// 清除所有审查记录
    pub async fn clear_all_reviews(&self) -> Result<()> {
        // 清空主文件
        fs::write(&self.data_path, "[]")
            .context("清空审查记录文件失败")?;

        // 删除缓存目录
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .context("删除缓存目录失败")?;
            fs::create_dir_all(&self.cache_dir)
                .context("重新创建缓存目录失败")?;
        }

        Ok(())
    }

    /// 获取缓存大小
    pub async fn get_cache_size(&self) -> Result<u64> {
        let mut total_size = 0u64;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)
                .context("读取缓存目录失败")?
            {
                let entry = entry
                    .context("读取缓存项失败")?;
                let metadata = entry.metadata()
                    .context("获取文件元数据失败")?;
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    /// 清除缓存
    pub async fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .context("删除缓存目录失败")?;
            fs::create_dir_all(&self.cache_dir)
                .context("重新创建缓存目录失败")?;
        }

        Ok(())
    }

    /// 获取默认配置
    pub async fn get_default_config(&self) -> ReviewConfig {
        ReviewConfig::default()
    }

    /// 检查筛选条件是否匹配
    fn matches_filters(&self, review: &ReviewRecord, filters: &ReviewFilters) -> bool {
        // 日期范围筛选
        if let Some(date_from) = &filters.date_from {
            if review.timestamp < date_from {
                return false;
            }
        }

        if let Some(date_to) = &filters.date_to {
            if review.timestamp > date_to {
                return false;
            }
        }

        // 审查类型筛选
        if let Some(review_types) = &filters.review_types {
            if !review_types.contains(&review.review_type) {
                return false;
            }
        }

        // 审查状态筛选
        if let Some(statuses) = &filters.statuses {
            if !statuses.contains(&review.status) {
                return false;
            }
        }

        // 文件筛选
        if let Some(files) = &filters.files {
            let has_file = review.files.iter().any(|f| files.contains(&f.path));
            if !has_file {
                return false;
            }
        }

        // 关键词搜索
        if let Some(query) = &filters.query {
            let query_lower = query.to_lowercase();
            let matches = review.summary.as_ref()
                .map(|s| s.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
                || review.files.iter().any(|f| f.path.to_lowercase().contains(&query_lower));

            if !matches {
                return false;
            }
        }

        true
    }

    /// 从缓存加载审查记录
    async fn load_review_from_cache(&self, id: &str) -> Result<Option<ReviewRecord>> {
        let cache_path = self.cache_dir.join(format!("{}.json", id));

        if !cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_path)
            .context("读取缓存文件失败")?;

        let review: ReviewRecord = serde_json::from_str(&content)
            .context("解析缓存文件失败")?;

        Ok(Some(review))
    }

    /// 写入缓存文件
    async fn write_cache_file(&self, review: &ReviewRecord) -> Result<()> {
        let cache_path = self.cache_dir.join(format!("{}.json", review.id));

        let json = serde_json::to_string_pretty(review)
            .context("序列化审查记录失败")?;

        fs::write(cache_path, json)
            .context("写入缓存文件失败")?;

        Ok(())
    }
}

/// 全局存储管理器实例
pub type GlobalReviewStorage = Arc<RwLock<ReviewStorage>>;

/// 创建全局存储管理器
pub async fn create_global_review_storage(config_dir: PathBuf) -> Result<GlobalReviewStorage> {
    let storage = ReviewStorage::new(config_dir)?;
    Ok(Arc::new(RwLock::new(storage)))
}
