use crate::types::git_types::{
    BranchInfo, CommitInfo, CommitRequest, DiffHunk, DiffLine, DiffLineType, DiffType,
    FileDiffRequest, FileDiffResult, FileStatus, FileStatusType, GitOperationResult,
    GitStatusResult, RevertRequest, RevertType, StageRequest,
};
use anyhow::{anyhow, Result};
use git2::{DiffOptions, Repository, Signature, StatusOptions};
use std::path::Path;

/// Git引擎，提供类似VSCode的Git功能
/// 作者：Evilek
pub struct GitEngine {
    repo_path: Option<String>,
}

impl GitEngine {
    pub fn new() -> Self {
        Self { repo_path: None }
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
    /// 编写日期：2025-08-04
    pub fn get_simple_file_diff(&self, file_path: &str) -> Result<String> {
        let repo_path = self
            .repo_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No repository opened"))?;

        let repo = Repository::open(repo_path)?;

        // 获取HEAD提交
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        let head_tree = head_commit.tree()?;

        // 获取工作目录状态
        let mut opts = DiffOptions::new();
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

    /// 获取当前仓库引用
    fn get_repository(&self) -> Result<Repository> {
        let repo_path = self
            .repo_path
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;
        Ok(Repository::open(repo_path)?)
    }

    /// 获取Git状态，类似VSCode Git面板的分类显示
    pub fn get_status(&self) -> Result<GitStatusResult> {
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
                        println!("暂存删除的文件: {}", file_path);
                        index.remove_path(path)?;
                        staged_count += 1;
                    }
                    (true, _) => {
                        // 文件存在：新增或修改的文件
                        println!("暂存存在的文件: {}", file_path);
                        match index.add_path(path) {
                            Ok(_) => staged_count += 1,
                            Err(e) => {
                                skipped_files.push(format!("{} (暂存失败: {})", file_path, e));
                            }
                        }
                    }
                    (false, false) => {
                        // 文件既不存在于工作目录也不存在于HEAD中，跳过
                        println!("跳过不存在的文件: {}", file_path);
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

    /// 获取文件差异
    /// 作者：Evilek
    /// 编写日期：2025-01-18
    pub fn get_file_diff(&self, request: &FileDiffRequest) -> Result<FileDiffResult> {
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
        println!("🔍 [GitEngine] 开始生成diff hunks");
        println!(
            "📋 [GitEngine] 输入参数: file_path={}, diff_type={:?}",
            file_path, diff_type
        );
        let mut diff_options = DiffOptions::new();
        diff_options.pathspec(file_path);
        diff_options.context_lines(3); // 设置上下文行数

        let diff = match diff_type {
            DiffType::WorkingTree => {
                println!("🔧 [GitEngine] 生成工作区与暂存区的差异");
                // 工作区与暂存区的差异
                let mut index = repo.index()?;
                let tree = index.write_tree()?;
                let tree = repo.find_tree(tree)?;
                println!("📊 [GitEngine] 暂存区tree ID: {}", tree.id());
                repo.diff_tree_to_workdir(Some(&tree), Some(&mut diff_options))?
            }
            DiffType::Staged => {
                println!("🔧 [GitEngine] 生成暂存区与HEAD的差异");
                // 暂存区与HEAD的差异
                let head = repo.head()?;
                let head_commit = head.peel_to_commit()?;
                let head_tree = head_commit.tree()?;
                let mut index = repo.index()?;
                let index_tree = index.write_tree()?;
                let index_tree = repo.find_tree(index_tree)?;
                println!(
                    "📊 [GitEngine] HEAD tree ID: {}, Index tree ID: {}",
                    head_tree.id(),
                    index_tree.id()
                );
                repo.diff_tree_to_tree(
                    Some(&head_tree),
                    Some(&index_tree),
                    Some(&mut diff_options),
                )?
            }
            DiffType::HeadToWorking => {
                println!("🔧 [GitEngine] 生成HEAD与工作区的差异");
                // HEAD与工作区的差异
                let head = repo.head()?;
                let head_commit = head.peel_to_commit()?;
                let head_tree = head_commit.tree()?;
                println!("📊 [GitEngine] HEAD tree ID: {}", head_tree.id());
                repo.diff_tree_to_workdir(Some(&head_tree), Some(&mut diff_options))?
            }
        };

        println!("📈 [GitEngine] Git diff操作完成，开始解析结果");

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

        println!("📊 [GitEngine] Diff解析完成统计:");
        println!("  总Hunk数量: {}", result.len());
        for (i, hunk) in result.iter().enumerate() {
            println!(
                "  Hunk {}: 老文件{}行起{}行, 新文件{}行起{}行, 包含{}行差异",
                i + 1,
                hunk.old_start,
                hunk.old_lines,
                hunk.new_start,
                hunk.new_lines,
                hunk.lines.len()
            );

            let mut context_count = 0;
            let mut delete_count = 0;
            let mut insert_count = 0;

            for line in &hunk.lines {
                match line.line_type {
                    DiffLineType::Context => context_count += 1,
                    DiffLineType::Delete => delete_count += 1,
                    DiffLineType::Insert => insert_count += 1,
                }
            }

            println!(
                "    行类型统计: 上下文{}行, 删除{}行, 新增{}行",
                context_count, delete_count, insert_count
            );

            if i == 0 && !hunk.lines.is_empty() {
                println!("    第一个hunk的前3行内容:");
                for (j, line) in hunk.lines.iter().take(3).enumerate() {
                    println!("      {}. {:?}: {}", j + 1, line.line_type, line.content);
                }
            }
        }

        if result.is_empty() {
            println!("⚠️ [GitEngine] 警告: 没有生成任何hunks，可能文件没有差异");
        }

        Ok(result)
    }
}
