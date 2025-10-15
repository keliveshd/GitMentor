use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::core::ai_config::OpenAIConfig;
use crate::core::ai_provider::*;

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
        // 所有模型都使用流式请求以避免超时
        let openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|msg| OpenAIMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(true), // 强制使用流式请求
        };

        let url = &format!("{}/chat/completions", self.config.base_url);
        println!("🔍 [OpenAI] 请求URL: {}", url);
        println!("🔍 [OpenAI] 请求头: {:?}", self.get_headers());
        println!("🔍 [OpenAI] 请求模型: {} (流式: 是)", openai_request.model);

        println!("🔍 [OpenAI] 开始流式请求...");

        let mut response = self
            .client
            .post(url)
            .headers(self.get_headers())
            .json(&openai_request)
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();

        println!("🔍 [OpenAI] 流式请求HTTP状态码: {}", status);
        println!("🔍 [OpenAI] 流式请求响应头: {:?}", headers);

        if !status.is_success() {
            let error_text = response.text().await?;
            println!("❌ [OpenAI] 流式请求错误: {}", error_text);
            return Err(anyhow::anyhow!("OpenAI streaming error: {}", error_text));
        }

        // 读取流式响应
        let mut final_content = String::new();
        let mut model_name = String::new();
        let mut usage_info = None;

        while let Some(chunk) = response.chunk().await? {
            let chunk_str = String::from_utf8_lossy(&chunk);

            // 解析SSE格式
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..]; // 移除 "data: " 前缀

                    if data.trim() == "[DONE]" {
                        println!("🔍 [OpenAI] 流式响应完成");
                        continue;
                    }

                    if let Ok(chunk_data) = serde_json::from_str::<serde_json::Value>(data) {
                        // 保存模型名称
                        if let Some(model) = chunk_data.get("model").and_then(|m| m.as_str()) {
                            model_name = model.to_string();
                        }

                        // 处理choices数组
                        if let Some(choices) = chunk_data.get("choices").and_then(|c| c.as_array())
                        {
                            if let Some(choice) = choices.first() {
                                // 处理delta内容
                                if let Some(delta) = choice.get("delta") {
                                    // 累积内容
                                    if let Some(content) =
                                        delta.get("content").and_then(|c| c.as_str())
                                    {
                                        final_content.push_str(content);
                                        // 实时打印接收到的内容（用于调试）
                                        print!("{}", content);
                                        std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                    }
                                }

                                // 处理使用量信息（通常在最后一个chunk中）
                                if let Some(usage) = chunk_data.get("usage") {
                                    usage_info = Some(usage.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        println!(
            "\n🔍 [OpenAI] 流式接收完成，总长度: {}",
            final_content.len()
        );

        // 如果没有收到任何内容，返回错误
        if final_content.is_empty() {
            return Err(anyhow::anyhow!(
                "No content received from streaming response"
            ));
        }

        // 首先使用 ReasoningParser 分离思考内容和实际内容
        let (actual_content, reasoning_content) =
            crate::core::ai_provider::ReasoningParser::parse_content(&final_content);

        // 如果有思考内容，打印日志
        if let Some(ref reasoning) = reasoning_content {
            println!("🔍 [DEBUG] 提取到思考内容，长度: {}", reasoning.len());
        }

        // 然后对实际内容进行进一步清理
        use crate::core::response_cleaner::ResponseCleaner;
        let cleaned_content = ResponseCleaner::clean_commit_message(&actual_content);

        println!(
            "🔍 [DEBUG] 原始响应长度: {}, 移除思考后长度: {}, 清理后长度: {}",
            final_content.len(),
            actual_content.len(),
            cleaned_content.len()
        );

        // 构造AI响应，直接使用已清理的内容
        Ok(AIResponse {
            content: cleaned_content,
            reasoning_content,
            model: model_name,
            usage: usage_info.map(|u| TokenUsage {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            }),
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn get_models(&self) -> Result<Vec<AIModel>> {
        let url = &format!("{}/models", self.config.base_url);

        let response = self
            .client
            .get(url)
            .headers(self.get_headers())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to get models: {}", error_text));
        }

        // 检查响应内容是否为空
        let response_text = response.text().await?;
        if response_text.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "OpenAI API returned empty response for models"
            ));
        }

        // 解析JSON响应
        let models_response: OpenAIModelsResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse OpenAI models response: {}. Response text: {}",
                    e,
                    if response_text.len() > 200 {
                        &response_text[..200]
                    } else {
                        &response_text
                    }
                )
            })?;

        let models: Vec<AIModel> = models_response
            .data
            .into_iter()
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
            Err(e) => Ok(ConnectionTestResult {
                success: false,
                message: format!("连接失败: {}", e),
                latency_ms: None,
                model_count: None,
            }),
        }
    }

    async fn is_available(&self) -> bool {
        !self.config.api_key.is_empty() && !self.config.base_url.is_empty()
    }

    async fn refresh_models(&self) -> Result<Vec<AIModel>> {
        self.get_models().await
    }
}
