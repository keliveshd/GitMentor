use crate::core::prompt_manager::PromptTemplate;
use crate::core::unified_template_manager::*;
use crate::types::template_types::*;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

// 全局统一模板管理器实例
static UNIFIED_TEMPLATE_MANAGER: Lazy<Mutex<Option<UnifiedTemplateManager>>> =
    Lazy::new(|| Mutex::new(None));

/// 初始化统一模板管理器
fn init_unified_template_manager(
    app_handle: &AppHandle,
) -> Result<&'static Mutex<Option<UnifiedTemplateManager>>> {
    let mut manager = UNIFIED_TEMPLATE_MANAGER.lock().unwrap();
    if manager.is_none() {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::new());
        let template_manager = UnifiedTemplateManager::new(&app_dir)?;
        *manager = Some(template_manager);
    }
    Ok(&UNIFIED_TEMPLATE_MANAGER)
}

// ==================== 提交模板命令 ====================

/// 获取所有提交模板
#[tauri::command]
pub fn get_all_commit_templates(app_handle: AppHandle) -> Result<Vec<PromptTemplate>, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let templates = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_all_commit_templates()
        .into_iter()
        .cloned()
        .collect();
    Ok(templates)
}

/// 获取指定提交模板
#[tauri::command]
pub fn get_commit_template(
    app_handle: AppHandle,
    template_id: String,
) -> Result<PromptTemplate, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let template = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_commit_template(&template_id)
        .ok_or_else(|| "模板不存在".to_string())?
        .clone();
    Ok(template)
}

/// 更新提交模板并创建新版本
#[tauri::command]
pub fn update_commit_template_with_version(
    app_handle: AppHandle,
    template_id: String,
    content: String,
    version_name: String,
    version_description: String,
) -> Result<String, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let version_id = manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .update_commit_template(&template_id, content, version_name, version_description)
        .map_err(|e| e.to_string())?;
    Ok(version_id)
}

/// 切换提交模板版本
#[tauri::command]
pub fn switch_commit_template_version(
    app_handle: AppHandle,
    template_id: String,
    version_id: String,
) -> Result<(), String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .switch_commit_template_version(&template_id, &version_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取提交模板版本历史
#[tauri::command]
pub fn get_commit_template_version_history(
    app_handle: AppHandle,
    template_id: String,
) -> Result<Vec<TemplateVersion>, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let versions = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_commit_template_versions(&template_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .cloned()
        .collect();
    Ok(versions)
}

/// 检查提交模板更新
#[tauri::command]
pub fn check_commit_template_updates(
    app_handle: AppHandle,
) -> Result<Vec<TemplateSystemUpdate>, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let updates = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .check_commit_template_updates();
    Ok(updates)
}

/// 应用提交模板更新
#[tauri::command]
pub fn apply_commit_template_update(
    app_handle: AppHandle,
    template_id: String,
) -> Result<(), String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .apply_commit_template_update(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 版本化模板命令（委托） ====================

/// 获取所有版本化模板
#[tauri::command]
pub fn get_all_unified_templates(
    app_handle: AppHandle,
) -> Result<Vec<TemplateConfigWithVersions>, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let templates = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_all_versioned_templates()
        .into_iter()
        .cloned()
        .collect();
    Ok(templates)
}

/// 获取版本化模板
#[tauri::command]
pub fn get_unified_template(
    app_handle: AppHandle,
    template_id: String,
) -> Result<TemplateConfigWithVersions, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let template = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_versioned_template(&template_id)
        .ok_or_else(|| "模板不存在".to_string())?
        .clone();
    Ok(template)
}

/// 获取统一模板的版本历史
#[tauri::command]
pub fn get_unified_template_version_history(
    app_handle: AppHandle,
    template_id: String,
) -> Result<Vec<TemplateVersion>, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let versions = {
        let manager_guard = manager.lock().unwrap();
        let manager_ref = manager_guard.as_ref().unwrap();
        let template = manager_ref
            .get_versioned_template(&template_id)
            .ok_or_else(|| "模板不存在".to_string())?;

        let mut versions = template.versions.clone();
        versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        versions
    };
    Ok(versions)
}

/// 更新版本化模板
#[tauri::command]
pub fn update_unified_template(
    app_handle: AppHandle,
    template_id: String,
    content: String,
    version_name: String,
    version_description: String,
) -> Result<String, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let version_id = manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .update_versioned_template(&template_id, content, version_name, version_description)
        .map_err(|e| e.to_string())?;
    Ok(version_id)
}

/// 新建统一模板版本
#[tauri::command]
pub fn update_template_version(
    app_handle: AppHandle,
    template_id: String,
    content: String,
    version_name: String,
    version_description: String,
) -> Result<String, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let version_id = manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .update_versioned_template(&template_id, content, version_name, version_description)
        .map_err(|e| e.to_string())?;
    Ok(version_id)
}

/// 切换版本化模板版本
#[tauri::command]
pub fn switch_unified_template_version(
    app_handle: AppHandle,
    template_id: String,
    version_id: String,
) -> Result<(), String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .switch_versioned_template_version(&template_id, &version_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 创建自定义模板
#[tauri::command]
pub fn create_unified_custom_template(
    app_handle: AppHandle,
    name: String,
    description: String,
    template_type: String,
    content: String,
    base_template_id: Option<String>,
) -> Result<String, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let template_id = manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .create_custom_template(name, description, template_type, content, base_template_id)
        .map_err(|e| e.to_string())?;
    Ok(template_id)
}

/// 删除自定义模板
#[tauri::command]
pub fn delete_unified_custom_template(
    app_handle: AppHandle,
    template_id: String,
) -> Result<(), String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .delete_custom_template(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 检查系统模板更新
#[tauri::command]
pub fn check_unified_system_updates(
    app_handle: AppHandle,
) -> Result<Vec<TemplateSystemUpdate>, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let updates = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .check_system_template_updates();
    Ok(updates)
}

/// 应用系统模板更新
#[tauri::command]
pub fn apply_unified_system_update(
    app_handle: AppHandle,
    template_id: String,
) -> Result<(), String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .apply_system_template_update(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 统一模板管理命令 ====================

/// 获取所有类型的模板（提交模板 + 版本化模板）
#[tauri::command]
pub fn get_all_templates_unified(
    app_handle: AppHandle,
) -> Result<TemplatesUnifiedResponse, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let manager_guard = manager.lock().unwrap();
    let manager_ref = manager_guard.as_ref().unwrap();

    let commit_templates = manager_ref
        .get_all_commit_templates()
        .into_iter()
        .cloned()
        .collect();

    let versioned_templates = manager_ref
        .get_all_versioned_templates()
        .into_iter()
        .cloned()
        .collect();

    Ok(TemplatesUnifiedResponse {
        commit_templates,
        versioned_templates,
    })
}

/// 模板统一响应结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplatesUnifiedResponse {
    pub commit_templates: Vec<PromptTemplate>,
    pub versioned_templates: Vec<TemplateConfigWithVersions>,
}

/// 批量更新系统模板
#[tauri::command]
pub fn batch_update_system_templates(
    app_handle: AppHandle,
    template_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let mut manager_guard = manager.lock().unwrap();
    let manager_ref = manager_guard.as_mut().unwrap();
    let mut updated_ids = Vec::new();

    for template_id in template_ids {
        if let Ok(()) = manager_ref.apply_system_template_update(&template_id) {
            updated_ids.push(template_id);
        }
    }

    Ok(updated_ids)
}

/// 重置所有系统模板到最新版本
#[tauri::command]
pub fn reset_all_system_templates(app_handle: AppHandle) -> Result<usize, String> {
    let manager = init_unified_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let updates = {
        let manager_guard = manager.lock().unwrap();
        let manager_ref = manager_guard.as_ref().unwrap();
        manager_ref.check_system_template_updates()
    };

    let mut count = 0;
    for update in updates {
        let mut manager_guard = manager.lock().unwrap();
        let manager_ref = manager_guard.as_mut().unwrap();
        if let Ok(()) = manager_ref.apply_system_template_update(&update.system_template_id) {
            count += 1;
        }
    }

    Ok(count)
}
