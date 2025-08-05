use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::core::ai_provider::{AIRequest, AIResponse};

/**
 * 对话记录管理器
 * 作者：Evilek
 * 编写日期：2025-01-30
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub template_id: String,
    pub repository_path: Option<String>, // 新增：仓库路径标识
    pub request: AIRequest,
    pub response: Option<AIResponse>,
    pub processing_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug)]
pub struct ConversationLogger {
    log_file_path: PathBuf,
    records: Vec<ConversationRecord>,
}

impl ConversationLogger {
    pub fn new(log_file_path: PathBuf) -> Result<Self> {
        let records = if log_file_path.exists() {
            let content = fs::read_to_string(&log_file_path)?;
            if content.trim().is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&content).unwrap_or_else(|e| {
                    eprintln!("警告: 无法解析对话记录文件: {}", e);
                    Vec::new()
                })
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            log_file_path,
            records,
        })
    }

    /// 记录成功的对话
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub fn log_success(
        &mut self,
        template_id: String,
        repository_path: Option<String>,
        request: AIRequest,
        response: AIResponse,
        processing_time_ms: u64,
    ) -> Result<()> {
        let record = ConversationRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            template_id,
            repository_path,
            request,
            response: Some(response),
            processing_time_ms,
            success: true,
            error_message: None,
        };

        self.records.push(record);
        self.save_to_file()?;
        Ok(())
    }

    /// 记录失败的对话
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub fn log_failure(
        &mut self,
        template_id: String,
        repository_path: Option<String>,
        request: AIRequest,
        error_message: String,
        processing_time_ms: u64,
    ) -> Result<()> {
        let record = ConversationRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            template_id,
            repository_path,
            request,
            response: None,
            processing_time_ms,
            success: false,
            error_message: Some(error_message),
        };

        self.records.push(record);
        self.save_to_file()?;
        Ok(())
    }

    /// 获取所有对话记录
    pub fn get_all_records(&self) -> &Vec<ConversationRecord> {
        &self.records
    }

    /// 根据仓库路径过滤对话记录
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub fn get_records_by_repository(
        &self,
        repository_path: Option<&str>,
    ) -> Vec<&ConversationRecord> {
        self.records
            .iter()
            .filter(|record| {
                match (&record.repository_path, repository_path) {
                    (Some(record_path), Some(filter_path)) => record_path == filter_path,
                    (None, None) => true, // 都为None时匹配
                    _ => false,
                }
            })
            .collect()
    }

    /// 获取所有不同的仓库路径
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub fn get_repository_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .records
            .iter()
            .filter_map(|record| record.repository_path.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        paths.sort();
        paths
    }

    /// 获取最近的N条记录
    pub fn get_recent_records(&self, limit: usize) -> Vec<&ConversationRecord> {
        let mut sorted_records: Vec<&ConversationRecord> = self.records.iter().collect();
        sorted_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted_records.into_iter().take(limit).collect()
    }

    /// 清空所有记录
    pub fn clear_all_records(&mut self) -> Result<()> {
        self.records.clear();
        self.save_to_file()?;
        Ok(())
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> ConversationStatistics {
        let total_count = self.records.len();
        let success_count = self.records.iter().filter(|r| r.success).count();
        let failure_count = total_count - success_count;

        let total_time: u64 = self.records.iter().map(|r| r.processing_time_ms).sum();
        let average_time = if total_count > 0 {
            total_time / total_count as u64
        } else {
            0
        };

        ConversationStatistics {
            total_count,
            success_count,
            failure_count,
            average_processing_time_ms: average_time,
        }
    }

    /// 保存到文件
    fn save_to_file(&self) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.log_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 限制记录数量，只保留最近的1000条记录
        let records_to_save = if self.records.len() > 1000 {
            let mut sorted_records = self.records.clone();
            sorted_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            sorted_records.into_iter().take(1000).collect()
        } else {
            self.records.clone()
        };

        let content = serde_json::to_string_pretty(&records_to_save)?;
        fs::write(&self.log_file_path, content)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationStatistics {
    pub total_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub average_processing_time_ms: u64,
}
