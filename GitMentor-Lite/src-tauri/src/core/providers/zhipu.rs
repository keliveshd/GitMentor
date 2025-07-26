use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::{Duration, Instant};

use crate::core::ai_provider::*;
use crate::core::ai_config::ZhipuConfig;

/**
 * 智谱AI提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct ZhipuRequest {
    model: String,
    messages: Vec<ZhipuMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ZhipuMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ZhipuResponse {
    choices: Vec<ZhipuChoice>,
    usage: Option<ZhipuUsage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct ZhipuChoice {
    message: ZhipuResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZhipuResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ZhipuUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct ZhipuProvider {
    client: Client,
    config: ZhipuConfig,
}

impl ZhipuProvider {
    pub fn new(config: ZhipuConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
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
                id: "glm-4".to_string(),
                name: "GLM-4".to_string(),
                max_tokens: Some(8192),
                provider: "Zhipu".to_string(),
                default: Some(true),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "glm-4-flash".to_string(),
                name: "GLM-4-Flash".to_string(),
                max_tokens: Some(8192),
                provider: "Zhipu".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "glm-3-turbo".to_string(),
                name: "GLM-3-Turbo".to_string(),
                max_tokens: Some(8192),
                provider: "Zhipu".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(false),
                }),
                cost: None,
            },
        ]
    }
}

#[async_trait]
impl AIProvider for ZhipuProvider {
    fn get_id(&self) -> &str {
        "Zhipu"
    }
    
    fn get_name(&self) -> &str {
        "智谱AI"
    }
    
    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let zhipu_request = ZhipuRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| ZhipuMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            }).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(false),
        };
        
        let response = self.client
            .post("https://open.bigmodel.cn/api/paas/v4/chat/completions")
            .headers(self.get_headers())
            .json(&zhipu_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Zhipu API error: {}", error_text));
        }
        
        let zhipu_response: ZhipuResponse = response.json().await?;
        
        if let Some(choice) = zhipu_response.choices.first() {
            Ok(AIResponse {
                content: choice.message.content.clone(),
                model: zhipu_response.model,
                usage: zhipu_response.usage.map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
                finish_reason: choice.finish_reason.clone(),
            })
        } else {
            Err(anyhow::anyhow!("No response from Zhipu AI"))
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
            model: "glm-4-flash".to_string(),
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
