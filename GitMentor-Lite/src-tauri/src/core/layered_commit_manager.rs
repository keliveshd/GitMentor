use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::ai_manager::AIManager;
use crate::core::git_engine::GitEngine;
use crate::core::conversation_logger::StepInfo;
use crate::core::ai_provider::{AIRequest, ChatMessage};
use crate::core::prompt_manager::{PromptManager, CommitContext};
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
    /// AI实时输出内容 - Author: Evilek, Date: 2025-01-10
    pub ai_stream_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub summary: String,
    #[serde(rename = "tokensUsed")]
    pub tokens_used: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredCommitResult {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "finalMessage")]
    pub final_message: String,
    #[serde(rename = "fileSummaries")]
    pub file_summaries: Vec<FileSummary>,
    #[serde(rename = "totalProcessingTimeMs")]
    pub total_processing_time_ms: u64,
    #[serde(rename = "conversationRecords")]
    pub conversation_records: Vec<String>, // 记录ID列表
    /// 推理内容（<think>标签内的内容）
    /// 作者：Evilek
    /// 编写日期：2025-01-10
    #[serde(rename = "reasoningContent")]
    pub reasoning_content: Option<String>,
}

pub struct LayeredCommitManager {
    ai_manager: Arc<RwLock<AIManager>>,
    git_engine: Arc<RwLock<GitEngine>>,
    cancelled: Arc<AtomicBool>, // 任务取消标志
}

impl LayeredCommitManager {
    pub fn new(
        ai_manager: Arc<RwLock<AIManager>>,
        git_engine: Arc<RwLock<GitEngine>>,
    ) -> Self {
        Self {
            ai_manager,
            git_engine,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 取消当前任务
    /// Author: Evilek, Date: 2025-01-09
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// 检查任务是否被取消
    /// Author: Evilek, Date: 2025-01-09
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
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
            max_tokens: Some(config.advanced.max_tokens), // 使用系统全局配置的max_tokens，而不是硬编码
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
        let files_with_diffs = self.get_files_with_diffs(&staged_files, template_id).await?;
        let total_files = files_with_diffs.len();

        // 初始化进度
        let mut progress = LayeredCommitProgress {
            session_id: session_id.clone(),
            current_step: 0,
            total_steps: (total_files + 1) as u32, // 文件分析 + 最终总结
            current_file: None,
            status: "开始分层提交".to_string(),
            file_summaries: Vec::new(),
            ai_stream_content: None,  // 初始化AI流式输出内容 - Author: Evilek, Date: 2025-01-10
        };
        progress_callback(progress.clone());

        // 第二步：为每个文件生成摘要
        let mut file_summaries = Vec::new();
        let mut conversation_records = Vec::new();

        for (index, (file_path, diff_content)) in files_with_diffs.iter().enumerate() {
            // 检查任务是否被取消 - Author: Evilek, Date: 2025-01-09
            if self.is_cancelled() {
                return Err(anyhow::anyhow!("分层提交已被用户取消"));
            }

            progress.current_step = (index + 1) as u32;
            progress.current_file = Some(file_path.clone());
            progress.status = format!("分析文件 {}/{}: {}", index + 1, total_files, file_path);

            let summary_result = self.analyze_single_file_with_stream(
                file_path,
                diff_content,
                template_id,
                &session_id,
                index as u32 + 1,
                total_files as u32,
                repository_path.clone(),
                &progress_callback,
            ).await?;

            file_summaries.push(summary_result.summary.clone());
            conversation_records.push(summary_result.record_id);
            progress.file_summaries.push(summary_result.summary);
        }

        // 第三步：生成最终的提交消息
        progress.current_step = total_files as u32 + 1;
        progress.current_file = None;
        progress.status = "生成最终提交消息".to_string();

        let final_result = self.generate_final_commit_message_with_stream(
            template_id,
            &file_summaries,
            branch_name,
            &session_id,
            repository_path.clone(),
            &progress_callback,
        ).await?;

        conversation_records.push(final_result.record_id);

        let total_time = start_time.elapsed().as_millis() as u64;

        Ok(LayeredCommitResult {
            session_id,
            final_message: final_result.message,
            file_summaries,
            total_processing_time_ms: total_time,
            conversation_records,
            reasoning_content: final_result.reasoning_content, // 添加推理内容 - Author: Evilek, Date: 2025-01-10
        })
    }

    /// 获取文件及其diff内容
    /// Author: Evilek, Date: 2025-01-08
    /// 支持处理带有特殊标记的文件（#truncated, #split）
    /// Updated: Evilek, Date: 2025-01-09 - 添加template_id参数用于文件分割控制
    async fn get_files_with_diffs(&self, staged_files: &[String], template_id: &str) -> Result<Vec<(String, String)>> {
        let git_engine = self.git_engine.read().await;
        let mut files_with_diffs = Vec::new();

        for file_path in staged_files {
            // 检查文件是否有特殊标记
            if file_path.contains("#truncated") {
                // 新增文件截取处理
                let actual_path = file_path.replace("#truncated", "");
                match git_engine.get_simple_file_diff(&actual_path) {
                    Ok(diff_content) => {
                        // 截取文件内容的前面部分（根据模板的max_tokens限制）
                        let truncated_content = self.truncate_new_file_content_with_template(&actual_path, &diff_content, template_id).await?;
                        files_with_diffs.push((actual_path, truncated_content));
                    },
                    Err(e) => {
                        return Err(anyhow::anyhow!("获取新增文件diff失败 {}: {}", actual_path, e));
                    }
                }
            } else if file_path.contains("#split") {
                // 变更文件分割处理
                let actual_path = file_path.replace("#split", "");
                match git_engine.get_simple_file_diff(&actual_path) {
                    Ok(diff_content) => {
                        // 分割大文件内容，使用模板的max_tokens作为分割依据
                        let split_contents = self.split_file_content_with_template(&actual_path, &diff_content, template_id).await?;
                        for (index, content) in split_contents.into_iter().enumerate() {
                            let split_path = format!("{}#part{}", actual_path, index + 1);
                            files_with_diffs.push((split_path, content));
                        }
                    },
                    Err(e) => {
                        return Err(anyhow::anyhow!("获取分割文件diff失败 {}: {}", actual_path, e));
                    }
                }

            } else {
                // 普通文件处理
                match git_engine.get_simple_file_diff(file_path) {
                    Ok(diff_content) => {
                        files_with_diffs.push((file_path.clone(), diff_content));
                    },
                    Err(e) => {
                        return Err(anyhow::anyhow!("获取文件diff失败 {}: {}", file_path, e));
                    }
                }
            }
        }

        Ok(files_with_diffs)
    }

    /// 分析单个文件（带流式输出支持）
    /// Author: Evilek, Date: 2025-01-10
    async fn analyze_single_file_with_stream<F>(
        &self,
        file_path: &str,
        diff_content: &str,
        template_id: &str,
        session_id: &str,
        step_index: u32,
        total_steps: u32,
        repository_path: Option<String>,
        progress_callback: &F,
    ) -> Result<SingleFileResult>
    where
        F: Fn(LayeredCommitProgress) + Send + Sync,
    {
        let ai_manager = self.ai_manager.read().await;
        let config = ai_manager.get_config().await;

        // 使用统一的模板系统生成提示词（重构优化）
        // Author: Evilek, Date: 2025-01-08
        // 移除批量文件处理逻辑，改为单文件独立处理 - Author: Evilek, Date: 2025-01-09
        let context = CommitContext {
            diff: diff_content.to_string(),
            staged_files: vec![file_path.to_string()],
            branch_name: None,
            commit_type: None,
            max_length: None,
            language: "zh-CN".to_string(),
        };

        // 使用PromptManager生成消息，统一模板系统
        let prompt_manager = PromptManager::new();
        let messages = prompt_manager
            .generate_file_analysis_messages(template_id, file_path, diff_content, &context)
            .map_err(|e| anyhow::anyhow!("生成文件分析消息失败: {}", e))?;

        // 转换为AIRequest格式，移除max_tokens限制确保完整输出 - Author: Evilek, Date: 2025-01-10
        let request = AIRequest {
            messages,
            model: config.base.model.clone(),
            temperature: Some(0.3),
            max_tokens: None,  // 移除token限制，让AI完整输出
            stream: Some(false),
        };

        // 显示AI分析开始状态 - Author: Evilek, Date: 2025-01-10
        let mut progress = LayeredCommitProgress {
            session_id: session_id.to_string(),
            current_step: step_index,
            total_steps,
            current_file: Some(file_path.to_string()),
            status: format!("分析文件 {}/{}: {}", step_index, total_steps, file_path),
            file_summaries: Vec::new(),
            ai_stream_content: Some(format!("⚡ 正在分析文件: {}\n\n📤 发送请求到AI...", file_path)),
        };
        progress_callback(progress.clone());

        // 在AI调用过程中提供流式更新 - Author: Evilek, Date: 2025-01-10
        let start_time = std::time::Instant::now();

        // 创建一个future来处理AI调用，并使用pin!宏固定它
        let ai_future = ai_manager.generate_commit_message(request.clone());
        tokio::pin!(ai_future);

        // 使用tokio::select来同时处理AI调用和进度更新
        let response = {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000));
            let mut step = 0;

            loop {
                tokio::select! {
                    result = &mut ai_future => {
                        // AI调用完成
                        break result?;
                    }
                    _ = interval.tick() => {
                        // 更新进度
                        step += 1;
                        let content = match step {
                            1 => format!("⚡ 正在分析文件: {}\n\n⏳ AI正在接收和处理请求...", file_path),
                            2 => format!("⚡ 正在分析文件: {}\n\n🧠 AI正在分析代码变更...", file_path),
                            3 => format!("⚡ 正在分析文件: {}\n\n💭 AI正在生成分析结果...", file_path),
                            _ => format!("⚡ 正在分析文件: {}\n\n⏳ AI正在完成分析...", file_path),
                        };

                        progress.ai_stream_content = Some(content);
                        progress_callback(progress.clone());
                    }
                }
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // 模拟流式显示AI的真实响应内容 - Author: Evilek, Date: 2025-01-10
        let mut ai_output = String::new();

        // 如果有推理内容，先流式显示推理过程
        if let Some(reasoning) = &response.reasoning_content {
            ai_output.push_str("🧠 AI推理过程:\n<think>\n");

            // 逐字符显示推理内容，减少延迟提高响应速度
            let reasoning_chars: Vec<char> = reasoning.chars().collect();
            let chunk_size = 30; // 增加每次显示的字符数

            for chunk in reasoning_chars.chunks(chunk_size) {
                let chunk_str: String = chunk.iter().collect();
                ai_output.push_str(&chunk_str);

                progress.ai_stream_content = Some(format!("{}\n</think>\n\n📝 正在生成分析结果...", ai_output));
                progress_callback(progress.clone());

                // 减少延迟，提高流式输出速度 - Author: Evilek, Date: 2025-01-10
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            ai_output.push_str("\n</think>\n\n");
        }

        // 流式显示分析结果
        ai_output.push_str("📝 分析结果:\n");
        let content_chars: Vec<char> = response.content.chars().collect();
        let chunk_size = 25; // 增加每次显示的字符数

        for chunk in content_chars.chunks(chunk_size) {
            let chunk_str: String = chunk.iter().collect();
            ai_output.push_str(&chunk_str);

            progress.ai_stream_content = Some(ai_output.clone());
            progress_callback(progress.clone());

            // 减少延迟，提高流式输出速度 - Author: Evilek, Date: 2025-01-10
            tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        }

        // 记录对话
        let step_info = StepInfo {
            step_type: "file_analysis".to_string(),
            step_index: Some(step_index),
            total_steps: Some(total_steps),
            file_path: Some(file_path.to_string()),
            description: Some(format!("分析文件: {}", file_path)),
        };

        let record_id = ai_manager.log_conversation_with_session(
            template_id.to_string(),
            repository_path,
            Some(session_id.to_string()),
            Some("layered".to_string()),
            Some(step_info),
            request,
            response.clone(),
            processing_time,
        ).await?;

        let summary = FileSummary {
            file_path: file_path.to_string(),
            summary: response.content.clone(),
            tokens_used: response.usage.map(|u| u.total_tokens).unwrap_or(0),
        };

        Ok(SingleFileResult {
            summary,
            record_id,
        })
    }

    /// 分析单个文件（原方法，保持向后兼容）
    async fn analyze_single_file(
        &self,
        file_path: &str,
        diff_content: &str,
        template_id: &str,
        session_id: &str,
        step_index: u32,
        total_steps: u32,
        repository_path: Option<String>,
    ) -> Result<SingleFileResult> {
        let ai_manager = self.ai_manager.read().await;
        let config = ai_manager.get_config().await;

        // 使用统一的模板系统生成提示词（重构优化）
        // Author: Evilek, Date: 2025-01-08
        // 移除批量文件处理逻辑，改为单文件独立处理 - Author: Evilek, Date: 2025-01-09
        let context = CommitContext {
            diff: diff_content.to_string(),
            staged_files: vec![file_path.to_string()],
            branch_name: None,
            commit_type: None,
            max_length: None,
            language: Self::convert_ai_language_to_code(&config.base.language),
        };

        // 使用PromptManager生成消息，统一模板系统
        let prompt_manager = PromptManager::new();
        let messages = prompt_manager
            .generate_file_analysis_messages(template_id, file_path, diff_content, &context)
            .map_err(|e| anyhow::anyhow!("生成文件分析消息失败: {}", e))?;

        // 转换为AIRequest格式 - Author: Evilek, Date: 2025-01-10
        let request = AIRequest {
            messages,
            model: config.base.model.clone(),
            temperature: Some(0.3),
            max_tokens: Some(config.advanced.max_tokens), // 使用系统全局配置的max_tokens，而不是硬编码
            stream: Some(false),  // 当前提供商实现不支持流式输出，保持false
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

    /// 生成最终提交消息（带流式输出支持）
    /// Author: Evilek, Date: 2025-01-10
    async fn generate_final_commit_message_with_stream<F>(
        &self,
        template_id: &str,
        file_summaries: &[FileSummary],
        _branch_name: Option<String>,
        session_id: &str,
        repository_path: Option<String>,
        progress_callback: &F,
    ) -> Result<FinalCommitResult>
    where
        F: Fn(LayeredCommitProgress) + Send + Sync,
    {
        let ai_manager = self.ai_manager.read().await;

        // 构建汇总的diff内容
        let summary_content = file_summaries
            .iter()
            .map(|fs| format!("文件: {}\n摘要: {}", fs.file_path, fs.summary))
            .collect::<Vec<_>>()
            .join("\n\n");

        // 获取模板内容并构建最终总结的提示词
        let prompt_manager = ai_manager.get_prompt_manager().await;
        let _template = prompt_manager.get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

        // 构建上下文
        let context = CommitContext {
            diff: summary_content.clone(),
            staged_files: file_summaries.iter().map(|fs| fs.file_path.clone()).collect(),
            branch_name: None,
            commit_type: None,
            max_length: None,
            language: "zh-CN".to_string(),
        };

        // 使用统一生成的消息（重构优化）
        let file_summary_strs: Vec<&str> = file_summaries.iter().map(|fs| fs.summary.as_str()).collect();
        let messages = prompt_manager.generate_summary_messages(template_id, &context, &file_summary_strs)
            .map_err(|e| anyhow::anyhow!("生成总结消息失败: {}", e))?;

        // 使用统一生成的消息（重构优化），移除max_tokens限制 - Author: Evilek, Date: 2025-01-10
        let config = ai_manager.get_config().await;
        let request = AIRequest {
            messages: messages.clone(),
            model: config.base.model.clone(),
            temperature: Some(0.3),
            max_tokens: None,  // 移除token限制，让AI完整输出最终提交消息
            stream: Some(false),
        };

        // 显示最终提交消息生成开始状态 - Author: Evilek, Date: 2025-01-10
        let mut progress = LayeredCommitProgress {
            session_id: session_id.to_string(),
            current_step: file_summaries.len() as u32 + 1,
            total_steps: file_summaries.len() as u32 + 1,
            current_file: None,
            status: "生成最终提交消息".to_string(),
            file_summaries: file_summaries.to_vec(),  // 保持已有的文件摘要 - Author: Evilek, Date: 2025-01-10
            ai_stream_content: Some("🎯 正在生成最终提交消息...\n\n📤 发送汇总请求到AI...".to_string()),
        };
        progress_callback(progress.clone());

        // 在AI调用过程中提供流式更新 - Author: Evilek, Date: 2025-01-10
        let start_time = std::time::Instant::now();
        let file_count = file_summaries.len();

        // 创建一个future来处理AI调用，并使用pin!宏固定它
        let ai_future = ai_manager.generate_commit_message(request.clone());
        tokio::pin!(ai_future);

        // 使用tokio::select来同时处理AI调用和进度更新
        let response = {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1200));
            let mut step = 0;

            loop {
                tokio::select! {
                    result = &mut ai_future => {
                        // AI调用完成
                        break result?;
                    }
                    _ = interval.tick() => {
                        // 更新进度
                        step += 1;
                        let content = match step {
                            1 => format!("🎯 正在生成最终提交消息...\n\n📊 基于 {} 个文件的分析结果\n⏳ AI正在接收和处理汇总请求...", file_count),
                            2 => format!("🎯 正在生成最终提交消息...\n\n📊 基于 {} 个文件的分析结果\n🧠 AI正在整合所有分析结果...", file_count),
                            3 => format!("🎯 正在生成最终提交消息...\n\n📊 基于 {} 个文件的分析结果\n🎨 AI正在生成统一的提交消息...", file_count),
                            _ => format!("🎯 正在生成最终提交消息...\n\n📊 基于 {} 个文件的分析结果\n⏳ AI正在完成提交消息生成...", file_count),
                        };

                        progress.ai_stream_content = Some(content);
                        progress_callback(progress.clone());
                    }
                }
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // 模拟流式显示AI的真实响应内容 - Author: Evilek, Date: 2025-01-10
        let mut ai_output = String::new();

        // 如果有推理内容，先流式显示推理过程
        if let Some(reasoning) = &response.reasoning_content {
            ai_output.push_str("🧠 AI推理过程:\n<think>\n");

            // 逐字符显示推理内容，优化速度
            let reasoning_chars: Vec<char> = reasoning.chars().collect();
            let chunk_size = 35; // 增加每次显示的字符数

            for chunk in reasoning_chars.chunks(chunk_size) {
                let chunk_str: String = chunk.iter().collect();
                ai_output.push_str(&chunk_str);

                progress.ai_stream_content = Some(format!("{}\n</think>\n\n📝 正在生成最终提交消息...", ai_output));
                progress_callback(progress.clone());

                // 减少延迟，提高流式输出速度 - Author: Evilek, Date: 2025-01-10
                tokio::time::sleep(tokio::time::Duration::from_millis(60)).await;
            }

            ai_output.push_str("\n</think>\n\n");
        }

        // 流式显示最终提交消息
        ai_output.push_str("📝 最终提交消息:\n");
        let content_chars: Vec<char> = response.content.chars().collect();
        let chunk_size = 30; // 增加每次显示的字符数

        for chunk in content_chars.chunks(chunk_size) {
            let chunk_str: String = chunk.iter().collect();
            ai_output.push_str(&chunk_str);

            progress.ai_stream_content = Some(ai_output.clone());
            progress_callback(progress.clone());

            // 减少延迟，提高流式输出速度 - Author: Evilek, Date: 2025-01-10
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        // 记录最终提交消息生成的对话
        let step_info = StepInfo {
            step_type: "final_commit_generation".to_string(),
            step_index: None,
            total_steps: None,
            file_path: None,
            description: Some("生成最终提交消息".to_string()),
        };

        let record_id = ai_manager.log_conversation_with_session(
            template_id.to_string(),
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
            reasoning_content: response.reasoning_content,
        })
    }

    /// 生成最终提交消息（原方法，保持向后兼容）
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
        let _template = prompt_manager.get_template(template_id)
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
                Self::convert_ai_language_to_code(&config.base.language)
            },
        };

        // 使用统一的总结消息生成系统（重构优化）
        // Author: Evilek, Date: 2025-01-08
        let file_summaries: Vec<&str> = file_summaries.iter().map(|s| s.summary.as_str()).collect();
        let messages = prompt_manager
            .generate_summary_messages(template_id, &temp_context, &file_summaries)
            .map_err(|e| anyhow::anyhow!("生成总结消息失败: {}", e))?;

        // 使用统一生成的消息（重构优化）
        let config = ai_manager.get_config().await;
        let request = AIRequest {
            messages,
            model: config.base.model.clone(),
            temperature: Some(0.3),
            max_tokens: Some(config.advanced.max_tokens), // 使用系统全局配置的max_tokens，而不是硬编码
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
            reasoning_content: response.reasoning_content, // 添加推理内容 - Author: Evilek, Date: 2025-01-10
        })
    }

    /// 获取模型的最大token限制
    async fn get_model_max_tokens(&self, model_id: &str) -> Result<Option<u32>> {
        // 简化实现，返回常见模型的token限制
        // Author: Evilek, Date: 2025-01-09 - 添加qwen模型支持
        let max_tokens = match model_id {
            m if m.contains("gpt-4") => Some(8192),
            m if m.contains("gpt-3.5") => Some(4096),
            m if m.contains("claude") => Some(100000),
            m if m.contains("gemini") => Some(32768),
            m if m.contains("qwen2.5:32b") => Some(32768), // qwen2.5:32b 支持32k上下文
            m if m.contains("qwen") => Some(8192), // 其他qwen模型默认8k
            _ => Some(4096), // 默认限制
        };

        Ok(max_tokens)
    }

    /// 截取新增文件内容（使用模板配置）
    /// Author: Evilek, Date: 2025-01-09
    /// 根据模板的max_tokens限制截取新增文件的前面部分，并包含文件名上下文
    async fn truncate_new_file_content_with_template(&self, file_path: &str, diff_content: &str, template_id: &str) -> Result<String> {
        // 获取模板的max_tokens配置作为截取依据
        // Author: Evilek, Date: 2025-01-09 - 修复PromptManager实例化问题，使用AI管理器中的实例
        let ai_manager = self.ai_manager.read().await;
        let prompt_manager = ai_manager.get_prompt_manager().await;
        let template_max_tokens = prompt_manager.get_template_config(template_id)
            .and_then(|(max_tokens, _)| max_tokens)
            .unwrap_or(1000); // 修复：增加默认值到1000 tokens，避免过度截取

        // 使用模板的max_tokens作为截取的安全限制（保留30%余量给文件名和格式）
        let safe_limit = (template_max_tokens as f32 * 0.7) as u32;

        // 预估文件名和格式开销的token数
        let file_context_tokens = TokenCounter::estimate_tokens(&format!("文件: {}\n\n", file_path)) + 50;

        let lines: Vec<&str> = diff_content.lines().collect();
        let total_lines = lines.len();
        let mut truncated_lines = Vec::new();
        let mut current_tokens = file_context_tokens;

        for line in &lines {
            let line_tokens = TokenCounter::estimate_tokens(line);
            if current_tokens + line_tokens > safe_limit {
                break;
            }
            truncated_lines.push(*line);
            current_tokens += line_tokens;
        }

        let truncated_content = truncated_lines.join("\n");
        let truncated_line_count = truncated_lines.len();

        // 添加文件名上下文和截取说明
        let result = if truncated_line_count < total_lines {
            format!("文件: {}\n\n{}\n\n# 文件内容已截取，显示前{}行（共{}行）",
                file_path, truncated_content, truncated_line_count, total_lines)
        } else {
            format!("文件: {}\n\n{}", file_path, truncated_content)
        };

        Ok(result)
    }

    /// 截取新增文件内容（保留原方法用于向后兼容）
    /// Author: Evilek, Date: 2025-01-08
    /// 根据token限制截取新增文件的前面部分
    async fn truncate_new_file_content(&self, diff_content: &str) -> Result<String> {
        let ai_manager = self.ai_manager.read().await;
        let config = ai_manager.get_config().await;
        let model_max_tokens = self.get_model_max_tokens(&config.base.model).await?;

        // 计算安全的token限制（保留30%余量）
        let safe_limit = if let Some(max_tokens) = model_max_tokens {
            (max_tokens as f32 * 0.7) as u32
        } else {
            2800 // 默认安全限制
        };

        let lines: Vec<&str> = diff_content.lines().collect();
        let total_lines = lines.len();
        let mut truncated_lines = Vec::new();
        let mut current_tokens = 0u32;

        for line in &lines {
            let line_tokens = TokenCounter::estimate_tokens(line);
            if current_tokens + line_tokens > safe_limit {
                break;
            }
            truncated_lines.push(*line);
            current_tokens += line_tokens;
        }

        // 添加截取说明
        let mut result = truncated_lines.join("\n");
        if truncated_lines.len() < total_lines {
            result.push_str("\n...");
            result.push_str(&format!("\n# 文件内容已截取，显示前{}行（共{}行）", truncated_lines.len(), total_lines));
        }

        Ok(result)
    }

    /// 分割文件内容（使用模板配置）
    /// Author: Evilek, Date: 2025-01-09
    /// 根据模板的max_tokens配置将大文件内容分割成多个部分，每个部分都包含文件名上下文
    async fn split_file_content_with_template(&self, file_path: &str, diff_content: &str, template_id: &str) -> Result<Vec<String>> {
        // 获取模板的max_tokens配置作为分割依据
        // Author: Evilek, Date: 2025-01-09 - 修复PromptManager实例化问题，使用AI管理器中的实例
        let ai_manager = self.ai_manager.read().await;
        let prompt_manager = ai_manager.get_prompt_manager().await;
        let template_max_tokens = prompt_manager.get_template_config(template_id)
            .and_then(|(max_tokens, _)| max_tokens)
            .unwrap_or(1000); // 修复：增加默认值到1000 tokens，避免过度分割

        println!("🔍 [split_file_content_with_template] 模板 {} 的max_tokens: {}", template_id, template_max_tokens);

        // 使用模板的max_tokens作为分割的安全限制（保留30%余量给文件名和格式）
        let safe_limit = (template_max_tokens as f32 * 0.7) as u32;

        println!("🔍 [split_file_content_with_template] 分割安全限制: {} tokens", safe_limit);

        let lines: Vec<&str> = diff_content.lines().collect();
        let mut split_contents = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_tokens = 0u32;

        // 预估文件名和格式开销的token数
        let file_context_tokens = TokenCounter::estimate_tokens(&format!("文件: {}\n\n", file_path)) + 50;

        for line in &lines {
            let line_tokens = TokenCounter::estimate_tokens(line);

            // 如果添加这一行会超过限制，保存当前块并开始新块
            if current_tokens + line_tokens + file_context_tokens > safe_limit && !current_chunk.is_empty() {
                // 为每个分割部分添加文件名上下文
                let chunk_with_context = format!("文件: {}\n\n{}", file_path, current_chunk.join("\n"));
                split_contents.push(chunk_with_context);
                current_chunk.clear();
                current_tokens = 0;
            }

            current_chunk.push(*line);
            current_tokens += line_tokens;
        }

        // 添加最后一个块
        if !current_chunk.is_empty() {
            let chunk_with_context = format!("文件: {}\n\n{}", file_path, current_chunk.join("\n"));
            split_contents.push(chunk_with_context);
        }

        // 如果有多个分片，为每个分片添加分片说明
        if split_contents.len() > 1 {
            let total_parts = split_contents.len();
            for (index, content) in split_contents.iter_mut().enumerate() {
                content.push_str(&format!("\n\n# 这是文件 {} 的第{}部分（共{}部分）", file_path, index + 1, total_parts));
            }
        }

        Ok(split_contents)
    }

    /// 分割文件内容（保留原方法用于向后兼容）
    /// Author: Evilek, Date: 2025-01-08
    /// 将大文件内容分割成多个部分
    async fn split_file_content(&self, diff_content: &str) -> Result<Vec<String>> {
        let ai_manager = self.ai_manager.read().await;
        let config = ai_manager.get_config().await;
        let model_max_tokens = self.get_model_max_tokens(&config.base.model).await?;

        // 计算每个分片的安全token限制
        let safe_limit = if let Some(max_tokens) = model_max_tokens {
            (max_tokens as f32 * 0.6) as u32 // 更保守的限制
        } else {
            2400 // 默认安全限制
        };

        let lines: Vec<&str> = diff_content.lines().collect();
        let mut split_contents = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_tokens = 0u32;

        for line in &lines {
            let line_tokens = TokenCounter::estimate_tokens(line);

            // 如果添加这一行会超过限制，保存当前块并开始新块
            if current_tokens + line_tokens > safe_limit && !current_chunk.is_empty() {
                split_contents.push(current_chunk.join("\n"));
                current_chunk.clear();
                current_tokens = 0;
            }

            current_chunk.push(*line);
            current_tokens += line_tokens;
        }

        // 添加最后一个块
        if !current_chunk.is_empty() {
            split_contents.push(current_chunk.join("\n"));
        }

        // 如果只有一个块，说明不需要分割
        if split_contents.len() == 1 {
            return Ok(split_contents);
        }

        // 为每个分片添加说明
        let total_parts = split_contents.len();
        for (index, content) in split_contents.iter_mut().enumerate() {
            content.push_str(&format!("\n# 这是文件的第{}部分（共{}部分）", index + 1, total_parts));
        }

        Ok(split_contents)
    }

    /// 将AI配置中的语言名称转换为语言代码
    /// Author: Evilek, Date: 2025-01-08
    /// 统一语言转换逻辑，避免代码重复
    fn convert_ai_language_to_code(language_name: &str) -> String {
        match language_name {
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
    /// 推理内容（<think>标签内的内容）
    /// 作者：Evilek
    /// 编写日期：2025-01-10
    reasoning_content: Option<String>,
}
