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

/// 测试网络连接命令
#[command]
pub async fn test_network_connection() -> Result<bool, String> {
    println!("[DEBUG] 开始网络连接测试...");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            println!("[ERROR] 创建HTTP客户端失败: {}", e);
            format!("创建HTTP客户端失败: {}", e)
        })?;

    println!("[DEBUG] HTTP客户端创建成功，开始请求 GitHub API...");

    // 测试连接到 GitHub API
    match client.get("https://api.github.com").send().await {
        Ok(response) => {
            let status = response.status();
            println!("[DEBUG] 网络连接测试收到响应: {}", status);
            let is_success = status.is_success();
            println!("[DEBUG] 响应是否成功: {}", is_success);

            if !is_success {
                println!("[WARN] GitHub API 返回非成功状态码: {}", status);
                // 即使状态码不是2xx，但能连接到GitHub就算网络正常
                Ok(true)
            } else {
                println!("[DEBUG] 网络连接测试完全成功");
                Ok(true)
            }
        }
        Err(e) => {
            println!("[ERROR] 网络连接测试失败: {}", e);
            println!("[ERROR] 错误详情: {:?}", e);

            // 检查是否是特定类型的错误
            if e.is_timeout() {
                println!("[ERROR] 错误类型: 超时");
            } else if e.is_connect() {
                println!("[ERROR] 错误类型: 连接失败");
            } else if e.is_request() {
                println!("[ERROR] 错误类型: 请求错误");
            }

            Err(format!("网络连接失败: {}", e))
        }
    }
}

/// 检查更新命令
#[command]
pub async fn check_for_updates() -> Result<VersionInfo, String> {
    println!("[DEBUG] ========== 开始检查更新 ==========");

    // 从 tauri.conf.json 读取当前版本
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    println!("[DEBUG] 当前版本: {}", current_version);

    println!("[DEBUG] 创建更新管理器...");
    let update_manager = UpdateManager::new(current_version);

    println!("[DEBUG] 调用更新管理器检查更新...");
    match update_manager.check_for_updates().await {
        Ok(version_info) => {
            println!("[DEBUG] 检查更新成功: {:?}", version_info);
            Ok(version_info)
        }
        Err(e) => {
            println!("[ERROR] 检查更新失败: {}", e);
            Err(format!("检查更新失败: {}", e))
        }
    }
}

/// 下载更新命令
#[command]
pub async fn download_update(
    app_handle: AppHandle,
    download_url: String,
) -> Result<String, String> {
    println!("[DEBUG] ========== 开始下载更新 ==========");
    println!("[DEBUG] 下载URL: {}", download_url);

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    println!("[DEBUG] 当前版本: {}", current_version);

    let update_manager = UpdateManager::new(current_version);
    println!("[DEBUG] 更新管理器创建成功");

    // 获取应用数据目录
    println!("[DEBUG] 获取应用数据目录...");
    let app_data_dir = match app_handle.path().app_data_dir() {
        Ok(dir) => {
            println!("[DEBUG] 应用数据目录: {:?}", dir);
            dir
        }
        Err(e) => {
            println!("[ERROR] 获取应用数据目录失败: {}", e);
            return Err(format!("获取应用数据目录失败: {}", e));
        }
    };

    // 创建下载路径
    let download_dir = app_data_dir.join("updates");
    println!("[DEBUG] 下载目录: {:?}", download_dir);

    let filename = extract_filename_from_url(&download_url)
        .unwrap_or_else(|| "GitMentor-update.zip".to_string());
    println!("[DEBUG] 文件名: {}", filename);

    let download_path = download_dir.join(&filename);
    println!("[DEBUG] 完整下载路径: {:?}", download_path);

    // 创建进度回调
    println!("[DEBUG] 创建进度回调...");
    let app_handle_clone = app_handle.clone();
    let progress_callback = Box::new(move |downloaded: u64, total: u64| {
        let percentage = if total > 0 {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "[DEBUG] 下载进度: {}/{} ({}%)",
            downloaded, total, percentage
        );

        let event = DownloadProgressEvent {
            downloaded,
            total,
            percentage,
        };

        // 发送进度事件到前端
        if let Err(e) = app_handle_clone.emit("download-progress", &event) {
            println!("[ERROR] 发送进度事件失败: {}", e);
        }
    });

    println!("[DEBUG] 开始调用更新管理器下载...");
    match update_manager
        .download_update(&download_url, &download_path, Some(progress_callback))
        .await
    {
        Ok(_) => {
            println!("[DEBUG] 下载完成成功");
            Ok(download_path.to_string_lossy().to_string())
        }
        Err(e) => {
            println!("[ERROR] 下载失败: {}", e);
            Err(format!("下载更新失败: {}", e))
        }
    }
}

/// 安装更新命令
#[command]
pub async fn install_update(installer_path: String) -> Result<(), String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let update_manager = UpdateManager::new(current_version);
    let path = PathBuf::from(installer_path);

    match update_manager.install_update(&path).await {
        Ok(_) => {
            println!("[DEBUG] 更新安装成功");
            Ok(())
        }
        Err(e) => {
            println!("[WARN] 直接更新失败，尝试延迟更新: {}", e);

            // 如果直接更新失败，尝试延迟更新
            let delayed_result = install_update_delayed(&path, &update_manager).await;

            match delayed_result {
                Ok(_) => {
                    println!("[DEBUG] 延迟更新已准备，重启后应用");
                    Ok(())
                }
                Err(delayed_e) => {
                    println!("[ERROR] 延迟更新也失败: {}", delayed_e);
                    Err(format!("安装更新失败: {}", e))
                }
            }
        }
    }
}

/// 延迟更新：在下次启动时应用更新
async fn install_update_delayed(zip_path: &PathBuf, update_manager: &UpdateManager) -> Result<(), String> {
    println!("[DEBUG] 准备延迟更新...");

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("获取当前可执行文件路径失败: {}", e))?;

    let app_dir = current_exe
        .parent()
        .ok_or_else(|| "无法获取可执行文件目录".to_string())?;

    // 创建 pending-update 目录
    let pending_dir = app_dir.join("pending-update");
    if !pending_dir.exists() {
        tokio::fs::create_dir_all(&pending_dir)
            .await
            .map_err(|e| format!("创建待更新目录失败: {}", e))?;
    }

    // 将 ZIP 文件复制到待更新目录
    let zip_file_name = zip_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("update.zip");

    let pending_zip = pending_dir.join(zip_file_name);
    tokio::fs::copy(zip_path, &pending_zip)
        .await
        .map_err(|e| format!("复制待更新文件失败: {}", e))?;

    // 创建标记文件
    let marker_file = pending_dir.join(".update-pending");
    tokio::fs::write(&marker_file, "pending")
        .await
        .map_err(|e| format!("创建更新标记失败: {}", e))?;

    println!("[DEBUG] 延迟更新已准备: {:?}", pending_zip);

    // 提示用户重启
    println!("[INFO] 更新已下载，将在下次重启时应用");

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

        serde_json::from_str(&content).map_err(|e| format!("解析更新设置失败: {}", e))
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
            extract_filename_from_url(
                "https://github.com/user/repo/releases/download/v1.0.0/app.msi"
            ),
            Some("app.msi".to_string())
        );

        assert_eq!(
            extract_filename_from_url("https://example.com/file.exe?param=value"),
            Some("file.exe".to_string())
        );

        assert_eq!(extract_filename_from_url("https://example.com/"), None);
    }
}
