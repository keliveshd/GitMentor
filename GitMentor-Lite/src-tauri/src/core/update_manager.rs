use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/**
 * GitMentor 自动更新管理器
 * 作者：Evilek
 * 编写日期：2025-01-18
 */

/// GitHub Release 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
}

/// GitHub Release Asset 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub id: u64,
    pub name: String,
    pub content_type: String,
    pub size: u64,
    pub browser_download_url: String,
}

/// 版本信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: String,
    pub has_update: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
}

/// 下载进度回调
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// 更新管理器
pub struct UpdateManager {
    client: Client,
    repo_owner: String,
    repo_name: String,
    current_version: String,
}

impl UpdateManager {
    /// 创建新的更新管理器实例
    pub fn new(current_version: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60)) // 增加超时时间到60秒
            .user_agent("GitMentor-Updater/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            repo_owner: "keliveshd".to_string(),
            repo_name: "GitMentor".to_string(),
            current_version,
        }
    }

    /// 检查是否有新版本可用
    pub async fn check_for_updates(&self) -> Result<VersionInfo> {
        println!("[DEBUG] UpdateManager::check_for_updates() 开始执行");

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        println!("[DEBUG] 检查更新: 请求URL: {}", url);
        println!(
            "[DEBUG] 仓库信息: owner={}, name={}",
            self.repo_owner, self.repo_name
        );

        println!("[DEBUG] 开始发送 HTTP 请求...");
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|e| {
                println!("[ERROR] GitHub API 请求失败: {}", e);
                println!("[ERROR] 错误类型: {:?}", e);
                anyhow::anyhow!("网络请求失败: {}。请检查网络连接或稍后重试。", e)
            })?;

        println!("[DEBUG] GitHub API 响应状态: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            println!("[ERROR] GitHub API 错误响应: {}", error_text);
            return Err(anyhow::anyhow!(
                "GitHub API 请求失败: {} - {}",
                status,
                error_text
            ));
        }

        let release: GitHubRelease = response.json().await.map_err(|e| {
            println!("[ERROR] 解析 GitHub API 响应失败: {}", e);
            anyhow::anyhow!("解析更新信息失败: {}", e)
        })?;

        println!(
            "[DEBUG] 获取到 release 信息: tag={}, draft={}, prerelease={}",
            release.tag_name, release.draft, release.prerelease
        );

        // 跳过草稿和预发布版本
        if release.draft || release.prerelease {
            println!("[INFO] 跳过草稿或预发布版本: {}", release.tag_name);
            return Ok(VersionInfo {
                current: self.current_version.clone(),
                latest: self.current_version.clone(),
                has_update: false,
                download_url: None,
                release_notes: None,
                published_at: None,
            });
        }

        let latest_version = self.normalize_version(&release.tag_name);
        let current_version = self.normalize_version(&self.current_version);

        println!(
            "[DEBUG] 版本比较: 当前={}, 最新={}",
            current_version, latest_version
        );

        let has_update = self.compare_versions(&current_version, &latest_version) == Ordering::Less;

        println!("[DEBUG] 是否有更新: {}", has_update);

        // 查找 Windows MSI 安装包
        let download_url = self.find_windows_installer(&release.assets);

        println!("[DEBUG] 找到的下载链接: {:?}", download_url);

        Ok(VersionInfo {
            current: self.current_version.clone(),
            latest: release.tag_name,
            has_update,
            download_url,
            release_notes: Some(release.body),
            published_at: Some(release.published_at),
        })
    }

    /// 下载更新包
    pub async fn download_update(
        &self,
        download_url: &str,
        download_path: &PathBuf,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        let response = self.client.get(download_url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Download failed: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        // 确保下载目录存在
        if let Some(parent) = download_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = fs::File::create(download_path).await?;

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // 调用进度回调
            if let Some(ref callback) = progress_callback {
                callback(downloaded, total_size);
            }
        }

        file.flush().await?;
        Ok(())
    }

    /// 安装更新包（Windows MSI）
    pub async fn install_update(&self, installer_path: &PathBuf) -> Result<()> {
        if !installer_path.exists() {
            return Err(anyhow::anyhow!("Installer file not found"));
        }

        // 生成安装日志路径（与安装包同目录）
        let log_path = installer_path.with_file_name(format!(
            "{}-install.log",
            installer_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("GitMentorLite")
        ));

        // 首先尝试以当前权限运行 msiexec
        let direct_output = Self::run_msiexec(&log_path, installer_path, false).await?;
        if direct_output.status.success() {
            return Ok(());
        }

        let direct_reason =
            Self::format_install_error(&direct_output, &log_path, "MSI install failed".to_string());

        // 针对典型的权限问题尝试请求管理员权限重新安装
        if Self::should_retry_with_elevation(&direct_output) {
            let elevated_output = Self::run_msiexec(&log_path, installer_path, true).await?;

            if elevated_output.status.success() {
                return Ok(());
            }

            // 用户主动取消 UAC 提示
            if matches!(elevated_output.status.code(), Some(1223)) {
                let msg = format!(
                    "用户取消了管理员权限请求，安装未完成；安装日志: {}",
                    log_path.display()
                );
                return Err(anyhow::anyhow!(msg));
            }

            let elevated_reason = Self::format_install_error(
                &elevated_output,
                &log_path,
                "尝试以管理员身份安装失败".to_string(),
            );
            return Err(anyhow::anyhow!(elevated_reason));
        }

        Err(anyhow::anyhow!(direct_reason))
    }

    async fn run_msiexec(
        log_path: &PathBuf,
        installer_path: &PathBuf,
        elevated: bool,
    ) -> Result<std::process::Output> {
        #[cfg(target_os = "windows")]
        {
            if elevated {
                let installer_arg = installer_path.to_string_lossy().replace('\'', "''");
                let log_arg = log_path.to_string_lossy().replace('\'', "''");
                let script = format!(
                    "$process = Start-Process msiexec -ArgumentList '/i','{}','/quiet','/norestart','/L*v','{}' -Verb RunAs -WindowStyle Hidden -Wait -PassThru; exit $process.ExitCode",
                    installer_arg, log_arg
                );

                let output = tokio::process::Command::new("powershell")
                    .arg("-NoProfile")
                    .arg("-Command")
                    .arg(script)
                    .output()
                    .await
                    .map_err(|e| anyhow::anyhow!("启动提升权限安装失败: {}", e))?;

                return Ok(output);
            }
        }

        let output = tokio::process::Command::new("msiexec")
            .arg("/i")
            .arg(installer_path.as_os_str())
            .arg("/quiet")
            .arg("/norestart")
            .arg("/L*v")
            .arg(log_path.as_os_str())
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("执行 msiexec 失败: {}", e))?;

        Ok(output)
    }

    fn format_install_error(
        output: &std::process::Output,
        log_path: &PathBuf,
        prefix: String,
    ) -> String {
        let exit_code = output.status.code();
        let stdout_msg = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let combined = format!("{} {}", stderr_msg, stdout_msg).to_lowercase();

        let detail = match exit_code {
            Some(1603) => {
                if combined.contains("error 1730")
                    || combined.contains("administrator to remove")
                    || combined.contains("need to be an administrator")
                {
                    "MSI exit code 1603（权限不足）。检测到 Error 1730：需要管理员权限才能卸载旧版本，请接受管理员权限提示或以管理员身份运行 GitMentor Lite。"
                        .to_string()
                } else if combined.contains("error 1925")
                    || combined.contains("insufficient privileges")
                {
                    "MSI exit code 1603（权限不足）。检测到 Error 1925：当前用户权限不足，请以管理员身份重试。".to_string()
                } else if combined.contains("error 1303") {
                    "MSI exit code 1603。目录权限不足，请确认安装目录可写或使用管理员权限运行安装。"
                        .to_string()
                } else {
                    "MSI exit code 1603。请确认 GitMentor Lite 已关闭，并且具有足够的安装权限。"
                        .to_string()
                }
            }
            Some(1618) => {
                "MSI exit code 1618。另一个安装正在进行中，请完成当前安装后再试。".to_string()
            }
            Some(code) => format!("MSI exit code {}", code),
            None => "MSI installer was terminated before completing.".to_string(),
        };

        let mut reason = format!("{}：{}", prefix, detail);

        if !stderr_msg.is_empty() {
            reason.push_str(&format!("；错误输出: {}", stderr_msg));
        } else if !stdout_msg.is_empty() {
            reason.push_str(&format!("；输出: {}", stdout_msg));
        }

        reason.push_str(&format!("；安装日志: {}", log_path.display()));
        reason
    }

    fn should_retry_with_elevation(output: &std::process::Output) -> bool {
        if !cfg!(target_os = "windows") {
            return false;
        }

        let exit_code = output.status.code();
        let combined = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
        .to_lowercase();

        matches!(exit_code, Some(1603 | 1925 | 1303))
            || combined.contains("error 1730")
            || combined.contains("error 1925")
            || combined.contains("administrator to remove")
            || combined.contains("需要管理员权限")
            || combined.contains("insufficient privileges")
    }

    /// 规范化版本号（移除 v 前缀）
    fn normalize_version(&self, version: &str) -> String {
        version.trim_start_matches('v').to_string()
    }

    /// 比较两个版本号（语义化版本比较）
    fn compare_versions(&self, current: &str, latest: &str) -> Ordering {
        let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
        let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

        // 补齐版本号长度
        let max_len = current_parts.len().max(latest_parts.len());
        let mut current_padded = current_parts;
        let mut latest_padded = latest_parts;

        current_padded.resize(max_len, 0);
        latest_padded.resize(max_len, 0);

        current_padded.cmp(&latest_padded)
    }

    /// 查找 Windows 安装包
    fn find_windows_installer(&self, assets: &[GitHubAsset]) -> Option<String> {
        assets
            .iter()
            .find(|asset| {
                asset.name.ends_with(".msi")
                    || (asset.name.contains("windows") && asset.name.ends_with(".exe"))
            })
            .map(|asset| asset.browser_download_url.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let manager = UpdateManager::new("0.1.3".to_string());

        assert_eq!(manager.compare_versions("0.1.3", "0.1.4"), Ordering::Less);
        assert_eq!(manager.compare_versions("0.1.3", "0.1.3"), Ordering::Equal);
        assert_eq!(
            manager.compare_versions("0.1.4", "0.1.3"),
            Ordering::Greater
        );
    }

    #[test]
    fn test_normalize_version() {
        let manager = UpdateManager::new("0.1.3".to_string());

        assert_eq!(manager.normalize_version("v0.1.4"), "0.1.4");
        assert_eq!(manager.normalize_version("0.1.4"), "0.1.4");
    }
}
