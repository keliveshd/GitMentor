use crate::core::git_config::{GitConfig, GitExecutionMode};
use crate::debug_log;
use crate::types::git_types::{
    BranchInfo, CommitInfo, CommitRequest, DiffHunk, DiffLine, DiffLineType, DiffType,
    FileDiffRequest, FileDiffResult, FileStatus, FileStatusType, GitOperationResult,
    GitStatusResult, RevertRequest, RevertType, StageRequest,
};
use anyhow::{anyhow, Result};
use git2::{DiffOptions, Repository, Signature, StatusOptions};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Git执行方式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum GitMethod {
    SystemGit,  // 系统安装的Git命令
    BundledGit, // 内置的Git可执行文件
    Git2Api,    // Git2库API（最后备选）
}

/// Git引擎，提供类似VSCode的Git功能
/// 作者：Evilek
#[derive(Clone)]
pub struct GitEngine {
    repo_path: Option<String>,
    git_method: GitMethod,
    git_config: GitConfig,
}

impl GitEngine {
    pub fn new() -> Self {
        let git_config = GitConfig::default();
        let git_method = Self::determine_git_method(&git_config);
        debug_log!("[DEBUG] 检测到Git执行方式: {:?}", git_method);
        Self {
            repo_path: None,
            git_method,
            git_config,
        }
    }

    /// 使用指定配置创建GitEngine
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    pub fn new_with_config(git_config: GitConfig) -> Self {
        let git_method = Self::determine_git_method(&git_config);
        debug_log!("[DEBUG] 使用配置创建GitEngine，执行方式: {:?}", git_method);
        Self {
            repo_path: None,
            git_method,
            git_config,
        }
    }

    /// 更新Git配置
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    pub fn update_config(&mut self, git_config: GitConfig) {
        debug_log!("[DEBUG] 更新Git配置: {:?}", git_config.execution_mode);
        self.git_config = git_config.clone();
        self.git_method = Self::determine_git_method(&git_config);
        debug_log!("[DEBUG] 新的Git执行方式: {:?}", self.git_method);
    }

    /// 获取当前Git配置
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    pub fn get_config(&self) -> &GitConfig {
        &self.git_config
    }

    /// 根据配置确定Git执行方式
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    fn determine_git_method(config: &GitConfig) -> GitMethod {
        match config.execution_mode {
            GitExecutionMode::Auto => {
                debug_log!("[DEBUG] 配置为自动检测，开始检测流程");
                Self::detect_git_method()
            }
            GitExecutionMode::SystemGit => {
                debug_log!("[DEBUG] 配置强制使用系统Git");
                GitMethod::SystemGit
            }
            GitExecutionMode::BundledGit => {
                debug_log!("[DEBUG] 配置强制使用内置Git");
                GitMethod::BundledGit
            }
            GitExecutionMode::Git2Api => {
                debug_log!("[DEBUG] 配置强制使用Git2库API");
                GitMethod::Git2Api
            }
        }
    }

    /// 检测最佳的Git执行方式（自动模式）
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    fn detect_git_method() -> GitMethod {
        debug_log!("[DEBUG] ========================================");
        debug_log!("[DEBUG] 开始Git执行方式自动检测流程");
        debug_log!("[DEBUG] ========================================");

        // 1. 检测系统Git
        debug_log!("[DEBUG] 步骤1: 检测系统Git");
        if Self::is_system_git_available() {
            debug_log!("[SUCCESS] ✅ 检测到系统Git命令，优先使用");
            debug_log!("[DEBUG] ========================================");
            return GitMethod::SystemGit;
        } else {
            debug_log!("[WARN] ❌ 系统Git不可用，尝试内置Git");
        }

        // 2. 检测内置Git
        debug_log!("[DEBUG] 步骤2: 检测内置Git");
        if Self::is_bundled_git_available() {
            debug_log!("[SUCCESS] ✅ 检测到内置Git，使用备选方案");
            debug_log!("[DEBUG] ========================================");
            return GitMethod::BundledGit;
        } else {
            debug_log!("[WARN] ❌ 内置Git不可用，降级到Git2库API");
        }

        // 3. 降级到Git2库API
        debug_log!("[DEBUG] 步骤3: 降级到Git2库API");
        debug_log!("[WARN] ⚠️  系统Git和内置Git都不可用，使用Git2库API（功能受限）");
        debug_log!("[DEBUG] ========================================");
        GitMethod::Git2Api
    }

    /// 检测系统是否安装了Git命令
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    fn is_system_git_available() -> bool {
        debug_log!("[DEBUG] ==================== 开始检测系统Git ====================");
        debug_log!("[DEBUG] 当前工作目录: {:?}", std::env::current_dir());
        debug_log!("[DEBUG] PATH环境变量: {:?}", std::env::var("PATH"));

        // 尝试执行git --version命令
        debug_log!("[DEBUG] 执行命令: git --version");

        // 先尝试直接执行git命令
        debug_log!("[DEBUG] 尝试方式1: 直接执行 'git'");
        match Command::new("git").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let status_code = output.status.code();

                debug_log!("[DEBUG] 方式1执行成功");
                debug_log!("[DEBUG] 退出状态码: {:?}", status_code);
                debug_log!("[DEBUG] 标准输出: '{}'", version.trim());
                debug_log!("[DEBUG] 标准错误: '{}'", stderr.trim());
                debug_log!("[DEBUG] 状态成功: {}", output.status.success());
                debug_log!("[DEBUG] 版本非空: {}", !version.trim().is_empty());

                let success = output.status.success() && !version.trim().is_empty();
                if success {
                    debug_log!("[SUCCESS] 方式1成功，系统Git可用");
                    debug_log!("[DEBUG] ==================== 系统Git检测完成 ====================");
                    return true;
                } else {
                    debug_log!("[WARN] 方式1失败，尝试方式2");
                }
            }
            Err(e) => {
                debug_log!("[ERROR] 方式1执行失败: {}", e);
                debug_log!("[ERROR] 错误类型: {:?}", e.kind());
                debug_log!("[WARN] 尝试方式2: 使用完整路径");
            }
        }

        // 方式2: 尝试常见的Git安装路径
        let git_paths = vec![
            "C:\\Program Files\\Git\\bin\\git.exe",
            "C:\\Program Files (x86)\\Git\\bin\\git.exe",
            "D:\\Soft\\Git\\bin\\git.exe", // 用户的Git路径
            "git.exe",
        ];

        for git_path in git_paths {
            debug_log!("[DEBUG] 尝试路径: {}", git_path);
            match Command::new(git_path).arg("--version").output() {
                Ok(output) => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    debug_log!("[DEBUG] 路径 {} 执行成功", git_path);
                    debug_log!("[DEBUG] 退出状态码: {:?}", output.status.code());
                    debug_log!("[DEBUG] 标准输出: '{}'", version.trim());
                    debug_log!("[DEBUG] 标准错误: '{}'", stderr.trim());

                    if output.status.success() && !version.trim().is_empty() {
                        debug_log!("[SUCCESS] ✅ 找到可用的Git: {}", git_path);
                        debug_log!(
                            "[DEBUG] ==================== 系统Git检测完成 ===================="
                        );
                        return true;
                    }
                }
                Err(e) => {
                    debug_log!("[DEBUG] 路径 {} 失败: {}", git_path, e);
                }
            }
        }

        debug_log!("[ERROR] ❌ 所有Git路径都失败了");
        debug_log!("[DEBUG] ==================== 系统Git检测失败 ====================");
        false
    }

    /// 检测内置Git是否可用
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    fn is_bundled_git_available() -> bool {
        debug_log!("[DEBUG] ==================== 开始检测内置Git ====================");

        // 获取当前目标平台
        let target_triple = Self::get_target_triple();
        debug_log!("[DEBUG] 当前目标平台: {}", target_triple);

        // 构建Git二进制文件名
        let git_binary_name = if cfg!(windows) {
            format!("git-{}.exe", target_triple)
        } else {
            format!("git-{}", target_triple)
        };
        debug_log!("[DEBUG] 期望的Git二进制文件名: {}", git_binary_name);

        // 检查二进制文件是否存在
        // 在开发环境中，检查 src-tauri/binaries/ 目录
        // 在生产环境中，Tauri会自动处理sidecar的路径
        let binary_path = if cfg!(debug_assertions) {
            // 开发环境：检查binaries/目录（当前工作目录已经是src-tauri）
            let current_dir =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            debug_log!("[DEBUG] 当前工作目录: {}", current_dir.display());

            let path = current_dir.join("binaries").join(&git_binary_name);
            debug_log!("[DEBUG] 开发环境检查路径: {}", path.display());
            path
        } else {
            // 生产环境：Tauri会处理sidecar路径，这里只是检查逻辑
            // 实际使用时应该通过tauri::api::process::Command::sidecar来调用
            let path = std::path::PathBuf::from("binaries").join(&git_binary_name);
            debug_log!("[DEBUG] 生产环境检查路径: {}", path.display());
            path
        };

        debug_log!("[DEBUG] 检查文件是否存在: {}", binary_path.display());
        let exists = binary_path.exists();
        debug_log!("[DEBUG] 文件存在性检查结果: {}", exists);

        if exists {
            debug_log!("[DEBUG] 找到内置Git文件: {}", binary_path.display());

            // 获取文件元数据
            match std::fs::metadata(&binary_path) {
                Ok(metadata) => {
                    debug_log!("[DEBUG] 文件大小: {} bytes", metadata.len());
                    debug_log!("[DEBUG] 文件类型: {:?}", metadata.file_type());

                    // 检查文件是否可执行（Unix系统）
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let permissions = metadata.permissions();
                        let mode = permissions.mode();
                        let is_executable = mode & 0o111 != 0;
                        debug_log!("[DEBUG] 文件权限模式: {:o}", mode);
                        debug_log!("[DEBUG] 文件可执行性: {}", is_executable);

                        if !is_executable {
                            debug_log!(
                                "[WARN] 内置Git文件存在但不可执行: {}",
                                binary_path.display()
                            );
                            debug_log!(
                                "[DEBUG] ==================== 内置Git检测失败 ===================="
                            );
                            return false;
                        }
                    }

                    #[cfg(windows)]
                    {
                        debug_log!("[DEBUG] Windows系统，跳过可执行性检查");
                    }
                }
                Err(e) => {
                    debug_log!("[ERROR] 无法获取文件元数据: {}", e);
                    debug_log!("[DEBUG] ==================== 内置Git检测失败 ====================");
                    return false;
                }
            }

            debug_log!("[DEBUG] 内置Git检测成功");
            debug_log!("[DEBUG] ==================== 内置Git检测完成 ====================");
            true
        } else {
            debug_log!("[DEBUG] 未找到内置Git文件: {}", binary_path.display());

            // 列出binaries目录的内容，帮助调试
            let binaries_dir = if cfg!(debug_assertions) {
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("binaries")
            } else {
                std::path::PathBuf::from("binaries")
            };

            debug_log!("[DEBUG] 检查binaries目录: {}", binaries_dir.display());
            if binaries_dir.exists() {
                match std::fs::read_dir(&binaries_dir) {
                    Ok(entries) => {
                        debug_log!("[DEBUG] binaries目录内容:");
                        for entry in entries {
                            if let Ok(entry) = entry {
                                debug_log!("[DEBUG]   - {}", entry.file_name().to_string_lossy());
                            }
                        }
                    }
                    Err(e) => {
                        debug_log!("[DEBUG] 无法读取binaries目录: {}", e);
                    }
                }
            } else {
                debug_log!("[DEBUG] binaries目录不存在");
            }

            debug_log!("[DEBUG] ==================== 内置Git检测失败 ====================");
            false
        }
    }

    /// 获取当前目标平台的target triple
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    fn get_target_triple() -> String {
        // 在编译时确定的目标平台，使用条件编译来确定
        #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
        return "x86_64-pc-windows-msvc".to_string();

        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        return "x86_64-unknown-linux-gnu".to_string();

        #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
        return "x86_64-apple-darwin".to_string();

        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        return "aarch64-apple-darwin".to_string();

        #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
        return "aarch64-unknown-linux-gnu".to_string();

        // 默认返回通用标识符（只有在不匹配任何已知平台时才会执行）
        #[cfg(not(any(
            all(target_arch = "x86_64", target_os = "windows"),
            all(target_arch = "x86_64", target_os = "linux"),
            all(target_arch = "x86_64", target_os = "macos"),
            all(target_arch = "aarch64", target_os = "macos"),
            all(target_arch = "aarch64", target_os = "linux")
        )))]
        "unknown-target".to_string()
    }

    /// 使用Git命令获取状态（超快速）
    /// 作者：Evilek
    /// 编写日期：2025-08-06
    pub fn get_status_with_git_command(&self) -> Result<GitStatusResult> {
        debug_log!("[DEBUG] 使用Git命令获取状态...");
        let start_time = Instant::now();

        let repo_path = self
            .repo_path
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        debug_log!("[DEBUG] 当前仓库路径: {}", repo_path);
        debug_log!("[DEBUG] 当前工作目录: {:?}", std::env::current_dir());

        // 获取当前分支
        println!("[DEBUG] 获取当前分支...");
        let branch_start = Instant::now();
        let branch = self.get_current_branch_with_command(repo_path)?;
        println!(
            "[DEBUG] 分支获取完成: {}, 耗时: {:?}",
            branch,
            branch_start.elapsed()
        );

        // 获取暂存区文件
        println!("[DEBUG] 获取暂存区文件...");
        let staged_start = Instant::now();
        let staged_files = self.get_staged_files_with_command(repo_path)?;
        println!(
            "[DEBUG] 暂存区文件获取完成，找到 {} 个文件，耗时: {:?}",
            staged_files.len(),
            staged_start.elapsed()
        );

        // 获取工作区修改文件
        println!("[DEBUG] 获取工作区修改文件...");
        let unstaged_start = Instant::now();
        let unstaged_files = self.get_unstaged_files_with_command(repo_path)?;
        println!(
            "[DEBUG] 工作区文件获取完成，找到 {} 个文件，耗时: {:?}",
            unstaged_files.len(),
            unstaged_start.elapsed()
        );

        // 获取未跟踪文件（可选，可能较慢）
        println!("[DEBUG] 获取未跟踪文件...");
        let untracked_start = Instant::now();
        let untracked_files = self.get_untracked_files_with_command(repo_path)?;
        println!(
            "[DEBUG] 未跟踪文件获取完成，找到 {} 个文件，耗时: {:?}",
            untracked_files.len(),
            untracked_start.elapsed()
        );

        // 获取远程分支信息（简化）
        let (ahead, behind) = (0, 0); // TODO: 实现远程分支比较

        println!(
            "[DEBUG] Git命令状态获取完成，总耗时: {:?}",
            start_time.elapsed()
        );

        Ok(GitStatusResult {
            branch,
            has_changes: !staged_files.is_empty()
                || !unstaged_files.is_empty()
                || !untracked_files.is_empty(),
            staged_files,
            unstaged_files,
            untracked_files,
            conflicted_files: Vec::new(), // TODO: 实现冲突文件检测
            ahead,
            behind,
        })
    }

    /// 使用Git命令获取当前分支
    fn get_current_branch_with_command(&self, repo_path: &str) -> Result<String> {
        let git_command = self.get_git_command();
        debug_log!(
            "[DEBUG] 执行Git命令: {} symbolic-ref --short HEAD",
            git_command
        );
        debug_log!("[DEBUG] 在目录: {}", repo_path);

        let output = Command::new(&git_command)
            .current_dir(repo_path)
            .args(&["symbolic-ref", "--short", "HEAD"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug_log!("[DEBUG] symbolic-ref 退出状态: {:?}", output.status.code());
        debug_log!("[DEBUG] symbolic-ref 标准输出: '{}'", stdout.trim());
        debug_log!("[DEBUG] symbolic-ref 标准错误: '{}'", stderr.trim());

        if output.status.success() {
            let branch = stdout.trim().to_string();
            debug_log!("[DEBUG] 成功获取分支: {}", branch);
            Ok(branch)
        } else {
            debug_log!("[DEBUG] symbolic-ref失败，尝试rev-parse");
            // 可能是detached HEAD，尝试获取commit hash
            let output = Command::new(&git_command)
                .current_dir(repo_path)
                .args(&["rev-parse", "--short", "HEAD"])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug_log!("[DEBUG] rev-parse 退出状态: {:?}", output.status.code());
            debug_log!("[DEBUG] rev-parse 标准输出: '{}'", stdout.trim());
            debug_log!("[DEBUG] rev-parse 标准错误: '{}'", stderr.trim());

            if output.status.success() {
                let commit = stdout.trim().to_string();
                let result = format!("HEAD@{}", commit);
                debug_log!("[DEBUG] 获取到commit hash: {}", result);
                Ok(result)
            } else {
                debug_log!("[ERROR] 所有分支获取方式都失败了，返回unknown");
                Ok("unknown".to_string())
            }
        }
    }

    /// 获取Git命令路径（根据当前使用的Git方式）
    /// 作者：Evilek
    /// 编写日期：2025-08-07
    fn get_git_command(&self) -> String {
        match self.git_method {
            GitMethod::SystemGit => {
                // 尝试使用完整路径，解决"error launching git"问题
                if cfg!(windows) {
                    // 尝试常见的Git安装路径
                    let git_paths = vec![
                        "D:\\Soft\\Git\\bin\\git.exe", // 用户的Git路径
                        "C:\\Program Files\\Git\\bin\\git.exe",
                        "C:\\Program Files (x86)\\Git\\bin\\git.exe",
                        "git.exe",
                        "git",
                    ];

                    for path in git_paths {
                        // 简单检查文件是否存在（对于完整路径）
                        if path.contains(":\\") {
                            if std::path::Path::new(path).exists() {
                                debug_log!("[DEBUG] 使用Git路径: {}", path);
                                return path.to_string();
                            }
                        } else {
                            // 对于相对路径，直接返回
                            debug_log!("[DEBUG] 使用Git命令: {}", path);
                            return path.to_string();
                        }
                    }
                    "git".to_string() // 降级
                } else {
                    "git".to_string()
                }
            }
            GitMethod::BundledGit => {
                // 对于内置Git，返回完整的二进制文件名
                // 注意：在实际使用中，应该通过Tauri的sidecar API来调用
                let target_triple = Self::get_target_triple();
                if cfg!(windows) {
                    format!("git-{}.exe", target_triple)
                } else {
                    format!("git-{}", target_triple)
                }
            }
            GitMethod::Git2Api => "git".to_string(), // 降级到系统Git
        }
    }

    /// 使用Git命令获取暂存区文件
    fn get_staged_files_with_command(&self, repo_path: &str) -> Result<Vec<FileStatus>> {
        let git_command = self.get_git_command();
        let output = Command::new(&git_command)
            .current_dir(repo_path)
            .args(&["diff", "--cached", "--name-status"])
            .output()?;

        let mut staged_files = Vec::new();

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if let Some((status_char, file_path)) = line.split_once('\t') {
                    let status_type = match status_char {
                        "A" => FileStatusType::Added,
                        "M" => FileStatusType::Modified,
                        "D" => FileStatusType::Deleted,
                        "R" => FileStatusType::Renamed,
                        "C" => FileStatusType::Copied,
                        _ => FileStatusType::Modified,
                    };

                    staged_files.push(FileStatus {
                        path: file_path.to_string(),
                        working_tree_status: None,
                        index_status: Some(status_type),
                        selected: false,
                        is_staged: true,
                    });
                }
            }
        }

        Ok(staged_files)
    }

    /// 使用Git命令获取工作区修改文件
    fn get_unstaged_files_with_command(&self, repo_path: &str) -> Result<Vec<FileStatus>> {
        let git_command = self.get_git_command();
        let output = Command::new(&git_command)
            .current_dir(repo_path)
            .args(&["diff", "--name-status"])
            .output()?;

        let mut unstaged_files = Vec::new();

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if let Some((status_char, file_path)) = line.split_once('\t') {
                    let status_type = match status_char {
                        "M" => FileStatusType::Modified,
                        "D" => FileStatusType::Deleted,
                        _ => FileStatusType::Modified,
                    };

                    unstaged_files.push(FileStatus {
                        path: file_path.to_string(),
                        working_tree_status: Some(status_type),
                        index_status: None,
                        selected: false,
                        is_staged: false,
                    });
                }
            }
        }

        Ok(unstaged_files)
    }

    /// 使用Git命令获取未跟踪文件
    fn get_untracked_files_with_command(&self, repo_path: &str) -> Result<Vec<FileStatus>> {
        let git_command = self.get_git_command();
        let output = Command::new(&git_command)
            .current_dir(repo_path)
            .args(&["ls-files", "--others", "--exclude-standard"])
            .output()?;

        let mut untracked_files = Vec::new();

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if !line.trim().is_empty() {
                    untracked_files.push(FileStatus {
                        path: line.trim().to_string(),
                        working_tree_status: Some(FileStatusType::Untracked),
                        index_status: None,
                        selected: false,
                        is_staged: false,
                    });
                }
            }
        }

        Ok(untracked_files)
    }

    pub fn open_repository(&mut self, path: &str) -> Result<()> {
        let _repo = Repository::open(path)?;
        self.repo_path = Some(path.to_string());
        Ok(())
    }

    /// 获取当前仓库路径
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub fn get_repository_path(&self) -> Option<String> {
        self.repo_path.clone()
    }

    /// 获取单个文件的diff内容（简单版本）
    /// 作者：Evilek
    /// 编写日期：2025-08-05
    pub fn get_simple_file_diff(&self, file_path: &str) -> Result<String> {
        let repo_path = self
            .repo_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No repository opened"))?;

        let repo = Repository::open(repo_path)
            .map_err(|e| anyhow::anyhow!("无法打开Git仓库 {}: {}", repo_path, e))?;

        let head = repo
            .head()
            .map_err(|e| anyhow::anyhow!("无法获取HEAD引用: {}", e))?;

        let head_commit = head
            .peel_to_commit()
            .map_err(|e| anyhow::anyhow!("无法获取HEAD提交: {}", e))?;

        let head_tree = head_commit
            .tree()
            .map_err(|e| anyhow::anyhow!("无法获取HEAD树: {}", e))?;

        // 获取工作目录状态
        let mut opts = DiffOptions::new();
        opts.include_untracked(true);

        let diff = repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut opts))
            .map_err(|e| anyhow::anyhow!("创建diff失败: {}", e))?;

        // 首先列出所有diff中的文件
        let mut all_files = Vec::new();

        // 使用更安全的回调处理方式
        let foreach_result = diff.foreach(
            &mut |delta, _progress| {
                match delta.new_file().path() {
                    Some(path) => {
                        let delta_path = path.to_string_lossy().to_string();
                        all_files.push(delta_path.clone());
                        true // 继续处理
                    }
                    None => {
                        true // 继续处理，即使这个delta没有路径
                    }
                }
            },
            None,
            None,
            None,
        );

        if let Err(e) = foreach_result {
            return Err(anyhow::anyhow!("遍历diff文件列表失败: {}", e));
        }

        // 查找指定文件的diff
        let mut file_diff = String::new();
        let mut found_file = false;

        // 使用更安全的回调处理方式，分离文件查找和内容处理
        let diff_result = diff.foreach(
            &mut |delta, _progress| {
                match delta.new_file().path() {
                    Some(path) => {
                        let delta_path = path.to_string_lossy();

                        // 尝试多种路径匹配方式
                        let delta_path_str = delta_path.as_ref();
                        let is_match = delta_path_str == file_path
                            || delta_path_str.ends_with(file_path)
                            || file_path.ends_with(delta_path_str)
                            || delta_path_str.replace('\\', "/") == file_path.replace('\\', "/");

                        if is_match {
                            found_file = true;
                        }
                        true // 总是返回true，避免用户中断错误
                    }
                    None => {
                        true // 总是返回true，避免用户中断错误
                    }
                }
            },
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                // 只处理匹配文件的diff行
                if let Some(path) = delta.new_file().path() {
                    let delta_path = path.to_string_lossy();
                    let delta_path_str = delta_path.as_ref();
                    let is_match = delta_path_str == file_path
                        || delta_path_str.ends_with(file_path)
                        || file_path.ends_with(delta_path_str)
                        || delta_path_str.replace('\\', "/") == file_path.replace('\\', "/");

                    if !is_match {
                        return true; // 不是目标文件，跳过这行
                    }
                }

                match line.origin() {
                    '+' | '-' | ' ' => {
                        file_diff.push(line.origin());
                        if let Ok(content) = std::str::from_utf8(line.content()) {
                            file_diff.push_str(content);
                        }
                    }
                    _ => {
                        // 跳过非内容行
                    }
                }
                true // 总是返回true继续处理
            }),
        );

        if let Err(e) = diff_result {
            return Err(anyhow::anyhow!("获取文件diff内容失败: {}", e));
        }

        if !found_file {
            // 尝试备用方法：使用简化的路径匹配
            return self.get_simple_file_diff_fallback(file_path);
        }

        if file_diff.is_empty() {
            return self.get_simple_file_diff_fallback(file_path);
        }

        Ok(file_diff)
    }

    /// 备用的文件diff获取方法
    /// 作者：Evilek
    /// 编写日期：2025-08-05
    fn get_simple_file_diff_fallback(&self, file_path: &str) -> Result<String> {
        let repo_path = self
            .repo_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No repository opened"))?;

        let repo = Repository::open(repo_path)
            .map_err(|e| anyhow::anyhow!("无法打开Git仓库 {}: {}", repo_path, e))?;

        // 使用更简单的方法：直接比较HEAD和工作目录
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        let mut opts = DiffOptions::new();
        opts.pathspec(file_path); // 只处理指定文件
        opts.context_lines(3);

        let diff = repo.diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut opts))?;

        let mut file_diff = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = String::from_utf8_lossy(line.content());
            file_diff.push_str(&content);
            true
        })?;

        if file_diff.is_empty() {
            return Err(anyhow::anyhow!("文件没有变更内容: {}", file_path));
        }

        Ok(file_diff)
    }

    /// 获取当前仓库引用
    fn get_repository(&self) -> Result<Repository> {
        let repo_path = self
            .repo_path
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;
        Ok(Repository::open(repo_path)?)
    }

    /// 获取Git状态，类似VSCode Git面板的分类显示
    /// 智能选择最佳执行方式
    /// 作者：Evilek
    /// 编写日期：2025-08-06
    pub fn get_status(&self) -> Result<GitStatusResult> {
        println!("[DEBUG] 开始获取Git状态，使用方式: {:?}", self.git_method);

        match self.git_method {
            GitMethod::SystemGit | GitMethod::BundledGit => {
                // 优先使用Git命令（超快速）
                match self.get_status_with_git_command() {
                    Ok(result) => {
                        println!("[DEBUG] Git命令方式成功");
                        return Ok(result);
                    }
                    Err(e) => {
                        println!("[WARN] Git命令方式失败，降级到Git2库API: {}", e);
                        // 降级到Git2库API
                    }
                }
            }
            GitMethod::Git2Api => {
                println!("[DEBUG] 直接使用Git2库API");
            }
        }

        // 使用Git2库API作为备选方案
        self.get_status_with_git2_api()
    }

    /// 使用Git2库API获取状态（备选方案）
    fn get_status_with_git2_api(&self) -> Result<GitStatusResult> {
        let repo = self.get_repository()?;

        // 获取当前分支
        let head = repo.head()?;
        let branch = head.shorthand().unwrap_or("unknown").to_string();

        // 获取文件状态
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut opts))?;

        let mut staged_files = Vec::new();
        let mut unstaged_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut conflicted_files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status_flags = entry.status();

            // 解析工作区状态
            let working_tree_status = if status_flags.is_wt_modified() {
                Some(FileStatusType::Modified)
            } else if status_flags.is_wt_deleted() {
                Some(FileStatusType::Deleted)
            } else if status_flags.is_wt_new() {
                Some(FileStatusType::Untracked)
            } else {
                None
            };

            // 解析暂存区状态
            let index_status = if status_flags.is_index_modified() {
                Some(FileStatusType::Modified)
            } else if status_flags.is_index_new() {
                Some(FileStatusType::Added)
            } else if status_flags.is_index_deleted() {
                Some(FileStatusType::Deleted)
            } else {
                None
            };

            let file_status = FileStatus {
                path: path.clone(),
                working_tree_status: working_tree_status.clone(),
                index_status: index_status.clone(),
                selected: false,
                is_staged: index_status.is_some(),
            };

            // 分类文件
            if status_flags.is_conflicted() {
                conflicted_files.push(file_status);
            } else if index_status.is_some() {
                staged_files.push(file_status);
            } else if working_tree_status == Some(FileStatusType::Untracked) {
                untracked_files.push(file_status);
            } else if working_tree_status.is_some() {
                unstaged_files.push(file_status);
            }
        }

        // 获取远程分支信息（简化版本）
        let (ahead, behind) = self.get_ahead_behind_count(&repo).unwrap_or((0, 0));

        Ok(GitStatusResult {
            branch,
            has_changes: !staged_files.is_empty()
                || !unstaged_files.is_empty()
                || !untracked_files.is_empty(),
            staged_files,
            unstaged_files,
            untracked_files,
            conflicted_files,
            ahead,
            behind,
        })
    }

    /// 获取领先/落后远程分支的提交数
    fn get_ahead_behind_count(&self, _repo: &Repository) -> Result<(u32, u32)> {
        // 简化实现，实际项目中可以更详细地处理
        Ok((0, 0))
    }

    /// 暂存或取消暂存文件
    /// 作者：Evilek
    /// 编写日期：2025-01-25
    /// 更新日期：2025-01-29 (添加删除文件和大文件处理逻辑)
    pub fn stage_files(&self, request: &StageRequest) -> Result<GitOperationResult> {
        let repo = self.get_repository()?;
        let mut index = repo.index()?;

        if request.stage {
            // 暂存文件 - 需要区分不同类型的文件状态
            let mut staged_count = 0;
            let mut skipped_files = Vec::new();

            for file_path in &request.file_paths {
                let path = Path::new(file_path);

                // 检查文件是否存在于工作目录
                let file_exists = repo
                    .workdir()
                    .map(|workdir| workdir.join(path).exists())
                    .unwrap_or(false);

                // 检查文件是否在HEAD中存在
                let file_in_head = match repo.head() {
                    Ok(head) => match head.peel_to_commit() {
                        Ok(commit) => match commit.tree() {
                            Ok(tree) => tree.get_path(path).is_ok(),
                            Err(_) => false,
                        },
                        Err(_) => false,
                    },
                    Err(_) => false,
                };

                // 检查文件大小（如果文件存在）
                if file_exists {
                    if let Some(workdir) = repo.workdir() {
                        let full_path = workdir.join(path);
                        if let Ok(metadata) = std::fs::metadata(&full_path) {
                            let file_size = metadata.len();
                            // 如果文件大于5MB，跳过并记录
                            if file_size > 5 * 1024 * 1024 {
                                skipped_files.push(format!(
                                    "{} (文件过大: {:.1}MB)",
                                    file_path,
                                    file_size as f64 / (1024.0 * 1024.0)
                                ));
                                continue;
                            }
                        }
                    }
                }

                // 根据文件状态选择合适的暂存方法
                match (file_exists, file_in_head) {
                    (false, true) => {
                        // 文件被删除：从工作目录删除但在HEAD中存在
                        index.remove_path(path)?;
                        staged_count += 1;
                    }
                    (true, _) => {
                        // 文件存在：新增或修改的文件
                        match index.add_path(path) {
                            Ok(_) => staged_count += 1,
                            Err(e) => {
                                skipped_files.push(format!("{} (暂存失败: {})", file_path, e));
                            }
                        }
                    }
                    (false, false) => {
                        // 文件既不存在于工作目录也不存在于HEAD中，跳过
                        skipped_files.push(format!("{} (文件不存在)", file_path));
                    }
                }
            }

            index.write()?;

            let mut message = format!("Successfully staged {} file(s)", staged_count);
            if !skipped_files.is_empty() {
                message.push_str(&format!(", skipped {} file(s)", skipped_files.len()));
            }

            Ok(GitOperationResult {
                success: true,
                message,
                details: if skipped_files.is_empty() {
                    None
                } else {
                    Some(format!("跳过的文件:\n{}", skipped_files.join("\n")))
                },
            })
        } else {
            // 取消暂存文件 - 使用正确的reset方法
            let head = repo.head()?;
            let head_commit = head.peel_to_commit()?;

            // 将指定文件从暂存区重置到HEAD状态
            repo.reset_default(Some(head_commit.as_object()), request.file_paths.iter())?;

            Ok(GitOperationResult {
                success: true,
                message: format!("Successfully unstaged {} file(s)", request.file_paths.len()),
                details: None,
            })
        }
    }

    /// 提交更改
    pub fn commit(&self, request: &CommitRequest) -> Result<GitOperationResult> {
        let repo = self.get_repository()?;
        let mut index = repo.index()?;

        // 如果指定了文件，先暂存这些文件
        if !request.selected_files.is_empty() {
            for file_path in &request.selected_files {
                index.add_path(Path::new(file_path))?;
            }
            index.write()?;
        }

        // 创建提交
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        // 获取签名
        let signature = self.get_signature(&repo)?;

        // 获取父提交
        let parent_commit = if let Ok(head) = repo.head() {
            if let Some(target) = head.target() {
                Some(repo.find_commit(target)?)
            } else {
                None
            }
        } else {
            None
        };

        // 创建提交
        let commit_id = if let Some(parent) = parent_commit {
            if request.amend {
                // 修正上次提交
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &request.message,
                    &tree,
                    &[&parent],
                )?
            } else {
                // 普通提交
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &request.message,
                    &tree,
                    &[&parent],
                )?
            }
        } else {
            // 初始提交
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &request.message,
                &tree,
                &[],
            )?
        };

        Ok(GitOperationResult {
            success: true,
            message: format!("Commit created: {}", commit_id),
            details: Some(request.message.clone()),
        })
    }

    /// 获取Git签名
    fn get_signature(&self, repo: &Repository) -> Result<Signature> {
        // 尝试从配置获取用户信息
        let config = repo.config()?;
        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "GitMentor User".to_string());
        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "user@gitmentor.local".to_string());

        Ok(Signature::now(&name, &email)?)
    }

    /// 回滚文件更改
    pub fn revert_files(&self, request: &RevertRequest) -> Result<GitOperationResult> {
        let repo = self.get_repository()?;

        match request.revert_type {
            RevertType::WorkingTree => {
                // 回滚工作区更改到HEAD状态
                let mut checkout_builder = git2::build::CheckoutBuilder::new();
                checkout_builder.force();

                for file_path in &request.file_paths {
                    checkout_builder.path(file_path);
                }

                // 从HEAD检出文件
                let head = repo.head()?;
                let head_commit = head.peel_to_commit()?;
                let head_tree = head_commit.tree()?;

                repo.checkout_tree(head_tree.as_object(), Some(&mut checkout_builder))?;

                Ok(GitOperationResult {
                    success: true,
                    message: format!(
                        "Reverted {} file(s) in working tree",
                        request.file_paths.len()
                    ),
                    details: None,
                })
            }
            RevertType::Staged => {
                // 回滚暂存区更改 - 从索引中移除文件
                let mut index = repo.index()?;

                for file_path in &request.file_paths {
                    // 尝试移除文件，如果文件不在索引中则忽略错误
                    let _ = index.remove_path(Path::new(file_path));
                }

                index.write()?;

                Ok(GitOperationResult {
                    success: true,
                    message: format!(
                        "Reverted {} file(s) in staging area",
                        request.file_paths.len()
                    ),
                    details: None,
                })
            }
            RevertType::Commit => {
                // 回滚提交（简化实现）
                Ok(GitOperationResult {
                    success: false,
                    message: "Commit revert not implemented in MVP".to_string(),
                    details: Some("This feature will be available in future versions".to_string()),
                })
            }
        }
    }

    /// 获取提交历史
    pub fn get_commit_history(&self, limit: usize) -> Result<Vec<CommitInfo>> {
        let repo = self.get_repository()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();

        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            let oid = oid?;
            let commit = repo.find_commit(oid)?;

            let commit_info = CommitInfo {
                hash: oid.to_string(),
                short_hash: oid.to_string()[..8].to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("").to_string(),
                email: commit.author().email().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
                files_changed: Vec::new(), // 简化实现
            };

            commits.push(commit_info);
        }

        Ok(commits)
    }

    /// 获取分支列表
    pub fn get_branches(&self) -> Result<Vec<BranchInfo>> {
        let repo = self.get_repository()?;
        let branches = repo.branches(None)?;
        let mut branch_list = Vec::new();

        let current_branch = repo.head()?.shorthand().unwrap_or("").to_string();

        for branch in branches {
            let (branch, branch_type) = branch?;
            if let Some(name) = branch.name()? {
                let branch_info = BranchInfo {
                    name: name.to_string(),
                    is_current: name == current_branch,
                    is_remote: branch_type == git2::BranchType::Remote,
                    upstream: None, // 简化实现
                };
                branch_list.push(branch_info);
            }
        }

        Ok(branch_list)
    }

    /// 丢弃所有工作区更改
    pub fn discard_all_changes(&self) -> Result<GitOperationResult> {
        let repo = self.get_repository()?;

        // 重置工作区到HEAD状态
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force();
        checkout_builder.remove_untracked(true);

        repo.checkout_tree(head_tree.as_object(), Some(&mut checkout_builder))?;

        Ok(GitOperationResult {
            success: true,
            message: "Discarded all working tree changes".to_string(),
            details: None,
        })
    }

    /// 暂存所有更改
    pub fn stage_all_changes(&self) -> Result<GitOperationResult> {
        let repo = self.get_repository()?;
        let mut index = repo.index()?;

        // 添加所有修改的文件
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        Ok(GitOperationResult {
            success: true,
            message: "Staged all changes".to_string(),
            details: None,
        })
    }

    /// 取消暂存所有文件
    pub fn unstage_all_changes(&self) -> Result<GitOperationResult> {
        let repo = self.get_repository()?;
        let mut index = repo.index()?;

        // 重置索引到HEAD状态
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        repo.reset_default(Some(head_commit.as_object()), ["*"].iter())?;
        index.write()?;

        Ok(GitOperationResult {
            success: true,
            message: "Unstaged all changes".to_string(),
            details: None,
        })
    }

    /// 获取文件差异摘要，包含实际的差异内容
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    pub fn get_diff_summary(&self, file_paths: &[String]) -> Result<String> {
        let repo = self.get_repository()?;
        let mut diff_output = String::new();

        // 获取暂存区的差异（用于提交消息生成）
        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3); // 设置上下文行数

        // 如果指定了文件路径，只获取这些文件的差异
        if !file_paths.is_empty() {
            for file_path in file_paths {
                diff_options.pathspec(file_path);
            }
        }

        // 生成暂存区与HEAD的差异
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;
        let mut index = repo.index()?;
        let index_tree = index.write_tree()?;
        let index_tree = repo.find_tree(index_tree)?;

        let diff =
            repo.diff_tree_to_tree(Some(&head_tree), Some(&index_tree), Some(&mut diff_options))?;

        // 将diff转换为文本格式
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = String::from_utf8_lossy(line.content());
            diff_output.push_str(&content);
            true
        })?;

        // 如果没有差异内容，返回文件列表
        if diff_output.trim().is_empty() {
            diff_output = format!("Files to be committed:\n{}", file_paths.join("\n"));
        }

        Ok(diff_output)
    }

    /// 获取单个文件的diff内容（用于分层提交）
    /// 作者：Evilek
    /// 编写日期：2025-08-04
    #[allow(dead_code)]
    pub fn get_file_diff(&self, file_path: &str) -> Result<String> {
        let repo = self.get_repository()?;

        // 获取HEAD提交
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        // 获取工作目录状态
        let mut opts = git2::DiffOptions::new();
        opts.include_untracked(true);

        let diff = repo.diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut opts))?;

        // 查找指定文件的diff
        let mut file_diff = String::new();
        diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path() {
                    if path.to_string_lossy() == file_path {
                        return true; // 找到目标文件
                    }
                }
                false
            },
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                match line.origin() {
                    '+' | '-' | ' ' => {
                        file_diff.push(line.origin());
                        if let Ok(content) = std::str::from_utf8(line.content()) {
                            file_diff.push_str(content);
                        }
                    }
                    _ => {}
                }
                true
            }),
        )?;

        if file_diff.is_empty() {
            return Err(anyhow::anyhow!("No diff found for file: {}", file_path));
        }

        Ok(file_diff)
    }

    /// 获取文件差异（原有方法）
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    pub fn get_file_diff_detailed(&self, request: &FileDiffRequest) -> Result<FileDiffResult> {
        let repo = self.get_repository()?;
        let file_path = &request.file_path;

        // 检查文件是否为二进制文件
        let is_binary = self.is_binary_file(&repo, file_path)?;

        if is_binary {
            return Ok(FileDiffResult {
                file_path: file_path.clone(),
                old_content: None,
                new_content: None,
                old_file_name: Some(file_path.clone()),
                new_file_name: Some(file_path.clone()),
                file_language: None,
                hunks: vec![],
                is_binary: true,
                is_new_file: false,
                is_deleted_file: false,
            });
        }

        match request.diff_type {
            DiffType::WorkingTree => self.get_working_tree_diff(&repo, file_path),
            DiffType::Staged => self.get_staged_diff(&repo, file_path),
            DiffType::HeadToWorking => self.get_head_to_working_diff(&repo, file_path),
        }
    }

    /// 检查文件是否为二进制文件
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    fn is_binary_file(&self, repo: &Repository, file_path: &str) -> Result<bool> {
        let workdir = repo
            .workdir()
            .ok_or_else(|| anyhow!("Repository has no working directory"))?;
        let full_path = workdir.join(file_path);

        if !full_path.exists() {
            return Ok(false);
        }

        // 简单的二进制文件检测：检查文件扩展名
        let extension = full_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let binary_extensions = [
            "exe", "dll", "so", "dylib", "bin", "obj", "o", "a", "lib", "jpg", "jpeg", "png",
            "gif", "bmp", "ico", "svg", "mp3", "mp4", "avi", "mov", "wav", "flac", "zip", "rar",
            "7z", "tar", "gz", "bz2", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        ];

        Ok(binary_extensions.contains(&extension.to_lowercase().as_str()))
    }

    /// 获取工作区与暂存区的差异
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    fn get_working_tree_diff(&self, repo: &Repository, file_path: &str) -> Result<FileDiffResult> {
        use std::fs;

        let workdir = repo
            .workdir()
            .ok_or_else(|| anyhow!("Repository has no working directory"))?;
        let full_path = workdir.join(file_path);

        // 获取工作区文件内容
        let new_content = if full_path.exists() {
            Some(fs::read_to_string(&full_path)?)
        } else {
            None
        };

        // 获取暂存区文件内容，如果暂存区没有则从HEAD获取
        let index = repo.index()?;
        let old_content = if let Some(entry) = index.get_path(Path::new(file_path), 0) {
            // 暂存区有该文件
            let blob = repo.find_blob(entry.id)?;
            Some(String::from_utf8_lossy(blob.content()).to_string())
        } else {
            // 暂存区没有该文件，从HEAD获取
            match repo.head() {
                Ok(head) => {
                    let head_commit = head.peel_to_commit()?;
                    let head_tree = head_commit.tree()?;
                    if let Ok(entry) = head_tree.get_path(Path::new(file_path)) {
                        let blob = repo.find_blob(entry.id())?;
                        Some(String::from_utf8_lossy(blob.content()).to_string())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        };

        // 生成diff hunks
        let hunks = self.generate_diff_hunks(repo, file_path, DiffType::WorkingTree)?;

        let file_language = self.detect_file_language(file_path);
        let is_new_file = old_content.is_none() && new_content.is_some();
        let is_deleted_file = old_content.is_some() && new_content.is_none();

        Ok(FileDiffResult {
            file_path: file_path.to_string(),
            old_content,
            new_content,
            old_file_name: Some(file_path.to_string()),
            new_file_name: Some(file_path.to_string()),
            file_language,
            hunks,
            is_binary: false,
            is_new_file,
            is_deleted_file,
        })
    }

    /// 获取暂存区与HEAD的差异
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    fn get_staged_diff(&self, repo: &Repository, file_path: &str) -> Result<FileDiffResult> {
        // 获取HEAD文件内容
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        let old_content = if let Ok(entry) = head_tree.get_path(Path::new(file_path)) {
            let blob = repo.find_blob(entry.id())?;
            Some(String::from_utf8_lossy(blob.content()).to_string())
        } else {
            None
        };

        // 获取暂存区文件内容
        let index = repo.index()?;
        let new_content = if let Some(entry) = index.get_path(Path::new(file_path), 0) {
            let blob = repo.find_blob(entry.id)?;
            Some(String::from_utf8_lossy(blob.content()).to_string())
        } else {
            None
        };

        // 生成diff hunks
        let hunks = self.generate_diff_hunks(repo, file_path, DiffType::Staged)?;

        let file_language = self.detect_file_language(file_path);
        let is_new_file = old_content.is_none() && new_content.is_some();
        let is_deleted_file = old_content.is_some() && new_content.is_none();

        Ok(FileDiffResult {
            file_path: file_path.to_string(),
            old_content,
            new_content,
            old_file_name: Some(file_path.to_string()),
            new_file_name: Some(file_path.to_string()),
            file_language,
            hunks,
            is_binary: false,
            is_new_file,
            is_deleted_file,
        })
    }

    /// 获取HEAD与工作区的差异
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    fn get_head_to_working_diff(
        &self,
        repo: &Repository,
        file_path: &str,
    ) -> Result<FileDiffResult> {
        use std::fs;

        // 获取HEAD文件内容
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        let old_content = if let Ok(entry) = head_tree.get_path(Path::new(file_path)) {
            let blob = repo.find_blob(entry.id())?;
            Some(String::from_utf8_lossy(blob.content()).to_string())
        } else {
            None
        };

        // 获取工作区文件内容
        let workdir = repo
            .workdir()
            .ok_or_else(|| anyhow!("Repository has no working directory"))?;
        let full_path = workdir.join(file_path);

        let new_content = if full_path.exists() {
            Some(fs::read_to_string(&full_path)?)
        } else {
            None
        };

        // 生成diff hunks
        let hunks = self.generate_diff_hunks(repo, file_path, DiffType::HeadToWorking)?;

        let file_language = self.detect_file_language(file_path);
        let is_new_file = old_content.is_none() && new_content.is_some();
        let is_deleted_file = old_content.is_some() && new_content.is_none();

        Ok(FileDiffResult {
            file_path: file_path.to_string(),
            old_content,
            new_content,
            old_file_name: Some(file_path.to_string()),
            new_file_name: Some(file_path.to_string()),
            file_language,
            hunks,
            is_binary: false,
            is_new_file,
            is_deleted_file,
        })
    }

    /// 检测文件语言类型
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    fn detect_file_language(&self, file_path: &str) -> Option<String> {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "rs" => Some("rust".to_string()),
            "js" | "mjs" => Some("javascript".to_string()),
            "ts" => Some("typescript".to_string()),
            "vue" => Some("vue".to_string()),
            "py" => Some("python".to_string()),
            "java" => Some("java".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "c" => Some("c".to_string()),
            "h" | "hpp" => Some("c".to_string()),
            "cs" => Some("csharp".to_string()),
            "go" => Some("go".to_string()),
            "php" => Some("php".to_string()),
            "rb" => Some("ruby".to_string()),
            "swift" => Some("swift".to_string()),
            "kt" => Some("kotlin".to_string()),
            "scala" => Some("scala".to_string()),
            "html" | "htm" => Some("html".to_string()),
            "css" => Some("css".to_string()),
            "scss" | "sass" => Some("scss".to_string()),
            "less" => Some("less".to_string()),
            "json" => Some("json".to_string()),
            "xml" => Some("xml".to_string()),
            "yaml" | "yml" => Some("yaml".to_string()),
            "toml" => Some("toml".to_string()),
            "md" => Some("markdown".to_string()),
            "sh" | "bash" => Some("bash".to_string()),
            "ps1" => Some("powershell".to_string()),
            "sql" => Some("sql".to_string()),
            _ => None,
        }
    }

    /// 生成diff hunks
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    fn generate_diff_hunks(
        &self,
        repo: &Repository,
        file_path: &str,
        diff_type: DiffType,
    ) -> Result<Vec<DiffHunk>> {
        let mut diff_options = DiffOptions::new();
        diff_options.pathspec(file_path);
        diff_options.context_lines(3); // 设置上下文行数

        let diff = match diff_type {
            DiffType::WorkingTree => {
                // 工作区与暂存区的差异
                let mut index = repo.index()?;
                let tree = index.write_tree()?;
                let tree = repo.find_tree(tree)?;
                repo.diff_tree_to_workdir(Some(&tree), Some(&mut diff_options))?
            }
            DiffType::Staged => {
                // 暂存区与HEAD的差异
                let head = repo.head()?;
                let head_commit = head.peel_to_commit()?;
                let head_tree = head_commit.tree()?;
                let mut index = repo.index()?;
                let index_tree = index.write_tree()?;
                let index_tree = repo.find_tree(index_tree)?;
                repo.diff_tree_to_tree(
                    Some(&head_tree),
                    Some(&index_tree),
                    Some(&mut diff_options),
                )?
            }
            DiffType::HeadToWorking => {
                // HEAD与工作区的差异
                let head = repo.head()?;
                let head_commit = head.peel_to_commit()?;
                let head_tree = head_commit.tree()?;
                repo.diff_tree_to_workdir(Some(&head_tree), Some(&mut diff_options))?
            }
        };

        use std::cell::RefCell;
        use std::rc::Rc;

        let hunks = Rc::new(RefCell::new(Vec::new()));
        let current_hunk = Rc::new(RefCell::new(None::<DiffHunk>));
        let current_lines = Rc::new(RefCell::new(Vec::new()));
        let old_line_num = Rc::new(RefCell::new(0u32));
        let new_line_num = Rc::new(RefCell::new(0u32));

        let hunks_clone = hunks.clone();
        let current_hunk_clone = current_hunk.clone();
        let current_lines_clone = current_lines.clone();
        let old_line_num_clone = old_line_num.clone();
        let new_line_num_clone = new_line_num.clone();

        diff.print(git2::DiffFormat::Patch, move |_delta, hunk, line| {
            let content = String::from_utf8_lossy(line.content());

            match line.origin() {
                'H' => {
                    // Hunk header - 保存之前的hunk并开始新的hunk
                    if let Some(mut prev_hunk) = current_hunk_clone.borrow_mut().take() {
                        prev_hunk.lines = current_lines_clone.borrow().clone();
                        hunks_clone.borrow_mut().push(prev_hunk);
                        current_lines_clone.borrow_mut().clear();
                    }

                    if let Some(hunk) = hunk {
                        *current_hunk_clone.borrow_mut() = Some(DiffHunk {
                            old_start: hunk.old_start(),
                            old_lines: hunk.old_lines(),
                            new_start: hunk.new_start(),
                            new_lines: hunk.new_lines(),
                            lines: Vec::new(),
                        });
                        *old_line_num_clone.borrow_mut() = hunk.old_start();
                        *new_line_num_clone.borrow_mut() = hunk.new_start();
                    }
                }
                '+' => {
                    // 新增行
                    let new_line = *new_line_num_clone.borrow();
                    current_lines_clone.borrow_mut().push(DiffLine {
                        line_type: DiffLineType::Insert,
                        content: content.trim_end().to_string(),
                        old_line_number: None,
                        new_line_number: Some(new_line),
                    });
                    *new_line_num_clone.borrow_mut() += 1;
                }
                '-' => {
                    // 删除行
                    let old_line = *old_line_num_clone.borrow();
                    current_lines_clone.borrow_mut().push(DiffLine {
                        line_type: DiffLineType::Delete,
                        content: content.trim_end().to_string(),
                        old_line_number: Some(old_line),
                        new_line_number: None,
                    });
                    *old_line_num_clone.borrow_mut() += 1;
                }
                ' ' => {
                    // 上下文行
                    let old_line = *old_line_num_clone.borrow();
                    let new_line = *new_line_num_clone.borrow();
                    current_lines_clone.borrow_mut().push(DiffLine {
                        line_type: DiffLineType::Context,
                        content: content.trim_end().to_string(),
                        old_line_number: Some(old_line),
                        new_line_number: Some(new_line),
                    });
                    *old_line_num_clone.borrow_mut() += 1;
                    *new_line_num_clone.borrow_mut() += 1;
                }
                _ => {
                    // 忽略其他类型的行（如文件头）
                }
            }
            true
        })?;

        // 保存最后一个hunk
        if let Some(mut last_hunk) = current_hunk.borrow_mut().take() {
            last_hunk.lines = current_lines.borrow().clone();
            hunks.borrow_mut().push(last_hunk);
        }

        let result = hunks.borrow().clone();
        Ok(result)
    }
}
