use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::{Duration, Instant};

use crate::core::ai_provider::*;
use crate::core::ai_config::OllamaConfig;

/**
 * Ollama提供商实现
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
    num_predict: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaResponseMessage,
    done: bool,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    size: u64,
    digest: String,
    modified_at: String,
}

pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // Ollama可能需要更长时间
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client, config }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    fn get_id(&self) -> &str {
        "Ollama"
    }
    
    fn get_name(&self) -> &str {
        "Ollama"
    }
    
    async fn generate_commit(&self, request: &AIRequest) -> Result<AIResponse> {
        let ollama_request = OllamaRequest {
            model: request.model.clone(),
            messages: request.messages.iter().map(|msg| OllamaMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            }).collect(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            }),
        };
        
        let response = self.client
            .post(&format!("{}/api/chat", self.config.base_url))
            .header("Content-Type", "application/json")
            .json(&ollama_request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }
        
        let ollama_response: OllamaResponse = response.json().await?;
        
        Ok(AIResponse {
            content: ollama_response.message.content,
            model: ollama_response.model,
            usage: None, // Ollama通常不返回token使用信息
            finish_reason: Some("stop".to_string()),
        })
    }
    
    async fn get_models(&self) -> Result<Vec<AIModel>> {
        let response = self.client
            .get(&format!("{}/api/tags", self.config.base_url))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to get Ollama models: {}", error_text));
        }
        
        let models_response: OllamaModelsResponse = response.json().await?;
        
        let models = models_response.models.into_iter()
            .map(|model| {
                // 从模型名称中提取基础名称（去掉版本标签）
                let base_name = model.name.split(':').next().unwrap_or(&model.name);
                
                AIModel {
                    id: model.name.clone(),
                    name: base_name.to_string(),
                    max_tokens: Some(4096), // Ollama模型的默认上下文长度
                    provider: "Ollama".to_string(),
                    default: Some(false),
                    hidden: Some(false),
                    capabilities: Some(ModelCapabilities {
                        streaming: Some(true),
                        function_calling: Some(false),
                    }),
                    cost: None,
                }
            })
            .collect();
        
        Ok(models)
    }
    
    async fn test_connection(&self) -> Result<ConnectionTestResult> {
        let start_time = Instant::now();
        
        // 尝试获取模型列表来测试连接
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
        !self.config.base_url.is_empty()
    }
    
    async fn refresh_models(&self) -> Result<Vec<AIModel>> {
        self.get_models().await
    }
}
