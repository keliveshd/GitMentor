use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

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

    /// 安装更新包（支持 ZIP 便携版和 MSI）
    pub async fn install_update(&self, installer_path: &PathBuf) -> Result<()> {
        if !installer_path.exists() {
            return Err(anyhow::anyhow!("安装包文件未找到"));
        }

        let file_name = installer_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        println!("[DEBUG] 开始安装更新: {}", file_name);

        // 检测文件类型并选择安装方式
        if file_name.to_lowercase().ends_with(".zip") {
            // ZIP 文件 - 覆盖式更新
            // 首先尝试直接更新（如果文件未被锁定）
            match self.install_portable_zip(installer_path).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    // 如果直接更新失败，检查是否是文件锁定问题
                    let error_msg = e.to_string().to_lowercase();
                    if error_msg.contains("locked") || error_msg.contains("in use") || error_msg.contains("being used") {
                        println!("[DEBUG] 检测到文件锁定，使用更新器进程");
                        self.install_with_updater_process(installer_path).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if file_name.to_lowercase().ends_with(".msi") {
            // MSI 文件 - 传统安装
            self.install_msi(installer_path).await
        } else if file_name.to_lowercase().ends_with(".exe") {
            // EXE 文件 - 尝试直接运行或使用 msiexec
            let output = tokio::process::Command::new(installer_path)
                .arg("/S")
                .arg("/SILENT")
                .arg("/NORESTART")
                .output()
                .await
                .map_err(|e| anyhow::anyhow!("启动安装程序失败: {}", e))?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("安装失败: {}", error));
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "不支持的安装包格式: {}",
                file_name
            ))
        }
    }

    /// 使用更新器进程安装（解决文件锁定问题）
    async fn install_with_updater_process(&self, installer_path: &PathBuf) -> Result<()> {
        println!("[DEBUG] 启动更新器进程...");

        let current_exe = std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("获取当前可执行文件路径失败: {}", e))?;

        let app_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?;

        let installer_path_str = installer_path
            .to_string_lossy()
            .to_string();

        // 准备更新器进程的参数
        let updater_args = vec![
            "updater".to_string(),
            "--installer".to_string(),
            installer_path_str,
            "--app-dir".to_string(),
            app_dir.to_string_lossy().to_string(),
            "--exe-name".to_string(),
            current_exe
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("GitMentorLite.exe")
                .to_string(),
        ];

        println!("[DEBUG] 调用更新器进程...");
        println!("[DEBUG] 更新器参数: {:?}", updater_args);

        // 检查是否为便携版 ZIP（更新器只处理 ZIP）
        let is_zip = installer_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase().ends_with(".zip"))
            .unwrap_or(false);

        if !is_zip {
            return Err(anyhow::anyhow!("更新器进程仅支持 ZIP 文件"));
        }

        // 启动更新器进程（独立进程）
        let mut child = Command::new(&current_exe)
            .args(&updater_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("启动更新器进程失败: {}", e))?;

        let child_id = child.id();
        println!("[DEBUG] 更新器进程已启动，PID: {}", child_id);

        // 等待更新器进程完成（不应该等待太久，因为更新器会立即重启应用）
        // 使用 tokio 的 spawn_blocking 来运行同步的 wait()
        let child_wait = tokio::task::spawn_blocking(move || {
            child.wait()
        });

        match tokio::time::timeout(Duration::from_secs(5), child_wait).await {
            Ok(result) => {
                match result {
                    Ok(Ok(status)) => {
                        if status.success() {
                            println!("[DEBUG] 更新器进程完成");
                            Ok(())
                        } else {
                            Err(anyhow::anyhow!("更新器进程执行失败: {:?}", status))
                        }
                    }
                    Ok(Err(e)) => {
                        Err(anyhow::anyhow!("等待更新器进程失败: {}", e))
                    }
                    Err(e) => {
                        Err(anyhow::anyhow!("等待更新器进程超时: {}", e))
                    }
                }
            }
            Err(_) => {
                println!("[DEBUG] 更新器进程仍在运行，这是正常的（它会重启应用）");
                Ok(())
            }
        }
    }

    /// 安装 Portable ZIP 包（覆盖式更新）
    async fn install_portable_zip(&self, zip_path: &PathBuf) -> Result<()> {
        println!("[DEBUG] 开始安装 Portable ZIP 包");

        let current_exe = std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("获取当前可执行文件路径失败: {}", e))?;

        let current_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?;

        println!("[DEBUG] 应用程序目录: {:?}", current_dir);

        // 备份当前版本（可选）
        let backup_dir = current_dir.join("backup-old");
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir).await?;
        }

        // 克隆路径以避免生命周期问题
        let zip_path_clone = zip_path.to_owned();

        // 在同步块中处理 ZIP 文件（避免 Send 问题）
        let files_to_extract = tokio::task::spawn_blocking(move || -> Result<Vec<(PathBuf, Vec<u8>, bool)>, anyhow::Error> {
            // 打开 ZIP 文件
            let zip_file = std::fs::File::open(&zip_path_clone)
                .map_err(|e| anyhow::anyhow!("打开 ZIP 文件失败: {}", e))?;
            let mut archive = ZipArchive::new(zip_file)?;

            let total_files = archive.len();
            println!("[DEBUG] ZIP 文件包含 {} 个条目", total_files);

            let mut files = Vec::new();

            // 遍历 ZIP 文件中的所有条目
            for i in 0..total_files {
                let mut file = archive.by_index(i)?;
                let file_path = match file.enclosed_name() {
                    Some(path) => path.to_owned(),
                    None => continue,
                };

                let is_dir = file.is_dir();

                if is_dir {
                    files.push((file_path.to_path_buf(), Vec::new(), true));
                } else {
                    // 读取文件内容
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .map_err(|e| anyhow::anyhow!("读取 ZIP 文件内容失败: {}", e))?;
                    files.push((file_path.to_path_buf(), buffer, false));
                }

                if i % 10 == 0 || i == total_files - 1 {
                    println!("[DEBUG] 已读取: {}/{}", i + 1, total_files);
                }
            }

            Ok(files)
        }).await??;

        // 现在在异步上下文中写入文件
        for (file_path, buffer, is_dir) in files_to_extract {
            let target_path = current_dir.join(&file_path);

            if is_dir {
                if !target_path.exists() {
                    fs::create_dir_all(&target_path).await?;
                }
            } else {
                // 确保父目录存在
                if let Some(parent) = target_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).await?;
                    }
                }
                // 写入文件
                fs::write(&target_path, buffer).await?;
            }
        }

        println!("[DEBUG] ZIP 安装完成");
        Ok(())
    }

    /// 运行更新器进程（在独立进程中执行）
    pub async fn run_updater_process(
        &self,
        installer_path: &PathBuf,
        app_dir: &PathBuf,
        exe_name: &str,
    ) -> Result<()> {
        println!("[UPDATER] === 更新器进程开始 ===");
        println!("[UPDATER] 安装包: {:?}", installer_path);
        println!("[UPDATER] 应用目录: {:?}", app_dir);

        // 等待一段时间，确保主进程已经退出
        println!("[UPDATER] 等待主进程退出...");
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 备份旧版本（可选）
        let backup_dir = app_dir.join("backup-old");
        if backup_dir.exists() {
            println!("[UPDATER] 清理旧备份...");
            fs::remove_dir_all(&backup_dir).await?;
        }

        // 克隆路径以避免生命周期问题
        let installer_path_clone = installer_path.to_owned();

        // 在同步块中处理 ZIP 文件（避免 Send 问题）
        let files_to_extract = tokio::task::spawn_blocking(move || -> Result<Vec<(PathBuf, Vec<u8>, bool)>, anyhow::Error> {
            // 打开 ZIP 文件
            let zip_file = std::fs::File::open(&installer_path_clone)
                .map_err(|e| anyhow::anyhow!("打开 ZIP 文件失败: {}", e))?;
            let mut archive = ZipArchive::new(zip_file)?;

            let total_files = archive.len();
            println!("[UPDATER] ZIP 包含 {} 个条目", total_files);

            let mut files = Vec::new();

            // 遍历 ZIP 文件中的所有条目
            for i in 0..total_files {
                let mut file = archive.by_index(i)?;
                let file_path = match file.enclosed_name() {
                    Some(path) => path.to_owned(),
                    None => continue,
                };

                // 跳过更新器进程本身和备份目录
                if file_path.to_string_lossy().contains("backup-old") {
                    continue;
                }

                let is_dir = file.is_dir();

                if is_dir {
                    files.push((file_path.to_path_buf(), Vec::new(), true));
                } else {
                    // 读取文件内容
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .map_err(|e| anyhow::anyhow!("读取 ZIP 文件内容失败: {}", e))?;
                    files.push((file_path.to_path_buf(), buffer, false));
                }

                if i % 10 == 0 || i == total_files - 1 {
                    println!("[UPDATER] 已读取: {}/{}", i + 1, total_files);
                }
            }

            Ok(files)
        }).await??;

        // 在异步上下文中写入文件
        for (i, (file_path, buffer, is_dir)) in files_to_extract.iter().enumerate() {
            let target_path = app_dir.join(file_path);

            if *is_dir {
                if !target_path.exists() {
                    fs::create_dir_all(&target_path).await?;
                }
            } else {
                // 确保父目录存在
                if let Some(parent) = target_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).await?;
                    }
                }
                // 写入文件
                fs::write(&target_path, buffer).await?;
            }

            // 显示进度（每10个文件报告一次）
            if i % 10 == 0 || i == files_to_extract.len() - 1 {
                let progress = (i as f64 / files_to_extract.len() as f64 * 100.0).round();
                println!("[UPDATER] 进度: {}% ({}/{})", progress, i + 1, files_to_extract.len());
            }
        }

        println!("[UPDATER] === 文件解压完成 ===");

        // 验证更新是否成功
        let exe_path = app_dir.join(exe_name);
        if !exe_path.exists() {
            return Err(anyhow::anyhow!("更新后未找到可执行文件: {:?}", exe_path));
        }

        println!("[UPDATER] 更新验证成功");
        println!("[UPDATER] 更新器进程结束");
        Ok(())
    }

    /// 安装 MSI 包（传统方式）
    async fn install_msi(&self, installer_path: &PathBuf) -> Result<()> {
        println!("[DEBUG] 开始安装 MSI 包");

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

    /// 查找 Windows 安装包（优先 Portable zip）
    fn find_windows_installer(&self, assets: &[GitHubAsset]) -> Option<String> {
        // 首先查找便携版 zip（避免杀毒软件拦截）
        if let Some(asset) = assets.iter().find(|asset| {
            asset.name.to_lowercase().contains("portable")
                && (asset.name.ends_with(".zip") || asset.name.ends_with(".7z"))
        }) {
            println!("[DEBUG] 找到便携版: {}", asset.name);
            return Some(asset.browser_download_url.clone());
        }

        // 其次查找普通 zip 包
        if let Some(asset) = assets.iter().find(|asset| asset.name.ends_with(".zip")) {
            println!("[DEBUG] 找到 ZIP 包: {}", asset.name);
            return Some(asset.browser_download_url.clone());
        }

        // 最后回退到 MSI
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
