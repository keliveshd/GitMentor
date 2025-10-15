use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::core::ai_config::DashScopeConfig;
use crate::core::ai_provider::*;

/**
 * 阿里云通义千问（DashScope）提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct DashScopeRequest {
    model: String,
    input: DashScopeInput,
    parameters: Option<DashScopeParameters>,
}

#[derive(Debug, Serialize)]
struct DashScopeInput {
    messages: Vec<DashScopeMessage>,
}

#[derive(Debug, Serialize)]
struct DashScopeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct DashScopeParameters {
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DashScopeResponse {
    output: DashScopeOutput,
    usage: Option<DashScopeUsage>,
    #[allow(dead_code)]
    request_id: String,
}

#[derive(Debug, Deserialize)]
struct DashScopeOutput {
    text: Option<String>,
    choices: Option<Vec<DashScopeChoice>>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashScopeChoice {
    message: DashScopeResponseMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashScopeResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct DashScopeUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

pub struct DashScopeProvider {
    client: Client,
    config: DashScopeConfig,
}

impl DashScopeProvider {
    pub fn new(config: DashScopeConfig) -> Self {
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
                id: "qwen-turbo".to_string(),
                name: "通义千问-Turbo".to_string(),
                max_tokens: Some(8192),
                provider: "DashScope".to_string(),
                default: Some(true),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(false),
                }),
                cost: None,
            },
            AIModel {
                id: "qwen-plus".to_string(),
                name: "通义千问-Plus".to_string(),
                max_tokens: Some(32768),
                provider: "DashScope".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "qwen-max".to_string(),
                name: "通义千问-Max".to_string(),
                max_tokens: Some(8192),
                provider: "DashScope".to_string(),
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
impl AIProvider for DashScopeProvider {
    fn get_id(&self) -> &str {
        "DashScope"
    }

    fn get_name(&self) -> &str {
        "阿里云通义千问"
    }

    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let dashscope_request = DashScopeRequest {
            model: request.model.clone(),
            input: DashScopeInput {
                messages: request
                    .messages
                    .iter()
                    .map(|msg| DashScopeMessage {
                        role: msg.role.clone(),
                        content: msg.content.clone(),
                    })
                    .collect(),
            },
            parameters: Some(DashScopeParameters {
                temperature: request.temperature,
                max_tokens: request.max_tokens,
                stream: Some(false),
            }),
        };

        let response = self
            .client
            .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
            .headers(self.get_headers())
            .json(&dashscope_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("DashScope API error: {}", error_text));
        }

        let dashscope_response: DashScopeResponse = response.json().await?;

        let content = if let Some(text) = dashscope_response.output.text {
            text
        } else if let Some(choices) = dashscope_response.output.choices {
            if let Some(choice) = choices.first() {
                choice.message.content.clone()
            } else {
                return Err(anyhow::anyhow!("No response from DashScope"));
            }
        } else {
            return Err(anyhow::anyhow!("No response from DashScope"));
        };

        // 使用推理内容解析工具处理响应 - Author: Evilek, Date: 2025-01-10
        use crate::core::ai_provider::ReasoningParser;

        Ok(ReasoningParser::create_response(
            content,
            request.model.clone(),
            dashscope_response.usage.map(|u| TokenUsage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: u.total_tokens,
            }),
            dashscope_response.output.finish_reason,
        ))
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
            model: "qwen-turbo".to_string(),
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
