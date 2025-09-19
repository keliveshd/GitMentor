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
        
        let url = &format!("{}/chat/completions", self.config.base_url);
        println!("🔍 [OpenAI] 请求URL: {}", url);
        println!("🔍 [OpenAI] 请求头: {:?}", self.get_headers());
        println!("🔍 [OpenAI] 请求模型: {}", openai_request.model);

        let response = self.client
            .post(url)
            .headers(self.get_headers())
            .json(&openai_request)
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();

        println!("🔍 [OpenAI] HTTP状态码: {}", status);
        println!("🔍 [OpenAI] 响应头: {:?}", headers);

        // 检查响应内容是否为空
        let response_text = response.text().await?;
        if response_text.trim().is_empty() {
            return Err(anyhow::anyhow!("OpenAI API returned empty response"));
        }

        if !status.is_success() {
            println!("❌ [OpenAI] API错误响应: {}", response_text);
            return Err(anyhow::anyhow!("OpenAI API error: {}", response_text));
        }

        // 处理可能的SSE响应格式
        let final_json = if response_text.starts_with("data: ") {
            println!("🔍 [OpenAI] SSE响应内容: {}", response_text);

            // 处理流式响应，聚合所有内容
            let lines: Vec<&str> = response_text.lines().collect();
            let mut final_content = String::new();
            let mut final_role = None;
            let mut model_name = String::new();
            let mut usage_info = None;

            for line in lines {
                if line.starts_with("data: ") && !line.trim_end_matches('\n').ends_with("[DONE]") {
                    let json_str = &line[6..]; // 移除 "data: " 前缀
                    if !json_str.trim().is_empty() {
                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(json_str) {
                            // 保存模型名称
                            if let Some(model) = chunk.get("model").and_then(|m| m.as_str()) {
                                model_name = model.to_string();
                            }

                            // 保存使用量信息
                            if let Some(usage) = chunk.get("usage") {
                                usage_info = Some(usage.clone());
                            }

                            // 处理choices数组
                            if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
                                if let Some(choice) = choices.first() {
                                    // 处理角色信息（通常只在第一个delta中）
                                    if let Some(delta) = choice.get("delta") {
                                        if let Some(role) = delta.get("role").and_then(|r| r.as_str()) {
                                            final_role = Some(role.to_string());
                                        }
                                        // 累积内容
                                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                            final_content.push_str(content);
                                        }
                                    }
                                    // 处理完整的message（非delta格式）
                                    else if let Some(message) = choice.get("message") {
                                        if let Some(role) = message.get("role").and_then(|r| r.as_str()) {
                                            final_role = Some(role.to_string());
                                        }
                                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                            final_content = content.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if final_content.is_empty() {
                return Err(anyhow::anyhow!("No content found in SSE response. Response: {}", response_text));
            }

            // 构造最终的OpenAI响应格式
            let final_response = serde_json::json!({
                "model": model_name,
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": final_role.unwrap_or_else(|| "assistant".to_string()),
                        "content": final_content
                    },
                    "finish_reason": "stop"
                }],
                "usage": usage_info.unwrap_or(serde_json::json!({
                    "prompt_tokens": 0,
                    "completion_tokens": 0,
                    "total_tokens": 0
                }))
            });

            println!("✅ [OpenAI] 构造的最终响应: {}", final_response);
            final_response.to_string()
        } else {
            response_text.clone()
        };

        // 解析JSON响应
        let openai_response: OpenAIResponse = serde_json::from_str(&final_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse OpenAI response: {}. Response text: {}", e, if final_json.len() > 200 { &final_json[..200] } else { &final_json }))?;
        
        if let Some(choice) = openai_response.choices.first() {
            // 使用推理内容解析工具处理响应 - Author: Evilek, Date: 2025-01-10
            use crate::core::ai_provider::ReasoningParser;

            // 清理响应内容，移除思考过程 - Author: Evilek, Date: 2025-01-19
            use crate::core::response_cleaner::ResponseCleaner;
            let cleaned_content = ResponseCleaner::clean_commit_message(&choice.message.content);

            println!("✅ [OpenAI] 原始响应长度: {}, 清理后长度: {}", choice.message.content.len(), cleaned_content.len());

            Ok(ReasoningParser::create_response(
                cleaned_content,
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
        let url = &format!("{}/models", self.config.base_url);

        let response = self.client
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
            return Err(anyhow::anyhow!("OpenAI API returned empty response for models"));
        }

        // 解析JSON响应
        let models_response: OpenAIModelsResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse OpenAI models response: {}. Response text: {}", e, if response_text.len() > 200 { &response_text[..200] } else { &response_text }))?;

        let models: Vec<AIModel> = models_response.data.into_iter()
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
