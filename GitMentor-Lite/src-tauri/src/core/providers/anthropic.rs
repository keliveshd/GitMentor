use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::{Duration, Instant};

use crate::core::ai_provider::*;
use crate::core::ai_config::AnthropicConfig;

/**
 * Anthropic (Claude) 提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: Option<f32>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    model: String,
    usage: Option<AnthropicUsage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

pub struct AnthropicProvider {
    client: Client,
    config: AnthropicConfig,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
    
    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        
        if !self.config.api_key.is_empty() {
            headers.insert(
                "x-api-key",
                self.config.api_key.parse().unwrap(),
            );
        }
        
        headers
    }
    
    fn get_available_models() -> Vec<AIModel> {
        vec![
            AIModel {
                id: "claude-3-5-sonnet-20241022".to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                max_tokens: Some(8192),
                provider: "Anthropic".to_string(),
                default: Some(true),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "claude-3-haiku-20240307".to_string(),
                name: "Claude 3 Haiku".to_string(),
                max_tokens: Some(4096),
                provider: "Anthropic".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(false),
                }),
                cost: None,
            },
            AIModel {
                id: "claude-3-opus-20240229".to_string(),
                name: "Claude 3 Opus".to_string(),
                max_tokens: Some(4096),
                provider: "Anthropic".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
        ]
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    fn get_id(&self) -> &str {
        "Anthropic"
    }
    
    fn get_name(&self) -> &str {
        "Anthropic (Claude)"
    }
    
    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let anthropic_request = AnthropicRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| AnthropicMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            }).collect(),
            max_tokens: request.max_tokens.unwrap_or(1024),
            temperature: request.temperature,
            stream: Some(false),
        };
        
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .headers(self.get_headers())
            .json(&anthropic_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }
        
        let anthropic_response: AnthropicResponse = response.json().await?;
        
        if let Some(content) = anthropic_response.content.first() {
            // 使用推理内容解析工具处理响应 - Author: Evilek, Date: 2025-01-10
            use crate::core::ai_provider::ReasoningParser;

            Ok(ReasoningParser::create_response(
                content.text.clone(),
                anthropic_response.model,
                anthropic_response.usage.map(|u| TokenUsage {
                    prompt_tokens: u.input_tokens,
                    completion_tokens: u.output_tokens,
                    total_tokens: u.input_tokens + u.output_tokens,
                }),
                anthropic_response.stop_reason,
            ))
        } else {
            Err(anyhow::anyhow!("No response from Anthropic"))
        }
    }
    
    async fn get_models(&self) -> Result<Vec<AIModel>> {
        Ok(Self::get_available_models())
    }
    
    async fn test_connection(&self) -> Result<ConnectionTestResult> {
        let start_time = Instant::now();
        
        // 创建一个简单的测试请求
        let test_request = AIRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            model: "claude-3-haiku-20240307".to_string(),
            temperature: Some(0.1),
            max_tokens: Some(10),
            stream: Some(false),
        };
        
        match self.generate_commit(&test_request).await {
            Ok(_) => {
                let latency = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTestResult {
                    success: true,
                    message: "连接成功".to_string(),
                    latency_ms: Some(latency),
                    model_count: Some(Self::get_available_models().len()),
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
        !self.config.api_key.is_empty()
    }
    
    async fn refresh_models(&self) -> Result<Vec<AIModel>> {
        Ok(Self::get_available_models())
    }
}
