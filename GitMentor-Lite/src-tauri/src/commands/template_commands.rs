use crate::core::versioned_template_manager::*;
use crate::types::template_types::*;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

// 全局模板管理器实例
static TEMPLATE_MANAGER: Lazy<Mutex<Option<VersionedTemplateManager>>> =
    Lazy::new(|| Mutex::new(None));

/// 初始化模板管理器
fn init_template_manager(
    app_handle: &AppHandle,
) -> Result<&'static Mutex<Option<VersionedTemplateManager>>> {
    let mut manager = TEMPLATE_MANAGER.lock().unwrap();
    if manager.is_none() {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::new());
        let template_manager = VersionedTemplateManager::new(&app_dir)?;
        *manager = Some(template_manager);
    }
    Ok(&TEMPLATE_MANAGER)
}

/// 获取所有模板列表
#[tauri::command]
pub fn get_all_templates(app_handle: AppHandle) -> Result<Vec<TemplateConfigWithVersions>, String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let templates = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_all_templates()
        .into_iter()
        .cloned()
        .collect();
    Ok(templates)
}

/// 获取指定模板的详细信息
#[tauri::command]
pub fn get_template_details(
    app_handle: AppHandle,
    template_id: String,
) -> Result<TemplateConfigWithVersions, String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let template = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_template(&template_id)
        .ok_or_else(|| "模板不存在".to_string())?
        .clone();
    Ok(template)
}

/// 获取模板的版本历史
#[tauri::command]
pub fn get_template_versions(
    app_handle: AppHandle,
    template_id: String,
) -> Result<Vec<TemplateVersion>, String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let versions = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_template_versions(&template_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .cloned()
        .collect();
    Ok(versions)
}

/// 更新模板内容并创建新版本
#[tauri::command]
pub fn update_template_content(
    app_handle: AppHandle,
    template_id: String,
    content: String,
    version_name: String,
    version_description: String,
) -> Result<String, String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let version_id = manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .update_template(&template_id, content, version_name, version_description)
        .map_err(|e| e.to_string())?;
    Ok(version_id)
}

/// 切换模板版本
#[tauri::command]
pub fn switch_template_version(
    app_handle: AppHandle,
    template_id: String,
    version_id: String,
) -> Result<(), String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .switch_template_version(&template_id, &version_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取系统模板更新信息
#[tauri::command]
pub fn get_system_template_updates(
    app_handle: AppHandle,
) -> Result<Vec<TemplateSystemUpdate>, String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let updates = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_system_template_updates();
    Ok(updates)
}

/// 应用系统模板更新
#[tauri::command]
pub fn apply_system_template_update(
    app_handle: AppHandle,
    template_id: String,
) -> Result<(), String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .apply_system_template_update(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 创建自定义模板（版本化）
#[tauri::command]
pub fn create_versioned_custom_template(
    app_handle: AppHandle,
    name: String,
    description: String,
    template_type: String,
    content: String,
    base_template_id: Option<String>,
) -> Result<String, String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
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
pub fn delete_custom_template_versioned(
    app_handle: AppHandle,
    template_id: String,
) -> Result<(), String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .delete_custom_template(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取模板的当前内容
#[tauri::command]
pub fn get_template_content_versioned(
    app_handle: AppHandle,
    template_id: String,
) -> Result<String, String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    let content = manager
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .get_template_content(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(content)
}

/// 还原到系统模板的初始版本
#[tauri::command]
pub fn revert_to_builtin_version(app_handle: AppHandle, template_id: String) -> Result<(), String> {
    let manager = init_template_manager(&app_handle).map_err(|e| e.to_string())?;
    manager
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .revert_to_builtin_version(&template_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}
