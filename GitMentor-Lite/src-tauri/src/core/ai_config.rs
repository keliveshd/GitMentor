use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/**
 * AI配置管理
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub base: BaseConfig,
    pub providers: ProvidersConfig,
    pub features: FeaturesConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    pub language: String,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub openai: OpenAIConfig,
    pub ollama: OllamaConfig,
    pub zhipu: ZhipuConfig,
    pub anthropic: AnthropicConfig,
    pub dashscope: DashScopeConfig,
    pub doubao: DoubaoConfig,
    pub gemini: GeminiConfig,
    pub deepseek: DeepseekConfig,
    pub siliconflow: SiliconflowConfig,
    pub openrouter: OpenRouterConfig,
    pub together: TogetherConfig,
    pub mistral: MistralConfig,
    pub baidu_qianfan: BaiduQianfanConfig,
    pub azure_openai: AzureOpenAIConfig,
    pub cloudflare: CloudflareConfig,
    pub vertexai: VertexAIConfig,
    pub groq: GroqConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZhipuConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashScopeConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubaoConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepseekConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiliconflowConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TogetherConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MistralConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaiduQianfanConfig {
    pub api_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOpenAIConfig {
    pub api_key: String,
    pub endpoint: String,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub api_key: String,
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAIConfig {
    pub project_id: String,
    pub location: String,
    pub credentials_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub enable_emoji: bool,
    pub enable_body: bool,
    pub enable_layered_commit: bool,
    pub use_recent_commits: bool,
    pub enable_streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout: u64,
    pub retry_count: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            base: BaseConfig {
                language: "Simplified Chinese".to_string(),
                provider: "OpenAI".to_string(),
                model: "".to_string(),
            },
            providers: ProvidersConfig {
                openai: OpenAIConfig {
                    api_key: "".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                },
                ollama: OllamaConfig {
                    base_url: "http://localhost:11434".to_string(),
                },
                zhipu: ZhipuConfig {
                    api_key: "".to_string(),
                },
                anthropic: AnthropicConfig {
                    api_key: "".to_string(),
                },
                dashscope: DashScopeConfig {
                    api_key: "".to_string(),
                },
                doubao: DoubaoConfig {
                    api_key: "".to_string(),
                },
                gemini: GeminiConfig {
                    api_key: "".to_string(),
                },
                deepseek: DeepseekConfig {
                    api_key: "".to_string(),
                },
                siliconflow: SiliconflowConfig {
                    api_key: "".to_string(),
                },
                openrouter: OpenRouterConfig {
                    api_key: "".to_string(),
                },
                together: TogetherConfig {
                    api_key: "".to_string(),
                },
                mistral: MistralConfig {
                    api_key: "".to_string(),
                },
                baidu_qianfan: BaiduQianfanConfig {
                    api_key: "".to_string(),
                    secret_key: "".to_string(),
                },
                azure_openai: AzureOpenAIConfig {
                    api_key: "".to_string(),
                    endpoint: "".to_string(),
                    api_version: "2024-02-01".to_string(),
                },
                cloudflare: CloudflareConfig {
                    api_key: "".to_string(),
                    account_id: "".to_string(),
                },
                vertexai: VertexAIConfig {
                    project_id: "".to_string(),
                    location: "us-central1".to_string(),
                    credentials_path: "".to_string(),
                },
                groq: GroqConfig {
                    api_key: "".to_string(),
                },
            },
            features: FeaturesConfig {
                enable_emoji: true,
                enable_body: true,
                enable_layered_commit: true,
                use_recent_commits: true,
                enable_streaming: true,
            },
            advanced: AdvancedConfig {
                temperature: 0.7,
                max_tokens: 2048,
                timeout: 60,
                retry_count: 3,
            },
        }
    }
}

pub struct AIConfigManager {
    config: AIConfig,
    config_path: PathBuf,
}

impl AIConfigManager {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AIConfig::default()
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    pub fn get_config(&self) -> &AIConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: AIConfig) -> Result<()> {
        self.config = config;
        self.save()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_provider_config(&self, provider_id: &str) -> Option<serde_json::Value> {
        let providers_value = serde_json::to_value(&self.config.providers).ok()?;
        providers_value.get(provider_id).cloned()
    }
}
