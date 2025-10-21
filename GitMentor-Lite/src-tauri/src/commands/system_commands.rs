use tauri::{command, AppHandle};
use tauri_plugin_opener::OpenerExt;

/**
 * 系统相关命令
 * 作者：Evilek
 * 编写日期：2025-08-21
 */

/// 打开浏览器链接
#[command]
pub async fn open_browser_url(app: AppHandle, url: String) -> Result<(), String> {
    println!("[DEBUG] 尝试打开浏览器链接: {}", url);

    // 验证URL格式
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("无效的URL格式".to_string());
    }

    // 使用tauri-plugin-opener打开链接
    match app.opener().open_url(url.clone(), None::<&str>) {
        Ok(_) => {
            println!("[DEBUG] 成功打开浏览器链接: {}", url);
            Ok(())
        }
        Err(e) => {
            println!("[ERROR] 打开浏览器链接失败: {}", e);
            Err(format!("打开浏览器失败: {}", e))
        }
    }
}

/// 获取应用信息
#[command]
pub async fn get_app_info() -> Result<AppInfo, String> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let name = env!("CARGO_PKG_NAME").to_string();
    let description = env!("CARGO_PKG_DESCRIPTION").to_string();
    let authors = env!("CARGO_PKG_AUTHORS").to_string();

    Ok(AppInfo {
        name,
        version,
        description,
        authors,
        repository_url: "https://github.com/keliveshd/GitMentor".to_string(),
        license: "GPL-3.0".to_string(),
    })
}

/// 应用信息结构
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: String,
    pub repository_url: String,
    pub license: String,
}
