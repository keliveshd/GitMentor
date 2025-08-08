use std::sync::atomic::{AtomicBool, Ordering};

/// 全局调试日志开关
/// 作者：Evilek
/// 编写日期：2025-08-06
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

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

/// 调试日志宏
/// 只有在调试开关启用时才输出日志
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::utils::debug_logger::is_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// 警告日志宏（始终显示）
#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

/// 错误日志宏（始终显示）
#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

/// 信息日志宏（始终显示）
#[macro_export]
macro_rules! info_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}
