use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::core::ai_config::DeepseekConfig;
use crate::core::ai_provider::*;

/**
 * Deepseek 提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct DeepseekRequest {
    model: String,
    messages: Vec<DeepseekMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DeepseekMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepseekResponse {
    choices: Vec<DeepseekChoice>,
    usage: Option<DeepseekUsage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct DeepseekChoice {
    message: DeepseekResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeepseekResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepseekUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct DeepseekProvider {
    client: Client,
    config: DeepseekConfig,
}

impl DeepseekProvider {
    pub fn new(config: DeepseekConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 增加到5分钟，避免长响应被截断 - Author: Evilek, Date: 2025-01-10
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
                id: "deepseek-chat".to_string(),
                name: "DeepSeek Chat".to_string(),
                max_tokens: Some(32768),
                provider: "Deepseek".to_string(),
                default: Some(true),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "deepseek-coder".to_string(),
                name: "DeepSeek Coder".to_string(),
                max_tokens: Some(32768),
                provider: "Deepseek".to_string(),
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
impl AIProvider for DeepseekProvider {
    fn get_id(&self) -> &str {
        "Deepseek"
    }

    fn get_name(&self) -> &str {
        "Deepseek"
    }

    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let deepseek_request = DeepseekRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|msg| DeepseekMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(false),
        };

        let response = self
            .client
            .post("https://api.deepseek.com/chat/completions")
            .headers(self.get_headers())
            .json(&deepseek_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Deepseek API error: {}", error_text));
        }

        let deepseek_response: DeepseekResponse = response.json().await?;

        if let Some(choice) = deepseek_response.choices.first() {
            // 使用推理内容解析工具处理响应 - Author: Evilek, Date: 2025-01-10
            use crate::core::ai_provider::ReasoningParser;

            Ok(ReasoningParser::create_response(
                choice.message.content.clone(),
                deepseek_response.model,
                deepseek_response.usage.map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                }),
                choice.finish_reason.clone(),
            ))
        } else {
            Err(anyhow::anyhow!("No response from Deepseek"))
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
            model: "deepseek-chat".to_string(),
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
            Err(e) => Ok(ConnectionTestResult {
                success: false,
                message: format!("连接失败: {}", e),
                latency_ms: None,
                model_count: None,
            }),
        }
    }

    async fn is_available(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    async fn refresh_models(&self) -> Result<Vec<AIModel>> {
        Ok(Self::get_available_models())
    }
}
