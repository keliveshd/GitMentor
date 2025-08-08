use tauri::State;
use tokio::sync::Mutex;
use crate::core::git_config::{GitConfig, GitConfigManager, GitExecutionMode};
use crate::core::git_engine::GitEngine;

/**
 * Git配置相关命令
 * 作者：Evilek
 * 编写日期：2025-08-07
 */

/// 获取Git配置
#[tauri::command]
pub async fn get_git_config(
    git_config_manager: State<'_, Mutex<GitConfigManager>>,
) -> Result<GitConfig, String> {
    let manager = git_config_manager.lock().await;
    Ok(manager.get_config().clone())
}

/// 更新Git配置
#[tauri::command]
pub async fn update_git_config(
    config: GitConfig,
    git_config_manager: State<'_, Mutex<GitConfigManager>>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<String, String> {
    // 更新配置管理器
    {
        let mut manager = git_config_manager.lock().await;
        manager.update_config(config.clone())
            .map_err(|e| format!("Failed to save git config: {}", e))?;
    }
    
    // 更新Git引擎配置
    {
        let mut engine = git_engine.lock().await;
        engine.update_config(config);
    }
    
    Ok("Git配置已更新".to_string())
}

/// 获取可用的Git执行方式
#[tauri::command]
pub async fn get_available_git_modes() -> Result<Vec<(GitExecutionMode, String)>, String> {
    let modes = crate::core::git_config::GitConfigManager::get_available_modes()
        .into_iter()
        .map(|(mode, desc)| (mode, desc.to_string()))
        .collect();
    Ok(modes)
}

/// 测试Git执行方式是否可用
#[tauri::command]
pub async fn test_git_execution_mode(
    mode: GitExecutionMode,
) -> Result<String, String> {
    use crate::core::git_config::GitConfig;
    
    let mut test_config = GitConfig::default();
    test_config.execution_mode = mode.clone();
        
    // 这里可以添加实际的测试逻辑
    // 比如尝试执行git --version命令
    
    match mode {
        GitExecutionMode::Auto => Ok("自动检测模式已设置".to_string()),
        GitExecutionMode::SystemGit => Ok("系统Git模式已设置".to_string()),
        GitExecutionMode::BundledGit => Ok("内置Git模式已设置".to_string()),
        GitExecutionMode::Git2Api => Ok("Git2库API模式已设置".to_string()),
    }
}

/// 重置Git配置为默认值
#[tauri::command]
pub async fn reset_git_config(
    git_config_manager: State<'_, Mutex<GitConfigManager>>,
    git_engine: State<'_, Mutex<GitEngine>>,
) -> Result<String, String> {
    let default_config = GitConfig::default();
    
    // 更新配置管理器
    {
        let mut manager = git_config_manager.lock().await;
        manager.update_config(default_config.clone())
            .map_err(|e| format!("Failed to reset git config: {}", e))?;
    }
    
    // 更新Git引擎配置
    {
        let mut engine = git_engine.lock().await;
        engine.update_config(default_config);
    }
    
    Ok("Git配置已重置为默认值".to_string())
}
