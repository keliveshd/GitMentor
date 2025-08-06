use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::ai_manager::AIManager;
use crate::core::git_engine::GitEngine;
use crate::core::conversation_logger::StepInfo;
use crate::core::ai_provider::{AIRequest, ChatMessage};
use crate::utils::token_counter::TokenCounter;

/**
 * 分层提交管理器
 * 作者：Evilek
 * 编写日期：2025-08-04
 */

#[derive(Debug, Clone)]
pub struct LayeredCommitProgress {
    pub session_id: String,
    pub current_step: u32,
    pub total_steps: u32,
    pub current_file: Option<String>,
    pub status: String,
    pub file_summaries: Vec<FileSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    pub file_path: String,
    pub summary: String,
    pub tokens_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredCommitResult {
    pub session_id: String,
    pub final_message: String,
    pub file_summaries: Vec<FileSummary>,
    pub total_processing_time_ms: u64,
    pub conversation_records: Vec<String>, // 记录ID列表
}

pub struct LayeredCommitManager {
    ai_manager: Arc<RwLock<AIManager>>,
    git_engine: Arc<RwLock<GitEngine>>,
}

impl LayeredCommitManager {
    pub fn new(
        ai_manager: Arc<RwLock<AIManager>>,
        git_engine: Arc<RwLock<GitEngine>>,
    ) -> Self {
        Self {
            ai_manager,
            git_engine,
        }
    }

    /// 检查是否需要启用分层提交
    pub async fn should_use_layered_commit(
        &self,
        _template_id: &str,
        diff_content: &str,
        _staged_files: &[String],
    ) -> Result<bool> {
        let ai_manager = self.ai_manager.read().await;
        let config = ai_manager.get_config().await;
        
        // 检查是否启用了分层提交功能
        if !config.features.enable_layered_commit {
            return Ok(false);
        }

        // 获取当前模型的token限制
        let model_max_tokens = self.get_model_max_tokens(&config.base.model).await?;
        
        // 构建基本的消息来估算token数量
        let system_message = "你是一个专业的Git提交消息生成助手。";
        let user_message = format!("请为以下代码变更生成提交消息:\n{}", diff_content);
        
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_message.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_message,
            },
        ];
        
        let request = AIRequest {
            messages,
            model: config.base.model.clone(),
            temperature: Some(0.3),
            max_tokens: Some(500),
            stream: Some(false),
        };

        let estimated_tokens = TokenCounter::estimate_request_tokens(&request);
        let is_over_limit = TokenCounter::is_over_limit(estimated_tokens, model_max_tokens);

        Ok(is_over_limit)
    }

    /// 执行分层提交
    pub async fn execute_layered_commit<F>(
        &self,
        template_id: &str,
        staged_files: Vec<String>,
        branch_name: Option<String>,
        repository_path: Option<String>,
        progress_callback: F,
    ) -> Result<LayeredCommitResult>
    where
        F: Fn(LayeredCommitProgress) + Send + Sync,
    {
        let session_id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        // 第一步：获取每个文件的diff
        let files_with_diffs = self.get_files_with_diffs(&staged_files).await?;
        let total_files = files_with_diffs.len();

        // 初始化进度
        let mut progress = LayeredCommitProgress {
            session_id: session_id.clone(),
            current_step: 0,
            total_steps: (total_files + 1) as u32, // 文件分析 + 最终总结
            current_file: None,
            status: "开始分层提交".to_string(),
            file_summaries: Vec::new(),
        };
        progress_callback(progress.clone());

        // 第二步：为每个文件生成摘要
        let mut file_summaries = Vec::new();
        let mut conversation_records = Vec::new();

        for (index, (file_path, diff_content)) in files_with_diffs.iter().enumerate() {
            progress.current_step = (index + 1) as u32;
            progress.current_file = Some(file_path.clone());
            progress.status = format!("分析文件 {}/{}: {}", index + 1, total_files, file_path);
            progress_callback(progress.clone());

            let summary_result = self.analyze_single_file(
                file_path,
                diff_content,
                &session_id,
                index as u32 + 1,
                total_files as u32,
                repository_path.clone(),
            ).await?;

            file_summaries.push(summary_result.summary.clone());
            conversation_records.push(summary_result.record_id);
            progress.file_summaries.push(summary_result.summary);
        }

        // 第三步：生成最终的提交消息
        progress.current_step = total_files as u32 + 1;
        progress.current_file = None;
        progress.status = "生成最终提交消息".to_string();
        progress_callback(progress.clone());

        let final_result = self.generate_final_commit_message(
            template_id,
            &file_summaries,
            branch_name,
            &session_id,
            repository_path.clone(),
        ).await?;

        conversation_records.push(final_result.record_id);

        let total_time = start_time.elapsed().as_millis() as u64;

        Ok(LayeredCommitResult {
            session_id,
            final_message: final_result.message,
            file_summaries,
            total_processing_time_ms: total_time,
            conversation_records,
        })
    }

    /// 获取文件及其diff内容
    async fn get_files_with_diffs(&self, staged_files: &[String]) -> Result<Vec<(String, String)>> {
        let git_engine = self.git_engine.read().await;
        let mut files_with_diffs = Vec::new();

        for file_path in staged_files {
            match git_engine.get_simple_file_diff(file_path) {
                Ok(diff_content) => {
                    files_with_diffs.push((file_path.clone(), diff_content));
                },
                Err(e) => {
                    return Err(anyhow::anyhow!("获取文件diff失败 {}: {}", file_path, e));
                }
            }
        }

        Ok(files_with_diffs)
    }

    /// 分析单个文件
    async fn analyze_single_file(
        &self,
        file_path: &str,
        diff_content: &str,
        session_id: &str,
        step_index: u32,
        total_steps: u32,
        repository_path: Option<String>,
    ) -> Result<SingleFileResult> {
        let ai_manager = self.ai_manager.read().await;
        let config = ai_manager.get_config().await;

        // 构建单文件分析的提示词
        let system_prompt = "你是一个专业的代码审查助手。请分析给定文件的变更内容，生成简洁的变更摘要（50-100字）。";
        let user_prompt = format!(
            "文件路径: {}\n\n变更内容:\n{}\n\n请生成这个文件变更的简洁摘要：",
            file_path, diff_content
        );

        let request = AIRequest {
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            model: config.base.model.clone(),
            temperature: Some(0.3),
            max_tokens: Some(200),
            stream: Some(false),
        };

        let start_time = std::time::Instant::now();
        let response = ai_manager.generate_commit_message(request.clone()).await?;
        let processing_time = start_time.elapsed().as_millis() as u64;

        // 记录对话到日志
        let step_info = StepInfo {
            step_type: "file_analysis".to_string(),
            step_index: Some(step_index),
            total_steps: Some(total_steps),
            file_path: Some(file_path.to_string()),
            description: Some(format!("分析文件: {}", file_path)),
        };

        // 记录对话到日志
        let record_id = ai_manager.log_conversation_with_session(
            "layered_commit".to_string(),
            repository_path,
            Some(session_id.to_string()),
            Some("layered".to_string()),
            Some(step_info),
            request,
            response.clone(),
            processing_time,
        ).await?;

        let tokens_used = TokenCounter::estimate_tokens(&response.content);

        Ok(SingleFileResult {
            summary: FileSummary {
                file_path: file_path.to_string(),
                summary: response.content,
                tokens_used,
            },
            record_id,
        })
    }

    /// 生成最终提交消息
    /// 作者：Evilek
    /// 编写日期：2025-08-05
    async fn generate_final_commit_message(
        &self,
        template_id: &str,
        file_summaries: &[FileSummary],
        _branch_name: Option<String>,
        session_id: &str,
        repository_path: Option<String>,
    ) -> Result<FinalCommitResult> {
        let ai_manager = self.ai_manager.read().await;

        // 构建汇总的diff内容
        let summary_content = file_summaries
            .iter()
            .map(|fs| format!("文件: {}\n摘要: {}", fs.file_path, fs.summary))
            .collect::<Vec<_>>()
            .join("\n\n");

        // 获取模板内容并构建最终总结的提示词
        let prompt_manager = ai_manager.get_prompt_manager().await;
        let template = prompt_manager.get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

        // 构建临时的CommitContext用于语言处理
        let temp_context = crate::core::prompt_manager::CommitContext {
            diff: summary_content.clone(),
            staged_files: file_summaries.iter().map(|fs| fs.file_path.clone()).collect(),
            branch_name: _branch_name.clone(),
            commit_type: None,
            max_length: None,
            language: {
                // 从AI配置中获取语言设置
                let config = ai_manager.get_config().await;
                match config.base.language.as_str() {
                    "Simplified Chinese" => "zh-CN",
                    "Traditional Chinese" => "zh-TW",
                    "English" => "en",
                    "Japanese" => "ja",
                    "Korean" => "ko",
                    "French" => "fr",
                    "German" => "de",
                    "Spanish" => "es",
                    "Russian" => "ru",
                    "Portuguese" => "pt",
                    "Italian" => "it",
                    "Dutch" => "nl",
                    "Swedish" => "sv",
                    "Czech" => "cs",
                    "Polish" => "pl",
                    "Turkish" => "tr",
                    "Vietnamese" => "vi",
                    "Thai" => "th",
                    "Indonesian" => "id",
                    _ => "en", // 默认英文
                }.to_string()
            },
        };

        // 使用动态系统提示词生成，确保语言声明正确应用
        let mut system_prompt = prompt_manager.generate_dynamic_system_prompt(template, &temp_context);

        // 添加分层提交的特定说明
        system_prompt.push_str("\n\n特别说明：你现在正在处理分层提交的最终总结阶段。以上是各个文件的AI分析摘要，请基于这些摘要生成一个统一、简洁的提交消息。");

        let user_prompt = format!(
            "以下是各个文件的变更摘要:\n\n{}\n\n请生成一个简洁、准确的提交消息：",
            summary_content
        );

        let request = AIRequest {
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            model: ai_manager.get_config().await.base.model.clone(),
            temperature: Some(0.3),
            max_tokens: Some(500),
            stream: Some(false),
        };

        let start_time = std::time::Instant::now();
        let response = ai_manager.generate_commit_message(request.clone()).await?;
        let processing_time = start_time.elapsed().as_millis() as u64;

        // 记录最终提交消息生成的对话
        let step_info = StepInfo {
            step_type: "final_commit_generation".to_string(),
            step_index: None,
            total_steps: None,
            file_path: None,
            description: Some("生成最终提交消息".to_string()),
        };

        let record_id = ai_manager.log_conversation_with_session(
            "layered_commit".to_string(),
            repository_path,
            Some(session_id.to_string()),
            Some("layered".to_string()),
            Some(step_info),
            request,
            response.clone(),
            processing_time,
        ).await?;

        Ok(FinalCommitResult {
            message: response.content,
            record_id,
        })
    }

    /// 获取模型的最大token限制
    async fn get_model_max_tokens(&self, model_id: &str) -> Result<Option<u32>> {
        // 简化实现，返回常见模型的token限制
        let max_tokens = match model_id {
            m if m.contains("gpt-4") => Some(8192),
            m if m.contains("gpt-3.5") => Some(4096),
            m if m.contains("claude") => Some(100000),
            m if m.contains("gemini") => Some(32768),
            _ => Some(4096), // 默认限制
        };

        Ok(max_tokens)
    }
}

#[derive(Debug)]
struct SingleFileResult {
    summary: FileSummary,
    record_id: String,
}

#[derive(Debug)]
struct FinalCommitResult {
    message: String,
    record_id: String,
}
