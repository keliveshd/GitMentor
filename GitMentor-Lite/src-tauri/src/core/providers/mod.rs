pub mod anthropic;
pub mod dashscope;
pub mod deepseek;
pub mod doubao;
pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod zhipu;

use crate::core::ai_config::AIConfig;
/**
 * AI提供商实现模块
 * 作者：Evilek
 * 编写日期：2025-07-25
 */
use crate::core::ai_provider::AIProviderFactory;

pub fn create_provider_factory(config: &AIConfig) -> AIProviderFactory {
    let mut factory = AIProviderFactory::new();

    // 注册OpenAI提供商
    factory.register_provider(Box::new(openai::OpenAIProvider::new(
        config.providers.openai.clone(),
        config.advanced.timeout,
    )));

    // 注册Ollama提供商
    factory.register_provider(Box::new(ollama::OllamaProvider::new(
        config.providers.ollama.clone(),
    )));

    // 注册智谱AI提供商
    factory.register_provider(Box::new(zhipu::ZhipuProvider::new(
        config.providers.zhipu.clone(),
    )));

    // 注册Anthropic提供商
    factory.register_provider(Box::new(anthropic::AnthropicProvider::new(
        config.providers.anthropic.clone(),
    )));

    // 注册DashScope提供商
    factory.register_provider(Box::new(dashscope::DashScopeProvider::new(
        config.providers.dashscope.clone(),
    )));

    // 注册Doubao提供商
    factory.register_provider(Box::new(doubao::DoubaoProvider::new(
        config.providers.doubao.clone(),
    )));

    // 注册Gemini提供商
    factory.register_provider(Box::new(gemini::GeminiProvider::new(
        config.providers.gemini.clone(),
    )));

    // 注册Deepseek提供商
    factory.register_provider(Box::new(deepseek::DeepseekProvider::new(
        config.providers.deepseek.clone(),
    )));

    factory
}
