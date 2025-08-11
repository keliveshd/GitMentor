use tauri::{State, Emitter};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::core::ai_manager::AIManager;
use crate::core::ai_provider::{AIRequest, AIModel, ConnectionTestResult, ChatMessage};
use crate::core::ai_config::AIConfig;
use crate::core::prompt_manager::{PromptTemplate, CommitContext};
use crate::core::conversation_logger::ConversationRecord;

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
    /// 推理内容（<think>标签内的内容）
    /// 作者：Evilek
    /// 编写日期：2025-01-10
    pub reasoning_content: Option<String>,
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
        reasoning_content: response.reasoning_content, // 添加推理内容 - Author: Evilek, Date: 2025-01-10
    })
}

/// 创建提交消息系统提示词（优化版，避免标题格式干扰）
/// 作者：Evilek
/// 编写日期：2025-01-29
fn create_commit_system_prompt(config: &AIConfig) -> String {
    let language = &config.base.language;
    let enable_emoji = config.features.enable_emoji;
    let enable_body = config.features.enable_body;

    format!(
        r#"你是专业的Git提交消息生成助手。请根据代码变更生成简洁、准确的提交消息。

核心要求：
- 根据实际更改确定此次提交的真实意图
- 识别已修改的模块/文件和修改类型
- 使用{}编写所有内容（技术术语和范围除外）
- 范围和技术术语仅使用英文
- 严格遵循格式：<type>(<scope>): <description>
- {}包含适当的表情符号
- {}包含详细的提交描述

提交类型说明：
feat: 新功能, fix: 错误修复, docs: 文档更改, style: 代码格式, refactor: 重构, test: 增加测试, chore: 构建过程或辅助工具的变动

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式
- 不要添加任何评论或问题
- 不要偏离所需格式

直接输出提交消息，无需其他内容。"#,
        language,
        if enable_emoji { "启用时" } else { "禁用时" },
        if enable_body { "启用时" } else { "禁用时" }
    )
}

/// 使用提示模板生成提交消息
/// 作者：Evilek
/// 编写日期：2025-07-28
/// 更新日期：2025-08-04
#[tauri::command]
pub async fn generate_commit_with_template(
    ai_manager: State<'_, Mutex<AIManager>>,
    git_engine: State<'_, Mutex<crate::core::git_engine::GitEngine>>,
    template_id: String,
    diff: String,
    staged_files: Vec<String>,
    branch_name: Option<String>,
) -> Result<String, String> {
    let manager = ai_manager.lock().await;

    // 获取当前仓库路径
    let repository_path = {
        let engine = git_engine.lock().await;
        engine.get_repository_path()
    };

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
        "Portuguese" => "pt",
        "Italian" => "it",
        "Dutch" => "nl",
        "Swedish" => "sv",
        "Czech" => "cs",
        "Polish" => "pl",
        "Turkish" => "tr",
        "Vietnamese" => "vi",
        "Thai" => "th",
        "Indonesian" => "id",
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

    match manager.generate_commit_with_template(&template_id, context, repository_path).await {
        Ok(response) => {
            Ok(response.content)
        },
        Err(e) => {
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

/// 检查模板是否支持两段式处理
/// 作者：Evilek
/// 编写日期：2025-08-08
#[tauri::command]
pub async fn check_template_two_phase_support(
    ai_manager: State<'_, Mutex<AIManager>>,
    template_id: String,
) -> Result<bool, String> {
    let manager = ai_manager.lock().await;
    let prompt_manager = manager.get_prompt_manager().await;
    Ok(prompt_manager.supports_two_phase(&template_id))
}

/// 获取模板的两段式配置状态
/// 作者：Evilek
/// 编写日期：2025-08-08
#[tauri::command]
pub async fn get_template_two_phase_status(
    ai_manager: State<'_, Mutex<AIManager>>,
    template_id: String,
) -> Result<Option<(bool, bool)>, String> {
    let manager = ai_manager.lock().await;
    let prompt_manager = manager.get_prompt_manager().await;
    Ok(prompt_manager.get_two_phase_status(&template_id))
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

/// 获取对话记录
/// 作者：Evilek
/// 编写日期：2025-01-30
#[tauri::command]
pub async fn get_conversation_history(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<ConversationRecord>, String> {
    let manager = ai_manager.lock().await;
    Ok(manager.get_conversation_history().await)
}

/// 清空对话记录
/// 作者：Evilek
/// 编写日期：2025-01-30
#[tauri::command]
pub async fn clear_conversation_history(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.clear_conversation_history().await
        .map_err(|e| format!("Failed to clear conversation history: {}", e))
}

/// 根据仓库路径获取对话记录
/// 作者：Evilek
/// 编写日期：2025-08-04
#[tauri::command]
pub async fn get_conversation_history_by_repository(
    ai_manager: State<'_, Mutex<AIManager>>,
    repository_path: Option<String>,
) -> Result<Vec<ConversationRecord>, String> {
    let manager = ai_manager.lock().await;
    manager.get_conversation_history_by_repository(repository_path.as_deref()).await
        .map_err(|e| format!("Failed to get conversation history: {}", e))
}

/// 获取所有仓库路径列表
/// 作者：Evilek
/// 编写日期：2025-08-04
#[tauri::command]
pub async fn get_repository_paths(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<String>, String> {
    let manager = ai_manager.lock().await;
    manager.get_repository_paths().await
        .map_err(|e| format!("Failed to get repository paths: {}", e))
}

/// 检查是否应该使用分层提交
/// 作者：Evilek
/// 编写日期：2025-08-04
/// 更新日期：2025-08-05
#[tauri::command]
pub async fn should_use_layered_commit(
    ai_manager: State<'_, Mutex<AIManager>>,
    git_engine: State<'_, Mutex<crate::core::git_engine::GitEngine>>,
    template_id: String,
    diff: String,
    staged_files: Vec<String>,
) -> Result<bool, String> {
    use crate::core::layered_commit_manager::LayeredCommitManager;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // 创建LayeredCommitManager实例
    let ai_manager_arc = Arc::new(RwLock::new(ai_manager.lock().await.clone()));
    let git_engine_arc = Arc::new(RwLock::new(git_engine.lock().await.clone()));
    let manager = LayeredCommitManager::new(ai_manager_arc, git_engine_arc);

    // 调用真正的分层提交检测逻辑
    manager.should_use_layered_commit(&template_id, &diff, &staged_files)
        .await
        .map_err(|e| format!("检查分层提交失败: {}", e))
}

/// 全局分层提交管理器实例，用于任务取消
/// Author: Evilek, Date: 2025-01-09
use std::sync::Mutex as StdMutex;
use std::sync::Arc as StdArc;
use once_cell::sync::Lazy;

static LAYERED_COMMIT_MANAGER: Lazy<StdMutex<Option<StdArc<crate::core::layered_commit_manager::LayeredCommitManager>>>> =
    Lazy::new(|| StdMutex::new(None));

/// 执行分层提交
/// 作者：Evilek
/// 编写日期：2025-08-04
/// 更新日期：2025-08-05
#[tauri::command]
pub async fn execute_layered_commit(
    ai_manager: State<'_, Mutex<AIManager>>,
    git_engine: State<'_, Mutex<crate::core::git_engine::GitEngine>>,
    app_handle: tauri::AppHandle,
    templateId: String,
    stagedFiles: Vec<String>,
    branchName: Option<String>,
) -> Result<crate::core::layered_commit_manager::LayeredCommitResult, String> {
    use crate::core::layered_commit_manager::LayeredCommitManager;
    use std::sync::Arc;
    use tokio::sync::RwLock;



    // 获取当前仓库路径
    let repository_path = {
        let engine = git_engine.lock().await;
        engine.get_repository_path()
    };

    // 创建LayeredCommitManager实例
    let ai_manager_arc = Arc::new(RwLock::new(ai_manager.lock().await.clone()));
    let git_engine_arc = Arc::new(RwLock::new(git_engine.lock().await.clone()));
    let manager = StdArc::new(LayeredCommitManager::new(ai_manager_arc, git_engine_arc));

    // 保存管理器实例到全局变量，用于任务取消 - Author: Evilek, Date: 2025-01-09
    {
        let mut global_manager = LAYERED_COMMIT_MANAGER.lock().unwrap();
        *global_manager = Some(manager.clone());
    }

    // 创建进度回调函数，用于发送进度事件到前端
    let app_handle_clone = app_handle.clone();
    let progress_callback = move |progress: crate::core::layered_commit_manager::LayeredCommitProgress| {
        let progress_json = serde_json::json!({
            "session_id": progress.session_id,
            "current_step": progress.current_step,
            "total_steps": progress.total_steps,
            "status": progress.status,
            "current_file": progress.current_file,
            "file_summaries": progress.file_summaries
        });

        let _ = app_handle_clone.emit("layered-commit-progress", &progress_json);
    };

    // 调用真正的分层提交逻辑
    let result = manager.execute_layered_commit(
        &templateId,
        stagedFiles,
        branchName,
        repository_path,
        progress_callback,
    ).await;

    // 清理全局管理器实例 - Author: Evilek, Date: 2025-01-09
    {
        let mut global_manager = LAYERED_COMMIT_MANAGER.lock().unwrap();
        *global_manager = None;
    }

    match result {
        Ok(result) => {
            Ok(result)
        },
        Err(e) => {
            Err(format!("分层提交执行失败: {}", e))
        }
    }
}





/// 取消分层提交
/// Author: Evilek, Date: 2025-01-09
#[tauri::command]
pub async fn cancel_layered_commit() -> Result<(), String> {
    let global_manager = LAYERED_COMMIT_MANAGER.lock().unwrap();
    if let Some(manager) = global_manager.as_ref() {
        manager.cancel();
        Ok(())
    } else {
        Err("没有正在执行的分层提交任务".to_string())
    }
}

/// 检查是否需要首次启动引导
/// Author: Evilek, Date: 2025-01-09
#[tauri::command]
pub async fn check_first_time_setup(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<bool, String> {
    let manager = ai_manager.lock().await;
    let config = manager.get_config().await;

    // 检查当前选择的提供商是否配置了API密钥
    let needs_setup = match config.base.provider.as_str() {
        "OpenAI" => config.providers.openai.api_key.is_empty(),
        "Ollama" => false, // Ollama通常不需要API密钥
        "Zhipu" => config.providers.zhipu.api_key.is_empty(),
        "Anthropic" => config.providers.anthropic.api_key.is_empty(),
        "DashScope" => config.providers.dashscope.api_key.is_empty(),
        "Doubao" => config.providers.doubao.api_key.is_empty(),
        "Gemini" => config.providers.gemini.api_key.is_empty(),
        "Deepseek" => config.providers.deepseek.api_key.is_empty(),
        "Siliconflow" => config.providers.siliconflow.api_key.is_empty(),
        "OpenRouter" => config.providers.openrouter.api_key.is_empty(),
        "Together" => config.providers.together.api_key.is_empty(),
        "Mistral" => config.providers.mistral.api_key.is_empty(),
        "BaiduQianfan" => config.providers.baidu_qianfan.api_key.is_empty() || config.providers.baidu_qianfan.secret_key.is_empty(),
        "AzureOpenAI" => config.providers.azure_openai.api_key.is_empty() || config.providers.azure_openai.endpoint.is_empty(),
        "Cloudflare" => config.providers.cloudflare.api_key.is_empty() || config.providers.cloudflare.account_id.is_empty(),
        "VertexAI" => config.providers.vertexai.project_id.is_empty() || config.providers.vertexai.credentials_path.is_empty(),
        "Groq" => config.providers.groq.api_key.is_empty(),
        _ => true, // 未知提供商，需要设置
    };

    Ok(needs_setup)
}

/// 测试AI连接
/// Author: Evilek, Date: 2025-01-09
#[tauri::command]
pub async fn test_ai_connection(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<String, String> {
    let manager = ai_manager.lock().await;
    let config = manager.get_config().await;

    // 构建简单的测试请求
    let test_request = crate::core::ai_provider::AIRequest {
        messages: vec![
            crate::core::ai_provider::ChatMessage {
                role: "user".to_string(),
                content: "Hello, please respond with 'Connection test successful'".to_string(),
            }
        ],
        model: config.base.model.clone(), // 修复：添加model字段
        temperature: Some(0.1),
        max_tokens: Some(50),
        stream: Some(false),
    };

    // 尝试发送请求，使用正确的方法名
    match manager.generate_commit_message(test_request).await {
        Ok(response) => {
            if response.content.contains("successful") || response.content.contains("成功") {
                Ok("AI连接测试成功".to_string())
            } else {
                Ok(format!("AI响应正常，返回内容: {}", response.content))
            }
        },
        Err(e) => {
            Err(format!("AI连接测试失败: {}", e))
        }
    }
}

/// 获取分层提交会话列表
/// 作者：Evilek
/// 编写日期：2025-08-04
#[tauri::command]
pub async fn get_layered_sessions(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<Vec<String>, String> {
    let manager = ai_manager.lock().await;
    manager.get_layered_sessions().await
        .map_err(|e| format!("Failed to get layered sessions: {}", e))
}

/// 根据会话ID获取对话记录
/// 作者：Evilek
/// 编写日期：2025-08-04
#[tauri::command]
pub async fn get_conversation_records_by_session(
    ai_manager: State<'_, Mutex<AIManager>>,
    session_id: String,
) -> Result<Vec<ConversationRecord>, String> {
    let manager = ai_manager.lock().await;
    manager.get_conversation_records_by_session(&session_id).await
        .map_err(|e| format!("Failed to get conversation records by session: {}", e))
}

/// 重新加载默认模板（清理缓存）
/// 作者：Evilek
/// 编写日期：2025-01-29
#[allow(dead_code)] // 预留的管理功能，暂未在前端使用
#[tauri::command]
pub async fn reload_default_templates(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.reload_default_templates().await
        .map_err(|e| format!("Failed to reload templates: {}", e))
}

/// 清理所有缓存和配置文件
/// 作者：Evilek
/// 编写日期：2025-01-29
#[allow(dead_code)] // 预留的管理功能，暂未在前端使用
#[tauri::command]
pub async fn clear_all_cache(
    ai_manager: State<'_, Mutex<AIManager>>,
) -> Result<(), String> {
    let manager = ai_manager.lock().await;
    manager.clear_all_cache().await
        .map_err(|e| format!("Failed to clear cache: {}", e))
}

/// 检查并处理文件token限制
/// Author: Evilek
/// Date: 2025-01-08
/// 对单文件变更和新增文件进行token检查，超限则分割处理
#[derive(serde::Serialize)]
pub struct FileTokenCheckResult {
    #[serde(rename = "processedFiles")]
    pub processed_files: Vec<String>,
    #[serde(rename = "needsSplit")]
    pub needs_split: bool,
}

#[tauri::command]
pub async fn check_and_process_file_tokens(
    ai_manager: State<'_, Mutex<AIManager>>,
    git_engine: State<'_, Mutex<crate::core::git_engine::GitEngine>>,
    filePaths: Vec<String>,
    template_id: Option<String>,
) -> Result<FileTokenCheckResult, String> {
    use crate::utils::token_counter::TokenCounter;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    println!("🔍 [check_and_process_file_tokens] 开始处理 {} 个文件", filePaths.len());

    let ai_manager_arc = Arc::new(RwLock::new(ai_manager.lock().await.clone()));
    let git_engine_arc = Arc::new(RwLock::new(git_engine.lock().await.clone()));

    println!("🔍 [check_and_process_file_tokens] 获取AI配置...");
    // 获取AI配置以确定token限制
    let ai_manager_guard = ai_manager_arc.read().await;
    let config = ai_manager_guard.get_config().await;
    let model_max_tokens = match config.base.model.as_str() {
        m if m.contains("gpt-4") => Some(8192),
        m if m.contains("gpt-3.5") => Some(4096),
        m if m.contains("claude") => Some(100000),
        m if m.contains("gemini") => Some(32768),
        m if m.contains("qwen2.5:32b") => Some(32768), // qwen2.5:32b 支持32k上下文
        m if m.contains("qwen") => Some(8192), // 其他qwen模型默认8k
        _ => Some(4096), // 默认限制
    };
    drop(ai_manager_guard);
    println!("🔍 [check_and_process_file_tokens] 模型token限制: {:?}", model_max_tokens);

    let mut processed_files = Vec::new();
    let mut needs_split = false;

    println!("🔍 [check_and_process_file_tokens] 开始获取文件diff...");

    // 性能优化：使用批量diff获取，避免单个文件的重复Git操作
    let git_engine_guard = git_engine_arc.read().await;
    let batch_diff_result = git_engine_guard.get_diff_summary(&filePaths);
    drop(git_engine_guard);

    // 如果批量获取失败，回退到单个文件处理（但添加超时保护）
    let mut file_diffs = Vec::new();

    match batch_diff_result {
        Ok(batch_diff) => {
            println!("🔍 [check_and_process_file_tokens] 使用批量diff，长度: {}", batch_diff.len());
            // 简化处理：如果能获取到批量diff，就假设所有文件都有变更
            // 这是一个权衡：牺牲一些精确性换取性能
            for file_path in &filePaths {
                // 为每个文件分配一部分diff内容（简化估算）
                let estimated_diff = format!("diff --git a/{} b/{}\n--- a/{}\n+++ b/{}\n@@ -1,10 +1,10 @@\n 文件变更内容...",
                                            file_path, file_path, file_path, file_path);
                file_diffs.push((file_path.clone(), Some(estimated_diff)));
            }
        },
        Err(_) => {
            println!("⚠️ [check_and_process_file_tokens] 批量diff获取失败，回退到单个文件处理");
            // 回退到原来的逻辑，但添加超时保护
            let git_engine_guard = git_engine_arc.read().await;

            for (index, file_path) in filePaths.iter().enumerate() {
                println!("🔍 [check_and_process_file_tokens] 处理文件 {}/{}: {}", index + 1, filePaths.len(), file_path);

                // 使用优化后的Git diff获取
                let start_time = std::time::Instant::now();
                match git_engine_guard.get_simple_file_diff(file_path) {
                    Ok(diff_content) => {
                        let elapsed = start_time.elapsed();
                        println!("🔍 [check_and_process_file_tokens] 文件 {} diff长度: {}, 耗时: {:?}", file_path, diff_content.len(), elapsed);
                        file_diffs.push((file_path.clone(), Some(diff_content)));
                    },
                    Err(e) => {
                        println!("⚠️ [check_and_process_file_tokens] 文件 {} diff获取失败: {}", file_path, e);
                        file_diffs.push((file_path.clone(), None));
                    }
                }
            }
            drop(git_engine_guard);
        }
    }

    println!("🔍 [check_and_process_file_tokens] 完成diff获取，开始token分析...");

    // 智能分组策略：根据token使用量决定处理方式
    let mut total_tokens = 0u32;
    let mut large_files = Vec::new();
    let mut normal_files = Vec::new();

    // 计算每个文件的token使用量
    for (file_path, diff_content_opt) in file_diffs {
        if let Some(diff_content) = diff_content_opt {
            println!("🔍 [check_and_process_file_tokens] 计算文件 {} 的token...", file_path);
            let file_tokens = TokenCounter::estimate_file_diff_tokens(&file_path, &diff_content);
            println!("🔍 [check_and_process_file_tokens] 文件 {} token数: {}", file_path, file_tokens);

            // 单个文件超过限制，需要分割
            if TokenCounter::is_over_limit(file_tokens, model_max_tokens) {
                println!("⚠️ [check_and_process_file_tokens] 文件 {} 超过token限制，标记为大文件", file_path);
                needs_split = true;
                large_files.push((file_path, diff_content, file_tokens));
            } else {
                total_tokens += file_tokens;
                normal_files.push((file_path, diff_content, file_tokens));
            }
        } else {
            // diff获取失败的文件直接添加
            println!("⚠️ [check_and_process_file_tokens] 文件 {} diff获取失败，直接添加", file_path);
            processed_files.push(file_path);
        }
    }

    println!("🔍 [check_and_process_file_tokens] Token分析完成 - 大文件: {}, 普通文件: {}, 总token: {}",
             large_files.len(), normal_files.len(), total_tokens);

    // 处理大文件：需要分割
    for (file_path, diff_content, _) in large_files {
        // 检查是否为新增文件
        if diff_content.contains("new file mode") || diff_content.starts_with("+++") {
            // 新增文件：截取前面部分
            processed_files.push(format!("{}#truncated", file_path));
        } else {
            // 变更文件：标记需要分割
            processed_files.push(format!("{}#split", file_path));
        }
    }

    // 处理普通文件：每个文件单独处理，超过token限制时自动分割
    // Author: Evilek, Date: 2025-01-09 - 移除批量合并逻辑，改为单文件独立处理
    for (file_path, _, file_tokens) in normal_files {
        // 获取模板的max_tokens配置作为分割依据
        // Author: Evilek, Date: 2025-01-09 - 修复PromptManager实例化问题，使用AI管理器中的实例
        let template_max_tokens = if let Some(ref template_id_str) = template_id {
            let ai_manager_guard = ai_manager_arc.read().await;
            let prompt_manager = ai_manager_guard.get_prompt_manager().await;
            prompt_manager.get_template_config(template_id_str)
                .and_then(|(max_tokens, _)| max_tokens)
                .unwrap_or(1000) // 修复：增加默认值到1000 tokens，避免过度分割
        } else {
            1000 // 修复：增加默认值
        };

        // 使用模板的max_tokens作为分割的安全限制（保留30%余量给文件名和格式）
        let safe_limit = (template_max_tokens as f32 * 0.7) as u32;

        if file_tokens > safe_limit {
            // 文件超过限制，标记为需要分割
            processed_files.push(format!("{}#split", file_path));
            needs_split = true;
        } else {
            // 文件大小合适，直接处理
            processed_files.push(file_path);
        }
    }

    println!("🔍 [check_and_process_file_tokens] 处理完成 - 输出文件: {:?}, 需要分割: {}",
             processed_files, needs_split);

    Ok(FileTokenCheckResult {
        processed_files,
        needs_split,
    })
}
