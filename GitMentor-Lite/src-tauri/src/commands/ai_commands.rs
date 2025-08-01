use tauri::State;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::core::ai_manager::AIManager;
use crate::core::ai_provider::{AIRequest, AIModel, ConnectionTestResult, ChatMessage};
use crate::core::ai_config::AIConfig;
use crate::core::prompt_manager::{PromptTemplate, CommitContext};

/**
 * AI相关的Tauri命令
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateCommitRequest {
    pub selected_files: Vec<String>,
    pub additional_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateCommitResponse {
    pub message: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub model_used: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConnectionRequest {
    pub provider_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetModelsRequest {
    pub provider_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvidersInfoResponse {
    pub providers: Vec<ProviderInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
}

/// 获取AI配置
#[tauri::command]
pub async fn get_ai_config(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<AIConfig, String> {
    let manager = ai_manager.lock().await;
    Ok(manager.get_config().await)
}

/// 更新AI配置
#[tauri::command]
pub async fn update_ai_config(
    ai_manager: State<'_, Mutex<AIManager>>,
    config: AIConfig,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.update_config(config).await
        .map_err(|e| format!("Failed to update AI config: {}", e))
}

/// 获取所有提供商信息
#[tauri::command]
pub async fn get_providers_info(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<ProvidersInfoResponse, String> {
    let manager = ai_manager.lock().await;
    let providers_info = manager.get_providers_info().await;
    
    let mut providers = Vec::new();
    for (id, name) in providers_info {
        let available = manager.is_provider_available(&id).await;
        providers.push(ProviderInfo {
            id,
            name,
            available,
        });
    }
    
    Ok(ProvidersInfoResponse { providers })
}

/// 获取指定提供商的模型列表
#[tauri::command]
pub async fn get_models_for_provider(
    request: GetModelsRequest,
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<AIModel>, String> {
    let manager = ai_manager.lock().await;
    manager.get_models_for_provider(&request.provider_id).await
        .map_err(|e| format!("Failed to get models: {}", e))
}

/// 使用临时配置获取指定提供商的模型列表
#[tauri::command(rename_all = "camelCase")]
pub async fn get_models_with_temp_config(
    provider_id: String,
    temp_config: AIConfig,
) -> Result<Vec<AIModel>, String> {
    use crate::core::providers;

    // 使用临时配置创建提供商工厂
    let factory = providers::create_provider_factory(&temp_config);

    // 获取模型列表
    factory.get_models(&provider_id).await
        .map_err(|e| format!("Failed to get models: {}", e))
}

/// 测试提供商连接
#[tauri::command]
pub async fn test_provider_connection(
    request: TestConnectionRequest,
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<ConnectionTestResult, String> {
    let manager = ai_manager.lock().await;
    manager.test_provider_connection(&request.provider_id).await
        .map_err(|e| format!("Failed to test connection: {}", e))
}

/// 使用临时配置测试提供商连接
#[tauri::command(rename_all = "camelCase")]
pub async fn test_connection_with_temp_config(
    provider_id: String,
    temp_config: AIConfig,
) -> Result<ConnectionTestResult, String> {
    use crate::core::providers;

    // 使用临时配置创建提供商工厂
    let factory = providers::create_provider_factory(&temp_config);

    // 测试连接
    factory.test_connection(&provider_id).await
        .map_err(|e| format!("Failed to test connection: {}", e))
}

/// 刷新提供商模型列表
#[tauri::command]
pub async fn refresh_provider_models(
    request: GetModelsRequest,
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<AIModel>, String> {
    let manager = ai_manager.lock().await;
    manager.refresh_provider_models(&request.provider_id).await
        .map_err(|e| format!("Failed to refresh models: {}", e))
}

/// 使用AI生成提交消息（增强版）
#[tauri::command]
pub async fn generate_commit_message_ai(
    request: GenerateCommitRequest,
    ai_manager: State<'_, Mutex<AIManager>>,
    git_engine: State<'_, Mutex<crate::core::git_engine::GitEngine>>,
) -> Result<GenerateCommitResponse, String> {
    use std::time::Instant;
    
    let start_time = Instant::now();
    
    // 获取Git状态和差异信息
    let git_status = {
        let engine = git_engine.lock().await;
        engine.get_status()
            .map_err(|e| format!("Failed to get git status: {}", e))?
    };
    
    let diff_summary = {
        let engine = git_engine.lock().await;
        engine.get_diff_summary(&request.selected_files)
            .map_err(|e| format!("Failed to get diff summary: {}", e))?
    };
    
    // 获取AI配置
    let manager = ai_manager.lock().await;
    let config = manager.get_config().await;
    
    // 构建提示词（参考Dish AI Commit的提示词模板）
    let system_prompt = create_commit_system_prompt(&config);
    let user_prompt = format!(
        "请为以下Git更改生成提交消息：\n\n分支: {}\n文件数量: {}\n修改的文件:\n{}\n\n差异摘要:\n{}\n\n{}",
        git_status.branch,
        request.selected_files.len(),
        request.selected_files.join("\n- "),
        diff_summary,
        request.additional_context.unwrap_or_default()
    );
    
    // 构建AI请求
    let ai_request = AIRequest {
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        model: config.base.model.clone(),
        temperature: Some(config.advanced.temperature),
        max_tokens: Some(config.advanced.max_tokens),
        stream: Some(false),
    };
    
    // 调用AI生成
    let response = manager.generate_commit_message(ai_request).await
        .map_err(|e| format!("Failed to generate commit message: {}", e))?;
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    Ok(GenerateCommitResponse {
        message: response.content,
        confidence: 0.85, // 简化的置信度
        processing_time_ms: processing_time,
        model_used: response.model,
    })
}

/// 创建提交消息系统提示词（参考Dish AI Commit）
fn create_commit_system_prompt(config: &AIConfig) -> String {
    let language = &config.base.language;
    let enable_emoji = config.features.enable_emoji;
    let enable_body = config.features.enable_body;
    
    format!(
        r#"# Git提交消息生成指南

## 核心指令

1. 根据实际更改确定此次提交的真实意图
2. 识别已修改的模块/文件
3. 确定修改类型
4. 使用{}编写所有内容（技术术语和范围除外）
5. 严格遵循示例中显示的确切格式模板
6. 范围和技术术语仅使用英文
7. {}包含适当的表情符号
8. {}包含详细的提交描述

## 禁止操作

1. 不要包含任何解释、问候或额外文本
2. 不要用英文写作（技术术语和范围除外）
3. 不要添加任何格式说明或元数据
4. 不要在输出中包含三重反引号（```）
5. 不要添加任何评论或问题
6. 不要偏离所需格式

## 格式模板

```
<type>(<scope>): <description>

[可选的详细描述]
```

## 类型检测指南

- feat: 新功能
- fix: 错误修复
- docs: 文档更改
- style: 代码格式（不影响代码运行的变动）
- refactor: 重构（既不是新增功能，也不是修改bug的代码变动）
- test: 增加测试
- chore: 构建过程或辅助工具的变动

请严格按照以上指南生成提交消息。"#,
        language,
        if enable_emoji { "启用时" } else { "禁用时" },
        if enable_body { "启用时" } else { "禁用时" }
    )
}

/// 使用提示模板生成提交消息
/// 作者：Evilek
/// 编写日期：2025-07-28
#[tauri::command]
pub async fn generate_commit_with_template(
    ai_manager: State<'_, Mutex<AIManager>>,
    template_id: String,
    diff: String,
    staged_files: Vec<String>,
    branch_name: Option<String>,
) -> Result<String, String> {
    let manager = ai_manager.lock().await;

    // 从AI配置中获取语言设置
    let config = manager.get_config().await;
    let language = match config.base.language.as_str() {
        "Simplified Chinese" => "zh-CN",
        "Traditional Chinese" => "zh-TW",
        "English" => "en",
        "Japanese" => "ja",
        "Korean" => "ko",
        "French" => "fr",
        "German" => "de",
        "Spanish" => "es",
        "Russian" => "ru",
        _ => "en", // 默认英文
    };

    let context = CommitContext {
        diff,
        staged_files,
        branch_name,
        commit_type: None,
        max_length: None,
        language: language.to_string(), // 使用配置中的语言设置
    };

    println!("🔍 [AI Commands] 使用模板生成提交消息，模板ID: {}", template_id);

    match manager.generate_commit_with_template(&template_id, context).await {
        Ok(response) => {
            println!("✅ [AI Commands] 提交消息生成成功");
            Ok(response.content)
        },
        Err(e) => {
            println!("❌ [AI Commands] 提交消息生成失败: {}", e);
            Err(format!("Failed to generate commit message: {}", e))
        },
    }
}

/// 获取所有可用的提示模板
#[tauri::command]
pub async fn get_prompt_templates(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<PromptTemplate>, String> {
    let manager = ai_manager.lock().await;
    Ok(manager.get_prompt_templates().await)
}

/// 创建自定义模板
/// 作者：Evilek
/// 编写日期：2025-01-29
#[tauri::command]
pub async fn create_custom_template(
    ai_manager: State<'_, Mutex<AIManager>>,
    template: PromptTemplate,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.create_custom_template(template).await
        .map_err(|e| format!("Failed to create template: {}", e))
}

/// 更新模板
/// 作者：Evilek
/// 编写日期：2025-01-29
#[tauri::command]
pub async fn update_template(
    ai_manager: State<'_, Mutex<AIManager>>,
    template: PromptTemplate,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.update_template(template).await
        .map_err(|e| format!("Failed to update template: {}", e))
}

/// 删除模板
/// 作者：Evilek
/// 编写日期：2025-01-29
#[tauri::command]
pub async fn delete_template(
    ai_manager: State<'_, Mutex<AIManager>>,
    template_id: String,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.delete_template(&template_id).await
        .map_err(|e| format!("Failed to delete template: {}", e))
}

/// 获取自定义模板列表
/// 作者：Evilek
/// 编写日期：2025-01-29
#[tauri::command]
pub async fn get_custom_templates(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<PromptTemplate>, String> {
    let manager = ai_manager.lock().await;
    Ok(manager.get_custom_templates().await)
}

/// 获取默认模板列表
/// 作者：Evilek
/// 编写日期：2025-01-29
#[tauri::command]
pub async fn get_default_templates(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<PromptTemplate>, String> {
    let manager = ai_manager.lock().await;
    Ok(manager.get_default_templates().await)
}

/// 添加自定义提示模板
#[tauri::command]
pub async fn add_prompt_template(
    ai_manager: State<'_, Mutex<AIManager>>,
    template: PromptTemplate,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.add_prompt_template(template).await;
    Ok(())
}
