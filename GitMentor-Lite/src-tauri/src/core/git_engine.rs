use git2::{Repository, StatusOptions, Signature};
use anyhow::{Result, anyhow};
use std::path::Path;
use crate::types::git_types::{
    FileStatus, GitStatusResult, FileStatusType, CommitInfo,
    StageRequest, RevertRequest, RevertType, BranchInfo,
    GitOperationResult, CommitRequest
};

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

    /// 获取当前仓库引用
    fn get_repository(&self) -> Result<Repository> {
        let repo_path = self.repo_path.as_ref()
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
            has_changes: !staged_files.is_empty() || !unstaged_files.is_empty() || !untracked_files.is_empty(),
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
    pub fn stage_files(&self, request: &StageRequest) -> Result<GitOperationResult> {
        let repo = self.get_repository()?;
        let mut index = repo.index()?;

        for file_path in &request.file_paths {
            if request.stage {
                // 暂存文件
                index.add_path(Path::new(file_path))?;
            } else {
                // 取消暂存文件 - 简化实现，直接从索引中移除
                index.remove_path(Path::new(file_path))?;
            }
        }

        index.write()?;

        Ok(GitOperationResult {
            success: true,
            message: format!(
                "Successfully {} {} file(s)",
                if request.stage { "staged" } else { "unstaged" },
                request.file_paths.len()
            ),
            details: None,
        })
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
        let name = config.get_string("user.name").unwrap_or_else(|_| "GitMentor User".to_string());
        let email = config.get_string("user.email").unwrap_or_else(|_| "user@gitmentor.local".to_string());

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
                    message: format!("Reverted {} file(s) in working tree", request.file_paths.len()),
                    details: None,
                })
            },
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
                    message: format!("Reverted {} file(s) in staging area", request.file_paths.len()),
                    details: None,
                })
            },
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

    pub fn get_diff_summary(&self, file_paths: &[String]) -> Result<String> {
        let _repo = self.get_repository()?;

        // 简化的差异摘要，实际项目中可以更详细
        let mut diff_output = String::new();

        for file_path in file_paths {
            diff_output.push_str(&format!("File: {}\n", file_path));
        }

        Ok(diff_output)
    }
}
