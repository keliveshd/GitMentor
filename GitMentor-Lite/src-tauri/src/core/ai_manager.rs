use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::ai_config::{AIConfig, AIConfigManager};
use crate::core::ai_provider::{
    AIModel, AIProviderFactory, AIRequest, AIResponse, ConnectionTestResult,
};
use crate::core::ai_response_cache::AIResponseCache;
use crate::core::conversation_logger::{ConversationLogger, ConversationRecord};
use crate::core::prompt_manager::{CommitContext, PromptManager, PromptTemplate};
use crate::core::providers::create_provider_factory;

/**
 * AI管理器 - 统一管理所有AI提供商
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Clone)]
pub struct AIManager {
    config_manager: Arc<RwLock<AIConfigManager>>,
    provider_factory: Arc<RwLock<AIProviderFactory>>,
    prompt_manager: Arc<RwLock<PromptManager>>,
    conversation_logger: Arc<RwLock<ConversationLogger>>,
    response_cache: Arc<RwLock<AIResponseCache>>,
}

impl AIManager {
    pub fn new(config_path: std::path::PathBuf) -> Result<Self> {
        let config_manager = AIConfigManager::new(config_path.clone())?;
        let config = config_manager.get_config().clone();
        let provider_factory = create_provider_factory(&config);

        // 创建对话记录器，日志文件放在配置目录下
        let mut log_path = config_path.clone();
        log_path.pop(); // 移除文件名，保留目录
        log_path.push("conversation_history.json");
        let conversation_logger = ConversationLogger::new(log_path)?;

        // 创建模板配置文件路径
        let mut template_config_path = config_path.clone();
        template_config_path.pop(); // 移除文件名，保留目录
        template_config_path.push("prompt_templates.json");
        let prompt_manager = PromptManager::new_with_config(template_config_path)?;

        // 创建响应缓存
        let mut cache_dir = config_path.clone();
        cache_dir.pop(); // 移除文件名，保留目录
        cache_dir.push("cache");
        std::fs::create_dir_all(&cache_dir)?;
        let response_cache = AIResponseCache::new(cache_dir);

        Ok(Self {
            config_manager: Arc::new(RwLock::new(config_manager)),
            provider_factory: Arc::new(RwLock::new(provider_factory)),
            prompt_manager: Arc::new(RwLock::new(prompt_manager)),
            conversation_logger: Arc::new(RwLock::new(conversation_logger)),
            response_cache: Arc::new(RwLock::new(response_cache)),
        })
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> AIConfig {
        let config_manager = self.config_manager.read().await;
        config_manager.get_config().clone()
    }

    /// 更新配置
    pub async fn update_config(&self, config: AIConfig) -> Result<()> {
        // 更新配置管理器
        {
            let mut config_manager = self.config_manager.write().await;
            config_manager.update_config(config.clone())?;
        }

        // 重新创建提供商工厂
        {
            let mut factory = self.provider_factory.write().await;
            *factory = create_provider_factory(&config);
        }

        Ok(())
    }

    /// 获取指定提供商（返回是否存在）
    #[allow(dead_code)]
    pub async fn has_provider(&self, provider_id: &str) -> bool {
        let factory = self.provider_factory.read().await;
        factory.get_provider(provider_id).is_some()
    }

    /// 获取所有提供商信息
    pub async fn get_providers_info(&self) -> Vec<(String, String)> {
        let factory = self.provider_factory.read().await;
        factory.get_providers_info()
    }

    /// 生成提交消息
    pub async fn generate_commit_message(&self, request: AIRequest) -> Result<AIResponse> {
        let config = self.get_config().await;
        let provider_id = &config.base.provider;

        let factory = self.provider_factory.read().await;
        factory.generate_commit(provider_id, &request).await
    }

    /// 生成AI分析报告
    pub async fn generate_analysis_report(&self, request: AIRequest) -> Result<AIResponse> {
        let start_time = std::time::Instant::now();
        let config = self.get_config().await;
        let provider_id = &config.base.provider;

        let factory = self.provider_factory.read().await;
        let result = factory.generate_commit(provider_id, &request).await;

        // 记录对话
        let mut logger = self.conversation_logger.write().await;
        match &result {
            Ok(response) => {
                let _ = logger.log_success(
                    "ai_analysis_report".to_string(),
                    None, // 仓库路径
                    request.clone(),
                    response.clone(),
                    start_time.elapsed().as_millis() as u64,
                );
            }
            Err(error) => {
                let _ = logger.log_failure(
                    "ai_analysis_report".to_string(),
                    None, // 仓库路径
                    request.clone(),
                    error.to_string(),
                    start_time.elapsed().as_millis() as u64,
                );
            }
        }

        result
    }

    /// 获取指定提供商的模型列表
    pub async fn get_models_for_provider(&self, provider_id: &str) -> Result<Vec<AIModel>> {
        let factory = self.provider_factory.read().await;
        factory.get_models(provider_id).await
    }

    /// 测试指定提供商的连接
    pub async fn test_provider_connection(
        &self,
        provider_id: &str,
    ) -> Result<ConnectionTestResult> {
        let factory = self.provider_factory.read().await;
        factory.test_connection(provider_id).await
    }

    /// 刷新指定提供商的模型列表
    pub async fn refresh_provider_models(&self, provider_id: &str) -> Result<Vec<AIModel>> {
        let factory = self.provider_factory.read().await;
        factory.refresh_models(provider_id).await
    }

    /// 检查指定提供商是否可用
    pub async fn is_provider_available(&self, provider_id: &str) -> bool {
        let factory = self.provider_factory.read().await;
        factory.is_available(provider_id).await
    }

    /// 使用提示模板生成提交消息
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    /// 更新日期：2025-01-19 - 添加缓存和重试逻辑
    pub async fn generate_commit_with_template(
        &self,
        template_id: &str,
        context: CommitContext,
        repository_path: Option<String>,
    ) -> Result<AIResponse> {
        use std::time::Instant;

        let start_time = Instant::now();
        let config = self.get_config().await;
        let provider_id = &config.base.provider;

        // 生成消息
        let prompt_manager = self.prompt_manager.read().await;
        let messages = prompt_manager.generate_messages(template_id, &context)?;

        // 获取模板配置，但优先使用系统全局的max_tokens配置
        let (_template_max_tokens, temperature) = prompt_manager
            .get_template_config(template_id)
            .unwrap_or((Some(200), Some(0.3)));

        // 使用系统全局配置的max_tokens，而不是模板中的小数值，避免响应被截断
        // Author: Evilek, Date: 2025-01-09
        let max_tokens = Some(config.advanced.max_tokens);

        // 检查缓存 - 使用diff和staged_files生成缓存键
        let file_changes: Vec<(String, String)> = context
            .staged_files
            .iter()
            .map(|file| (file.clone(), context.diff.clone()))
            .collect();

        let cache_key_repo_path = repository_path.as_deref();
        let cache = self.response_cache.write().await;

        if let Some(cached_entry) = cache.get(
            template_id,
            &config.base.model,
            &file_changes,
            cache_key_repo_path,
        ) {
            drop(cache);

            // 从缓存创建响应
            let cached_response = AIResponse {
                content: cached_entry.content,
                model: cached_entry.model,
                usage: None, // 缓存不保存usage信息
                finish_reason: Some("stop".to_string()),
                reasoning_content: cached_entry.reasoning_content,
            };

            eprintln!("🎯 [Cache] 返回缓存响应，跳过API调用");
            return Ok(cached_response);
        }
        drop(cache);

        let request = AIRequest {
            messages,
            model: config.base.model.clone(),
            temperature,
            max_tokens,
            stream: Some(false),
        };

        // 重试逻辑
        let max_retries = config.advanced.retry_count.max(1);
        let mut last_error = None;

        for retry in 0..=max_retries {
            let factory = self.provider_factory.read().await;
            let result = factory.generate_commit(provider_id, &request).await;
            let processing_time = start_time.elapsed().as_millis() as u64;

            match result {
                Ok(response) => {
                    // 记录对话 - 包含仓库路径信息
                    let mut logger = self.conversation_logger.write().await;
                    let _ = logger.log_success(
                        template_id.to_string(),
                        repository_path.clone(),
                        request.clone(),
                        response.clone(),
                        processing_time,
                    );
                    drop(logger);

                    // 缓存成功响应
                    let mut cache = self.response_cache.write().await;
                    let _ = cache.set(
                        template_id,
                        &config.base.model,
                        &file_changes,
                        cache_key_repo_path,
                        response.content.clone(),
                        response.reasoning_content.clone(),
                    );

                    if retry > 0 {
                        eprintln!("🔄 [Retry] 第 {} 次重试成功", retry);
                    }

                    return Ok(response);
                }
                Err(error) => {
                    last_error = Some(error);

                    // 记录失败日志
                    let mut logger = self.conversation_logger.write().await;
                    let _ = logger.log_failure(
                        template_id.to_string(),
                        repository_path.clone(),
                        request.clone(),
                        last_error.as_ref().unwrap().to_string(),
                        processing_time,
                    );
                    drop(logger);

                    if retry < max_retries {
                        eprintln!("⚠️ [Retry] 第 {} 次尝试失败，准备重试...", retry + 1);
                        // 指数退避等待
                        let wait_time =
                            tokio::time::Duration::from_millis(1000 * 2_u64.pow(retry as u32));
                        tokio::time::sleep(wait_time).await;
                    }
                }
            }
        }

        // 所有重试都失败
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("未知错误")))
    }

    /// 获取所有可用的提示模板
    pub async fn get_prompt_templates(&self) -> Vec<PromptTemplate> {
        let prompt_manager = self.prompt_manager.read().await;
        prompt_manager
            .get_all_templates()
            .into_iter()
            .cloned()
            .collect()
    }

    /// 获取提示词管理器的只读引用
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    pub async fn get_prompt_manager(&self) -> tokio::sync::RwLockReadGuard<'_, PromptManager> {
        self.prompt_manager.read().await
    }

    /// 获取对话记录
    pub async fn get_conversation_history(&self) -> Vec<ConversationRecord> {
        let logger = self.conversation_logger.read().await;
        logger.get_all_records().clone()
    }

    /// 清空对话记录
    pub async fn clear_conversation_history(&self) -> Result<()> {
        let mut logger = self.conversation_logger.write().await;
        logger.clear_all_records()
    }

    /// 根据仓库路径获取对话记录
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub async fn get_conversation_history_by_repository(
        &self,
        repository_path: Option<&str>,
    ) -> Result<Vec<ConversationRecord>> {
        let logger = self.conversation_logger.read().await;
        let records = logger.get_records_by_repository(repository_path);
        Ok(records.into_iter().cloned().collect())
    }

    /// 获取所有仓库路径列表
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub async fn get_repository_paths(&self) -> Result<Vec<String>> {
        let logger = self.conversation_logger.read().await;
        Ok(logger.get_repository_paths())
    }

    /// 获取所有分层提交会话
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub async fn get_layered_sessions(&self) -> Result<Vec<String>> {
        let logger = self.conversation_logger.read().await;
        Ok(logger.get_layered_sessions())
    }

    /// 根据会话ID获取对话记录
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub async fn get_conversation_records_by_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<ConversationRecord>> {
        let logger = self.conversation_logger.read().await;
        let records = logger.get_records_by_session(session_id);
        Ok(records.into_iter().cloned().collect())
    }

    /// 记录对话（带会话信息）
    /// 作者：Evilek
    /// 编写日期：2025-08-05
    pub async fn log_conversation_with_session(
        &self,
        template_id: String,
        repository_path: Option<String>,
        session_id: Option<String>,
        session_type: Option<String>,
        step_info: Option<crate::core::conversation_logger::StepInfo>,
        request: AIRequest,
        response: AIResponse,
        processing_time_ms: u64,
    ) -> Result<String> {
        let mut logger = self.conversation_logger.write().await;
        logger.log_success_with_session(
            template_id,
            repository_path,
            session_id,
            session_type,
            step_info,
            request,
            response,
            processing_time_ms,
        )?;
        Ok(uuid::Uuid::new_v4().to_string())
    }

    /// 添加自定义提示模板
    pub async fn add_prompt_template(&self, template: PromptTemplate) {
        let mut prompt_manager = self.prompt_manager.write().await;
        prompt_manager.add_template(template);
    }

    /// 创建自定义模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub async fn create_custom_template(&self, template: PromptTemplate) -> Result<()> {
        let mut prompt_manager = self.prompt_manager.write().await;
        prompt_manager.create_custom_template(template)
    }

    /// 更新模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub async fn update_template(&self, template: PromptTemplate) -> Result<()> {
        let mut prompt_manager = self.prompt_manager.write().await;
        prompt_manager.update_template(template)
    }

    /// 删除模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub async fn delete_template(&self, template_id: &str) -> Result<()> {
        let mut prompt_manager = self.prompt_manager.write().await;
        prompt_manager.delete_template(template_id)
    }

    /// 重新加载默认模板（清理缓存）
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    #[allow(dead_code)] // 预留的管理功能，暂未直接使用
    pub async fn reload_default_templates(&self) -> Result<()> {
        let mut prompt_manager = self.prompt_manager.write().await;
        prompt_manager.reload_default_templates()
    }

    /// 清理所有缓存和配置文件
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    #[allow(dead_code)] // 预留的管理功能，暂未直接使用
    pub async fn clear_all_cache(&self) -> Result<()> {
        // 清理对话记录
        {
            let mut logger = self.conversation_logger.write().await;
            logger.clear_all_records()?;
        }

        // 重新加载默认模板
        self.reload_default_templates().await?;

        Ok(())
    }

    /// 获取自定义模板列表
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub async fn get_custom_templates(&self) -> Vec<PromptTemplate> {
        let prompt_manager = self.prompt_manager.read().await;
        prompt_manager
            .get_custom_templates()
            .into_iter()
            .cloned()
            .collect()
    }

    /// 获取默认模板列表
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub async fn get_default_templates(&self) -> Vec<PromptTemplate> {
        let prompt_manager = self.prompt_manager.read().await;
        prompt_manager
            .get_default_templates()
            .into_iter()
            .cloned()
            .collect()
    }

    /// 获取响应缓存统计信息
    /// 作者：Evilek
    /// 编写日期：2025-01-19
    pub async fn get_cache_stats(&self) -> std::collections::HashMap<String, u64> {
        let cache = self.response_cache.read().await;
        cache.get_stats()
    }

    /// 清理所有响应缓存
    /// 作者：Evilek
    /// 编写日期：2025-01-19
    pub async fn clear_response_cache(&self) -> Result<()> {
        let mut cache = self.response_cache.write().await;
        cache.clear_all()
    }

    /// 设置缓存过期时间（秒）
    /// 作者：Evilek
    /// 编写日期：2025-01-19
    pub async fn set_cache_max_age(&self, seconds: u64) {
        let mut cache = self.response_cache.write().await;
        cache.set_max_age(seconds);
    }
}
