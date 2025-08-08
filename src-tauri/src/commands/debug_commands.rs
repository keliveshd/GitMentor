use crate::utils::debug_logger;
use serde::{Deserialize, Serialize};

/// 调试设置结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct DebugSettings {
    pub debug_logs_enabled: bool,
}

/// 获取调试设置
/// 作者：Evilek
/// 编写日期：2025-08-06
#[tauri::command]
pub async fn get_debug_settings() -> Result<DebugSettings, String> {
    Ok(DebugSettings {
        debug_logs_enabled: debug_logger::is_debug_enabled(),
    })
}

/// 设置调试日志开关
/// 作者：Evilek
/// 编写日期：2025-08-06
#[tauri::command]
pub async fn set_debug_logs_enabled(enabled: bool) -> Result<String, String> {
    debug_logger::set_debug_enabled(enabled);
    
    let message = if enabled {
        "调试日志已启用"
    } else {
        "调试日志已禁用"
    };
    
    Ok(message.to_string())
}

/// 更新调试设置
/// 作者：Evilek
/// 编写日期：2025-08-06
#[tauri::command]
pub async fn update_debug_settings(settings: DebugSettings) -> Result<String, String> {
    debug_logger::set_debug_enabled(settings.debug_logs_enabled);
    Ok("调试设置已更新".to_string())
}
