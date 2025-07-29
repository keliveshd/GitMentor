use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::core::ai_provider::{AIProviderFactory, AIRequest, AIResponse, AIModel, ConnectionTestResult};
use crate::core::ai_config::{AIConfig, AIConfigManager};
use crate::core::prompt_manager::{PromptManager, PromptTemplate, CommitContext};
use crate::core::providers::create_provider_factory;

/**
 * AI管理器 - 统一管理所有AI提供商
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

pub struct AIManager {
    config_manager: Arc<RwLock<AIConfigManager>>,
    provider_factory: Arc<RwLock<AIProviderFactory>>,
    prompt_manager: Arc<RwLock<PromptManager>>,
}

impl AIManager {
    pub fn new(config_path: std::path::PathBuf) -> Result<Self> {
        let config_manager = AIConfigManager::new(config_path)?;
        let config = config_manager.get_config().clone();
        let provider_factory = create_provider_factory(&config);
        
        Ok(Self {
            config_manager: Arc::new(RwLock::new(config_manager)),
            provider_factory: Arc::new(RwLock::new(provider_factory)),
            prompt_manager: Arc::new(RwLock::new(PromptManager::new())),
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

    /// 获取指定提供商的模型列表
    pub async fn get_models_for_provider(&self, provider_id: &str) -> Result<Vec<AIModel>> {
        let factory = self.provider_factory.read().await;
        factory.get_models(provider_id).await
    }

    /// 测试指定提供商的连接
    pub async fn test_provider_connection(&self, provider_id: &str) -> Result<ConnectionTestResult> {
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
    pub async fn generate_commit_with_template(
        &self,
        template_id: &str,
        context: CommitContext,
    ) -> Result<AIResponse> {
        let config = self.get_config().await;
        let provider_id = &config.base.provider;

        // 生成消息
        let prompt_manager = self.prompt_manager.read().await;
        let messages = prompt_manager.generate_messages(template_id, &context)?;

        // 获取模板配置
        let (max_tokens, temperature) = prompt_manager.get_template_config(template_id)
            .unwrap_or((Some(200), Some(0.3)));

        let request = AIRequest {
            messages,
            model: config.base.model.clone(),
            temperature,
            max_tokens,
            stream: Some(false),
        };

        let factory = self.provider_factory.read().await;
        factory.generate_commit(provider_id, &request).await
    }

    /// 获取所有可用的提示模板
    pub async fn get_prompt_templates(&self) -> Vec<PromptTemplate> {
        let prompt_manager = self.prompt_manager.read().await;
        prompt_manager.get_all_templates().into_iter().cloned().collect()
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

    /// 获取自定义模板列表
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub async fn get_custom_templates(&self) -> Vec<PromptTemplate> {
        let prompt_manager = self.prompt_manager.read().await;
        prompt_manager.get_custom_templates().into_iter().cloned().collect()
    }

    /// 获取默认模板列表
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub async fn get_default_templates(&self) -> Vec<PromptTemplate> {
        let prompt_manager = self.prompt_manager.read().await;
        prompt_manager.get_default_templates().into_iter().cloned().collect()
    }
}
