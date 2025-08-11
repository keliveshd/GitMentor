use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::{Duration, Instant};

use crate::core::ai_provider::*;
use crate::core::ai_config::OpenAIConfig;

/**
 * OpenAI提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModelInfo {
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    owned_by: String,
}

pub struct OpenAIProvider {
    client: Client,
    config: OpenAIConfig,
}

impl OpenAIProvider {
    pub fn new(config: OpenAIConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))  // 增加到5分钟，避免长响应被截断 - Author: Evilek, Date: 2025-01-10
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }
    
    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        
        if !self.config.api_key.is_empty() {
            headers.insert(
                "Authorization",
                format!("Bearer {}", self.config.api_key).parse().unwrap(),
            );
        }
        
        headers
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn get_id(&self) -> &str {
        "OpenAI"
    }
    
    fn get_name(&self) -> &str {
        "OpenAI"
    }
    
    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| OpenAIMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            }).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(false),
        };
        
        let response = self.client
            .post(&format!("{}/chat/completions", self.config.base_url))
            .headers(self.get_headers())
            .json(&openai_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }
        
        let openai_response: OpenAIResponse = response.json().await?;
        
        if let Some(choice) = openai_response.choices.first() {
            // 使用推理内容解析工具处理响应 - Author: Evilek, Date: 2025-01-10
            use crate::core::ai_provider::ReasoningParser;

            Ok(ReasoningParser::create_response(
                choice.message.content.clone(),
                openai_response.model,
                openai_response.usage.map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
                choice.finish_reason.clone(),
            ))
        } else {
            Err(anyhow::anyhow!("No response from OpenAI"))
        }
    }
    
    async fn get_models(&self) -> Result<Vec<AIModel>> {
        let response = self.client
            .get(&format!("{}/models", self.config.base_url))
            .headers(self.get_headers())
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to get models: {}", error_text));
        }
        
        let models_response: OpenAIModelsResponse = response.json().await?;
        
        let models = models_response.data.into_iter()
            .filter(|model| model.id.contains("gpt") || model.id.contains("text"))
            .map(|model| AIModel {
                id: model.id.clone(),
                name: model.id,
                max_tokens: Some(4096), // 默认值，实际应根据模型调整
                provider: "OpenAI".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            })
            .collect();
        
        Ok(models)
    }
    
    async fn test_connection(&self) -> Result<ConnectionTestResult> {
        let start_time = Instant::now();
        
        match self.get_models().await {
            Ok(models) => {
                let latency = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTestResult {
                    success: true,
                    message: "连接成功".to_string(),
                    latency_ms: Some(latency),
                    model_count: Some(models.len()),
                })
            }
            Err(e) => {
                Ok(ConnectionTestResult {
                    success: false,
                    message: format!("连接失败: {}", e),
                    latency_ms: None,
                    model_count: None,
                })
            }
        }
    }
    
    async fn is_available(&self) -> bool {
        !self.config.api_key.is_empty() && !self.config.base_url.is_empty()
    }
    
    async fn refresh_models(&self) -> Result<Vec<AIModel>> {
        self.get_models().await
    }
}
