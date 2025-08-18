use crate::core::update_manager::{UpdateManager, VersionInfo};
use anyhow::Result;
use std::path::PathBuf;
use tauri::{command, AppHandle, Emitter, Manager};

/**
 * GitMentor 更新相关 Tauri 命令
 * 作者：Evilek
 * 编写日期：2025-01-18
 */

/// 下载进度事件
#[derive(Clone, serde::Serialize)]
struct DownloadProgressEvent {
    downloaded: u64,
    total: u64,
    percentage: f64,
}

/// 检查更新命令
#[command]
pub async fn check_for_updates() -> Result<VersionInfo, String> {
    // 从 tauri.conf.json 读取当前版本
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let update_manager = UpdateManager::new(current_version);
    
    update_manager
        .check_for_updates()
        .await
        .map_err(|e| format!("检查更新失败: {}", e))
}

/// 下载更新命令
#[command]
pub async fn download_update(
    app_handle: AppHandle,
    download_url: String,
) -> Result<String, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let update_manager = UpdateManager::new(current_version);
    
    // 获取应用数据目录
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    
    // 创建下载路径
    let download_dir = app_data_dir.join("updates");
    let filename = extract_filename_from_url(&download_url)
        .unwrap_or_else(|| "GitMentor-update.msi".to_string());
    let download_path = download_dir.join(&filename);
    
    // 创建进度回调
    let app_handle_clone = app_handle.clone();
    let progress_callback = Box::new(move |downloaded: u64, total: u64| {
        let percentage = if total > 0 {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        let event = DownloadProgressEvent {
            downloaded,
            total,
            percentage,
        };
        
        // 发送进度事件到前端
        let _ = app_handle_clone.emit("download-progress", &event);
    });
    
    update_manager
        .download_update(&download_url, &download_path, Some(progress_callback))
        .await
        .map_err(|e| format!("下载更新失败: {}", e))?;
    
    Ok(download_path.to_string_lossy().to_string())
}

/// 安装更新命令
#[command]
pub async fn install_update(installer_path: String) -> Result<(), String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let update_manager = UpdateManager::new(current_version);
    let path = PathBuf::from(installer_path);
    
    update_manager
        .install_update(&path)
        .await
        .map_err(|e| format!("安装更新失败: {}", e))?;
    
    Ok(())
}

/// 获取当前版本信息命令
#[command]
pub async fn get_current_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// 清理下载的更新文件命令
#[command]
pub async fn cleanup_update_files(app_handle: AppHandle) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    
    let download_dir = app_data_dir.join("updates");
    
    if download_dir.exists() {
        tokio::fs::remove_dir_all(&download_dir)
            .await
            .map_err(|e| format!("清理更新文件失败: {}", e))?;
    }
    
    Ok(())
}

/// 检查更新文件是否存在命令
#[command]
pub async fn check_update_file_exists(
    app_handle: AppHandle,
    filename: String,
) -> Result<bool, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    
    let file_path = app_data_dir.join("updates").join(filename);
    Ok(file_path.exists())
}

/// 从 URL 中提取文件名
fn extract_filename_from_url(url: &str) -> Option<String> {
    url.split('/')
        .last()
        .and_then(|s| {
            // 移除查询参数
            s.split('?').next()
        })
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

/// 更新设置结构
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateSettings {
    pub auto_check: bool,
    pub check_interval_hours: u32,
    pub include_prerelease: bool,
    pub download_path: Option<String>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check: true,
            check_interval_hours: 24,
            include_prerelease: false,
            download_path: None,
        }
    }
}

/// 获取更新设置命令
#[command]
pub async fn get_update_settings(app_handle: AppHandle) -> Result<UpdateSettings, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    
    let settings_path = app_data_dir.join("update_settings.json");
    
    if settings_path.exists() {
        let content = tokio::fs::read_to_string(&settings_path)
            .await
            .map_err(|e| format!("读取更新设置失败: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("解析更新设置失败: {}", e))
    } else {
        Ok(UpdateSettings::default())
    }
}

/// 保存更新设置命令
#[command]
pub async fn save_update_settings(
    app_handle: AppHandle,
    settings: UpdateSettings,
) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    
    // 确保目录存在
    tokio::fs::create_dir_all(&app_data_dir)
        .await
        .map_err(|e| format!("创建应用数据目录失败: {}", e))?;
    
    let settings_path = app_data_dir.join("update_settings.json");
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("序列化更新设置失败: {}", e))?;
    
    tokio::fs::write(&settings_path, content)
        .await
        .map_err(|e| format!("保存更新设置失败: {}", e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename_from_url() {
        assert_eq!(
            extract_filename_from_url("https://github.com/user/repo/releases/download/v1.0.0/app.msi"),
            Some("app.msi".to_string())
        );
        
        assert_eq!(
            extract_filename_from_url("https://example.com/file.exe?param=value"),
            Some("file.exe".to_string())
        );
        
        assert_eq!(
            extract_filename_from_url("https://example.com/"),
            None
        );
    }
}
