use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::{Duration, Instant};

use crate::core::ai_provider::*;
use crate::core::ai_config::GeminiConfig;

/**
 * Google Gemini 提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    role: Option<String>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    temperature: Option<f32>,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}

pub struct GeminiProvider {
    client: Client,
    config: GeminiConfig,
}

impl GeminiProvider {
    pub fn new(config: GeminiConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))  // 增加到5分钟，避免长响应被截断 - Author: Evilek, Date: 2025-01-10
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }
    
    fn get_available_models() -> Vec<AIModel> {
        vec![
            AIModel {
                id: "gemini-1.5-flash".to_string(),
                name: "Gemini 1.5 Flash".to_string(),
                max_tokens: Some(8192),
                provider: "Gemini".to_string(),
                default: Some(true),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "gemini-1.5-pro".to_string(),
                name: "Gemini 1.5 Pro".to_string(),
                max_tokens: Some(8192),
                provider: "Gemini".to_string(),
                default: Some(false),
                hidden: Some(false),
                capabilities: Some(ModelCapabilities {
                    streaming: Some(true),
                    function_calling: Some(true),
                }),
                cost: None,
            },
            AIModel {
                id: "gemini-pro".to_string(),
                name: "Gemini Pro".to_string(),
                max_tokens: Some(8192),
                provider: "Gemini".to_string(),
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
    
    fn convert_messages_to_contents(&self, messages: &[ChatMessage]) -> Vec<GeminiContent> {
        messages.iter().map(|msg| {
            let role = match msg.role.as_str() {
                "system" => None, // Gemini doesn't have system role, merge with user
                "user" => Some("user".to_string()),
                "assistant" => Some("model".to_string()),
                _ => Some("user".to_string()),
            };
            
            GeminiContent {
                parts: vec![GeminiPart {
                    text: msg.content.clone(),
                }],
                role,
            }
        }).collect()
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    fn get_id(&self) -> &str {
        "Gemini"
    }
    
    fn get_name(&self) -> &str {
        "Google Gemini"
    }
    
    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let contents = self.convert_messages_to_contents(&request.messages);
        
        let gemini_request = GeminiRequest {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            }),
        };
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            request.model, self.config.api_key
        );
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&gemini_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
        }
        
        let gemini_response: GeminiResponse = response.json().await?;
        
        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                // 使用推理内容解析工具处理响应 - Author: Evilek, Date: 2025-01-10
                use crate::core::ai_provider::ReasoningParser;

                Ok(ReasoningParser::create_response(
                    part.text.clone(),
                    request.model.clone(),
                    gemini_response.usage_metadata.map(|u| TokenUsage {
                        prompt_tokens: u.prompt_token_count,
                        completion_tokens: u.candidates_token_count,
                        total_tokens: u.total_token_count,
                    }),
                    candidate.finish_reason.clone(),
                ))
            } else {
                Err(anyhow::anyhow!("No content in Gemini response"))
            }
        } else {
            Err(anyhow::anyhow!("No candidates in Gemini response"))
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
            model: "gemini-1.5-flash".to_string(),
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
