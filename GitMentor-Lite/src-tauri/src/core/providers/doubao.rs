use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::{Duration, Instant};

use crate::core::ai_provider::*;
use crate::core::ai_config::DoubaoConfig;

/**
 * 字节跳动豆包（Doubao）提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct DoubaoRequest {
    model: String,
    messages: Vec<DoubaoMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DoubaoMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct DoubaoResponse {
    choices: Vec<DoubaoChoice>,
    usage: Option<DoubaoUsage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct DoubaoChoice {
    message: DoubaoResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DoubaoResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct DoubaoUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct DoubaoProvider {
    client: Client,
    config: DoubaoConfig,
}

impl DoubaoProvider {
    pub fn new(config: DoubaoConfig) -> Self {
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
    
    fn get_available_models() -> Vec<AIModel> {
        vec![
            AIModel {
                id: "doubao-lite-4k".to_string(),
                name: "豆包-Lite-4K".to_string(),
                max_tokens: Some(4096),
                provider: "Doubao".to_string(),
                default: Some(true),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(false),
                }),
                cost: None,
            },
            AIModel {
                id: "doubao-pro-4k".to_string(),
                name: "豆包-Pro-4K".to_string(),
                max_tokens: Some(4096),
                provider: "Doubao".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "doubao-pro-32k".to_string(),
                name: "豆包-Pro-32K".to_string(),
                max_tokens: Some(32768),
                provider: "Doubao".to_string(),
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
impl AIProvider for DoubaoProvider {
    fn get_id(&self) -> &str {
        "Doubao"
    }
    
    fn get_name(&self) -> &str {
        "字节跳动豆包"
    }
    
    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let doubao_request = DoubaoRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| DoubaoMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            }).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(false),
        };
        
        let response = self.client
            .post("https://ark.cn-beijing.volces.com/api/v3/chat/completions")
            .headers(self.get_headers())
            .json(&doubao_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Doubao API error: {}", error_text));
        }
        
        let doubao_response: DoubaoResponse = response.json().await?;
        
        if let Some(choice) = doubao_response.choices.first() {
            // 使用推理内容解析工具处理响应 - Author: Evilek, Date: 2025-01-10
            use crate::core::ai_provider::ReasoningParser;

            Ok(ReasoningParser::create_response(
                choice.message.content.clone(),
                doubao_response.model,
                doubao_response.usage.map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
                choice.finish_reason.clone(),
            ))
        } else {
            Err(anyhow::anyhow!("No response from Doubao"))
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
            model: "doubao-lite-4k".to_string(),
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
