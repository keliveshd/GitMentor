use serde::{Deserialize, Serialize};
// 调试命令：全局开关 + 前端配置面板读写
// Author: Evilek, Date: 2025-08-11
// 这个模块只改一个布尔状态，想加花活去前端玩

use std::sync::atomic::{AtomicBool, Ordering};

/// 全局调试日志开关
/// 作者：Evilek
/// 编写日期：2025-08-07
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(true); // 临时启用调试日志

/// 设置调试日志开关
pub fn set_debug_enabled(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        println!("[DEBUG] 调试日志已启用");
    } else {
        println!("[INFO] 调试日志已禁用");
    }
}

/// 获取调试日志开关状态
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

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
        debug_logs_enabled: is_debug_enabled(),
    })
}

/// 设置调试日志开关
/// 作者：Evilek
/// 编写日期：2025-08-06
#[tauri::command]
pub async fn set_debug_logs_enabled(enabled: bool) -> Result<String, String> {
    set_debug_enabled(enabled);

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
    set_debug_enabled(settings.debug_logs_enabled);
    Ok("调试设置已更新".to_string())
}
