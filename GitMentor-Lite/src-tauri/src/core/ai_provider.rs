use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::collections::HashMap;

/**
 * AI提供商接口定义
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub id: String,
    pub name: String,
    pub max_tokens: Option<u32>,
    pub provider: String,
    pub default: Option<bool>,
    pub hidden: Option<bool>,
    pub capabilities: Option<ModelCapabilities>,
    pub cost: Option<ModelCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub streaming: Option<bool>,
    pub function_calling: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
    pub model_count: Option<usize>,
}

/// AI提供商接口
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// 获取提供商ID
    fn get_id(&self) -> &str;
    
    /// 获取提供商名称
    fn get_name(&self) -> &str;
    
    /// 生成提交消息
    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse>;
    
    /// 获取可用模型列表
    async fn get_models(&self) -> Result<Vec<AIModel>>;
    
    /// 测试连接
    async fn test_connection(&self) -> Result<ConnectionTestResult>;
    
    /// 检查服务是否可用
    async fn is_available(&self) -> bool;
    
    /// 刷新模型列表
    async fn refresh_models(&self) -> Result<Vec<AIModel>>;
    
    /// 计算token数量（可选）
    async fn count_tokens(&self, request: &AIRequest) -> Result<u32> {
        // 默认实现：简单估算
        let total_chars: usize = request.messages.iter()
            .map(|msg| msg.content.len())
            .sum();
        Ok((total_chars / 4) as u32) // 粗略估算：4个字符约等于1个token
    }
}

/// AI提供商工厂
pub struct AIProviderFactory {
    providers: HashMap<String, Box<dyn AIProvider>>,
}

impl AIProviderFactory {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    
    /// 注册提供商
    pub fn register_provider(&mut self, provider: Box<dyn AIProvider>) {
        let id = provider.get_id().to_string();
        self.providers.insert(id, provider);
    }
    
    /// 获取提供商
    pub fn get_provider(&self, provider_id: &str) -> Option<&dyn AIProvider> {
        self.providers.get(provider_id).map(|p| p.as_ref())
    }
    
    /// 获取所有提供商ID
    pub fn get_provider_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    /// 获取所有提供商信息
    pub fn get_providers_info(&self) -> Vec<(String, String)> {
        self.providers.iter()
            .map(|(id, provider)| (id.clone(), provider.get_name().to_string()))
            .collect()
    }

    /// 生成提交消息
    pub async fn generate_commit(&self, provider_id: &str, request: &AIRequest) -> Result<AIResponse> {
        if let Some(provider) = self.get_provider(provider_id) {
            provider.generate_commit(request).await
        } else {
            Err(anyhow::anyhow!("Provider '{}' not found", provider_id))
        }
    }

    /// 获取模型列表
    pub async fn get_models(&self, provider_id: &str) -> Result<Vec<AIModel>> {
        if let Some(provider) = self.get_provider(provider_id) {
            provider.get_models().await
        } else {
            Err(anyhow::anyhow!("Provider '{}' not found", provider_id))
        }
    }

    /// 测试连接
    pub async fn test_connection(&self, provider_id: &str) -> Result<ConnectionTestResult> {
        if let Some(provider) = self.get_provider(provider_id) {
            provider.test_connection().await
        } else {
            Err(anyhow::anyhow!("Provider '{}' not found", provider_id))
        }
    }

    /// 刷新模型列表
    pub async fn refresh_models(&self, provider_id: &str) -> Result<Vec<AIModel>> {
        if let Some(provider) = self.get_provider(provider_id) {
            provider.refresh_models().await
        } else {
            Err(anyhow::anyhow!("Provider '{}' not found", provider_id))
        }
    }

    /// 检查是否可用
    pub async fn is_available(&self, provider_id: &str) -> bool {
        if let Some(provider) = self.get_provider(provider_id) {
            provider.is_available().await
        } else {
            false
        }
    }
}

impl Default for AIProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}
