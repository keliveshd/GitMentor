use crate::core::git_config::{GitConfig, GitExecutionMode};

use crate::debug_log;

use crate::types::git_types::{

    BranchInfo, CommitInfo, CommitRequest, DiffHunk, DiffLine, DiffLineType, DiffType,

    FileDiffRequest, FileDiffResult, FileStatus, FileStatusType, GitOperationResult,

    GitStatusResult, GitflowActionRequest, GitflowBranchInfo, GitflowBranchStatus,

    GitflowBranchType, GitflowConfig, GitflowCreateRequest, GitflowDivergence, GitflowSummary,

    RevertRequest, RevertType, StageRequest,

};

use anyhow::{anyhow, Result};

use chrono::{FixedOffset, Utc};

use git2::{BranchType, DiffOptions, Repository, Signature, StatusOptions};

use notify::{

    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,

};

use serde::Serialize;

use std::fmt::Write;

use std::path::{Path, PathBuf};

use std::process::Command;

use std::sync::{Arc, Mutex as StdMutex};

use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};



/// Git执行方式枚举

#[derive(Debug, Clone, PartialEq)]

pub enum GitMethod {

    SystemGit,  // 系统安装的Git命令

    BundledGit, // 内置的Git可执行文件

    Git2Api,    // Git2库API（最后备选）

}



/// Git引擎，提供类似VSCode的Git功能

/// 作者：Evilek

pub struct GitEngine {

    repo_path: Option<String>,

    git_method: GitMethod,

    git_config: GitConfig,

    git_path: Option<String>, // 缓存检测到的Git路径

    repo_watcher: Option<RepoWatcherHandle>,

}



impl Clone for GitEngine {

    fn clone(&self) -> Self {

        Self {

            repo_path: self.repo_path.clone(),

            git_method: self.git_method.clone(),

            git_config: self.git_config.clone(),

            git_path: self.git_path.clone(),

            repo_watcher: None,

        }

    }

}



#[derive(Debug)]

struct RepoWatcherHandle {

    watcher: RecommendedWatcher,

}



#[derive(Debug, Serialize, Clone)]

struct GitStatusDirtyPayload {

    repository: String,

    #[serde(rename = "eventKind")]

    event_kind: String,

}



impl GitEngine {

    #[allow(dead_code)]

    pub fn new() -> Self {

        let git_config = GitConfig::default();

        let git_method = Self::determine_git_method(&git_config);

        let git_path = Self::detect_git_path();

        debug_log!(

            "[DEBUG] 检测到Git执行方式: {:?}, Git路径: {:?}",

            git_method,

            git_path

        );

        Self {

            repo_path: None,

            git_method,

            git_config,

            git_path,

            repo_watcher: None,

        }

    }



    /// 使用指定配置创建GitEngine

    /// 作者：Evilek

    /// 编写日期：2025-08-07

    pub fn new_with_config(git_config: GitConfig) -> Self {

        let git_method = Self::determine_git_method(&git_config);

        let git_path = Self::detect_git_path();

        debug_log!(

            "[DEBUG] 使用配置创建GitEngine，执行方式: {:?}, Git路径: {:?}",

            git_method,

            git_path

        );

        Self {

            repo_path: None,

            git_method,

            git_config,

            git_path,

            repo_watcher: None,

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

    #[allow(dead_code)]

    pub fn get_config(&self) -> &GitConfig {

        &self.git_config

    }



    pub fn start_repo_watcher(&mut self, app_handle: AppHandle) -> Result<()> {

        let repo_path = self

            .repo_path

            .clone()

            .ok_or_else(|| anyhow!("No repository opened"))?;



        self.stop_repo_watcher();



        let repo_path_buf = PathBuf::from(&repo_path);

        let repo_path_for_event = Arc::new(repo_path);

        let debounce_for_cb: Arc<StdMutex<Option<Instant>>> = Arc::new(StdMutex::new(None));

        let app_handle_for_event = app_handle;



        debug_log!("[DEBUG] 启动仓库文件监控: {}", repo_path_for_event.as_ref());



        let mut watcher = RecommendedWatcher::new(

            move |res: Result<Event, notify::Error>| match res {

                Ok(event) => {

                    if !GitEngine::should_emit_event(&event.kind) {

                        return;

                    }



                    let mut should_emit = true;

                    if let Ok(mut guard) = debounce_for_cb.lock() {

                        let now = Instant::now();

                        if let Some(prev) = *guard {

                            if now.duration_since(prev) < Duration::from_millis(400) {

                                should_emit = false;

                            }

                        }

                        if should_emit {

                            *guard = Some(now);

                        }

                    }



                    if !should_emit {

                        return;

                    }



                    let payload = GitStatusDirtyPayload {

                        repository: repo_path_for_event.as_ref().clone(),

                        event_kind: format!("{:?}", event.kind),

                    };



                    if let Err(err) = app_handle_for_event.emit("git-status::dirty", payload) {

                        debug_log!("[DEBUG] git-status::dirty 事件发送失败: {}", err);

                    }

                }

                Err(err) => {

                    debug_log!("[DEBUG] 仓库文件监控出现错误: {}", err);

                }

            },

            NotifyConfig::default().with_poll_interval(Duration::from_secs(2)),

        )

        .map_err(|e| anyhow!("Failed to start repository watcher: {}", e))?;



        watcher

            .watch(&repo_path_buf, RecursiveMode::Recursive)

            .map_err(|e| anyhow!("Failed to watch repository directory: {}", e))?;



        self.repo_watcher = Some(RepoWatcherHandle { watcher });



        Ok(())

    }



    pub fn stop_repo_watcher(&mut self) {

        if self.repo_watcher.is_some() {

            debug_log!("[DEBUG] 停止仓库文件监控");

        }

        self.repo_watcher = None;

    }



    pub fn close_repository(&mut self) {

        self.stop_repo_watcher();

        self.repo_path = None;

    }



    pub fn list_gitflow_branches(&self) -> Result<GitflowSummary> {

        let repo_path = self

            .repo_path

            .as_ref()

            .ok_or_else(|| anyhow!("No repository opened"))?;

        let repo = Repository::open(repo_path)?;

        let config = self.get_gitflow_config();



        let head_name = repo

            .head()

            .ok()

            .and_then(|head| head.shorthand().map(|s| s.to_string()));



        let mut branches = Vec::new();

        let mut branch_iter = repo.branches(Some(BranchType::Local))?;



        while let Some(branch_result) = branch_iter.next() {

            let (branch, _) = branch_result?;

            let branch_name_opt = branch.name()?;

            let branch_name = match branch_name_opt {

                Some(name) => name.to_string(),

                None => continue,

            };



            if branch_name == config.develop_branch || branch_name == config.main_branch {

                continue;

            }



            let branch_type = match Self::classify_gitflow_branch(&branch_name, &config) {

                Some(branch_type) => branch_type,

                None => continue,

            };



            let upstream = branch

                .upstream()

                .ok()

                .and_then(|up| up.name().ok().flatten().map(|s| s.to_string()));



            let reference = branch.into_reference();

            let target = match reference.target() {

                Some(oid) => oid,

                None => continue,

            };



            let commit = repo

                .find_commit(target)

                .map_err(|e| anyhow!("Failed to read branch commit: {}", e))?;

            let last_updated_at = Some(Self::format_git_time(commit.time()));

            let created_at = last_updated_at.clone();

            let latest_commit = commit.summary().map(|s| s.to_string());



            let base_branch_name = Self::resolve_base_branch(&branch_type, &config);

            let divergence =

                if let Ok(base_branch) = repo.find_branch(&base_branch_name, BranchType::Local) {

                    if let Some(base_target) = base_branch.into_reference().target() {

                        let (ahead, behind) = repo

                            .graph_ahead_behind(target, base_target)

                            .unwrap_or((0, 0));

                        GitflowDivergence {

                            ahead: ahead as u32,

                            behind: behind as u32,

                        }

                    } else {

                        GitflowDivergence::default()

                    }

                } else {

                    GitflowDivergence::default()

                };



            let status = Self::infer_gitflow_status(&branch_type, &divergence);



            branches.push(GitflowBranchInfo {

                id: branch_name.clone(),

                name: branch_name.clone(),

                branch_type,

                base: base_branch_name,

                status,

                created_at,

                last_updated_at,

                latest_commit,

                divergence,

                upstream,

                is_current: head_name

                    .as_ref()

                    .map(|current| current == &branch_name)

                    .unwrap_or(false),

            });

        }



        branches.sort_by(|a, b| a.name.cmp(&b.name));



        Ok(GitflowSummary { config, branches })

    }



    fn checkout_branch_internal(&self, repo_path: &str, branch: &str) -> Result<()> {

        let git_command = self.get_git_command();

        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(["checkout", branch])

            .output()?;



        if !output.status.success() {

            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(anyhow!("切换分支失败: {}", stderr.trim()));

        }



        Ok(())

    }



    fn merge_branch_into(&self, repo_path: &str, source: &str, target: &str) -> Result<()> {

        let repo = Repository::open(repo_path)?;

        if repo.find_branch(target, BranchType::Local).is_err() {

            return Err(anyhow!(format!(

                "未找到目标分支 {}，请确认 Gitflow 配置中的基线分支名称。",

                target

            )));

        }

        if repo.find_branch(source, BranchType::Local).is_err() {

            return Err(anyhow!(format!(

                "未找到源分支 {}，请先拉取或创建该分支。",

                source

            )));

        }



        self.checkout_branch_internal(repo_path, target)?;

        let git_command = self.get_git_command();

        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(["merge", "--no-ff", source])

            .output()?;



        if !output.status.success() {

            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(anyhow!(

                "合并 {} 到 {} 失败: {}",

                source,

                target,

                stderr.trim()

            ));

        }



        Ok(())

    }



    fn get_current_branch_name(&self, repo_path: &str) -> Result<String> {

        let git_command = self.get_git_command();

        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(["rev-parse", "--abbrev-ref", "HEAD"])

            .output()?;



        if !output.status.success() {

            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(anyhow!("无法获取当前分支: {}", stderr.trim()));

        }



        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())

    }



    pub fn execute_gitflow_action(

        &self,

        request: &GitflowActionRequest,

    ) -> Result<GitOperationResult> {

        let repo_path = self

            .repo_path

            .as_ref()

            .ok_or_else(|| anyhow!("No repository opened"))?;

        let config = self.get_gitflow_config();

        let current_branch = self.get_current_branch_name(repo_path)?;

        let branch_type = Self::classify_gitflow_branch(&request.branch_name, &config);



        let result = (|| -> Result<GitOperationResult> {

            match request.action.as_str() {

                "finish_feature" => {

                    self.merge_branch_into(

                        repo_path,

                        &request.branch_name,

                        &config.develop_branch,

                    )?;

                    Ok(GitOperationResult {

                        success: true,

                        message: format!(

                            "已将 {} 合并回 {}",

                            request.branch_name, config.develop_branch

                        ),

                        details: Some(

                            "已完成 Feature 分支回流，可继续创建 PR 或删除分支".to_string(),

                        ),

                    })

                }

                "finish_bugfix" => {

                    self.merge_branch_into(

                        repo_path,

                        &request.branch_name,

                        &config.develop_branch,

                    )?;

                    Ok(GitOperationResult {

                        success: true,

                        message: format!("已完成缺陷分支 {} 的回流", request.branch_name),

                        details: Some(format!(

                            "分支已合并到 {}，请确认测试通过",

                            config.develop_branch

                        )),

                    })

                }

                "finish_release" => {

                    let repo = Repository::open(repo_path)?;

                    if repo

                        .find_branch(&request.branch_name, BranchType::Local)

                        .is_err()

                    {

                        return Err(anyhow!(format!(

                            "当前仓库未找到分支 {}，请刷新后重试。",

                            request.branch_name

                        )));

                    }

                    drop(repo);



                    let push_details =

                        self.push_branch_with_upstream(repo_path, &request.branch_name)?;



                    Ok(GitOperationResult {

                        success: true,

                        message: format!("已将 {} 推送至远程", request.branch_name),

                        details: Some(push_details),

                    })

                }

                "close_release_local" => {

                    let repo = Repository::open(repo_path)?;

                    if repo

                        .find_branch(&request.branch_name, BranchType::Local)

                        .is_err()

                    {

                        return Err(anyhow!(format!(

                            "当前仓库未找到分支 {}，请刷新后重试。",

                            request.branch_name

                        )));

                    }

                    drop(repo);



                    self.merge_branch_into(repo_path, &request.branch_name, &config.main_branch)?;

                    if config.develop_branch != config.main_branch {

                        self.merge_branch_into(

                            repo_path,

                            &request.branch_name,

                            &config.develop_branch,

                        )?;

                    }



                    let target_branch = if config.develop_branch != config.main_branch {

                        &config.develop_branch

                    } else {

                        &config.main_branch

                    };

                    self.checkout_branch_internal(repo_path, target_branch)?;

                    let _ = self.delete_local_branch(repo_path, &request.branch_name);



                    Ok(GitOperationResult {

                        success: true,

                        message: format!(

                            "release 分支 {} 已在本地合并并关闭",

                            request.branch_name

                        ),

                        details: Some(

                            "已清理本地 release 分支，未执行远端 push，如需同步请手动推送主干分支。"

                                .to_string(),

                        ),

                    })

                }

                "finalize_release" => {

                    let repo = Repository::open(repo_path)?;

                    if repo

                        .find_branch(&request.branch_name, BranchType::Local)

                        .is_err()

                    {

                        return Err(anyhow!(format!(

                            "当前仓库未找到分支 {}，请确认 release 分支仍然存在。",

                            request.branch_name

                        )));

                    }

                    drop(repo);



                    self.merge_branch_into(repo_path, &request.branch_name, &config.main_branch)?;

                    if config.develop_branch != config.main_branch {

                        self.merge_branch_into(

                            repo_path,

                            &request.branch_name,

                            &config.develop_branch,

                        )?;

                    }



                    self.push_branch_to_remote(repo_path, &config.main_branch, false)?;

                    if config.develop_branch != config.main_branch {

                        self.push_branch_to_remote(repo_path, &config.develop_branch, false)?;

                    }



                    let target_branch = if config.develop_branch != config.main_branch {

                        &config.develop_branch

                    } else {

                        &config.main_branch

                    };

                    self.checkout_branch_internal(repo_path, target_branch)?;

                    let _ = self.delete_remote_branch(repo_path, &request.branch_name);

                    let _ = self.delete_local_branch(repo_path, &request.branch_name);



                    Ok(GitOperationResult {

                        success: true,

                        message: format!("发布分支 {} 已合并至主干并完成清理", request.branch_name),

                        details: Some(format!(

                            "已推送更新至远程，并删除 release 分支 {}",

                            request.branch_name

                        )),

                    })

                }

                "finish_hotfix" => {

                    self.merge_branch_into(repo_path, &request.branch_name, &config.main_branch)?;

                    let backport_targets = if config.develop_branch != config.main_branch {

                        self.merge_branch_into(

                            repo_path,

                            &request.branch_name,

                            &config.develop_branch,

                        )?;

                        format!("{} 与 {}", config.main_branch, config.develop_branch)

                    } else {

                        config.main_branch.clone()

                    };

                    Ok(GitOperationResult {

                        success: true,

                        message: format!(

                            "热修分支 {} 已回流至 {}",

                            request.branch_name, backport_targets

                        ),

                        details: Some("已同步热修补丁，请确认线上监控恢复".to_string()),

                    })

                }

                "sync_with_base" => {

                    let branch_kind = branch_type

                        .ok_or_else(|| anyhow!("无法识别分支类型: {}", request.branch_name))?;

                    let base_branch = Self::resolve_base_branch(&branch_kind, &config);

                    let repo = Repository::open(repo_path)?;

                    if repo.find_branch(&base_branch, BranchType::Local).is_err() {

                        return Err(anyhow!(format!(

                            "基线分支 {} 不存在，请先创建或在 Gitflow 设置中调整基线名称。",

                            base_branch

                        )));

                    }

                    if repo

                        .find_branch(&request.branch_name, BranchType::Local)

                        .is_err()

                    {

                        return Err(anyhow!(format!(

                            "当前仓库未找到分支 {}，请刷新 Gitflow 仪表盘后重试。",

                            request.branch_name

                        )));

                    }

                    drop(repo);



                    // 更新基线分支到最新

                    self.checkout_branch_internal(repo_path, &base_branch)?;

                    let _ = self.fetch_remote(Some("origin"))?;

                    let _ = self.pull_current_branch()?;



                    // 合并最新基线到目标分支

                    self.merge_branch_into(repo_path, &base_branch, &request.branch_name)?;



                    Ok(GitOperationResult {

                        success: true,

                        message: format!(

                            "已同步 {} 的最新提交到 {}",

                            base_branch, request.branch_name

                        ),

                        details: Some("建议在本地验证后再继续开发或创建 PR".to_string()),

                    })

                }

                "generate_status_report" => {

                    let digest =

                        self.build_branch_digest(repo_path, &request.branch_name, "状态播报")?;

                    Ok(GitOperationResult {

                        success: true,

                        message: format!("已生成 {} 的状态播报草稿", request.branch_name),

                        details: Some(digest),

                    })

                }

                "create_pull_request" => {

                    let push_details =

                        self.push_branch_with_upstream(repo_path, &request.branch_name)?;

                    let remote_url = self.get_remote_url(repo_path, "origin")?;

                    let pr_hint = remote_url

                        .as_deref()

                        .and_then(|url| Self::build_pr_url_hint(url, &request.branch_name));



                    let mut details = String::new();

                    if let Some(pr_url) = pr_hint {

                        write!(&mut details, "PR 链接建议：{}\n", pr_url).ok();

                    }

                    if !push_details.is_empty() {

                        write!(&mut details, "Git 推送输出：{}", push_details).ok();

                    }



                    Ok(GitOperationResult {

                        success: true,

                        message: format!("分支 {} 已推送远程，可继续创建 PR", request.branch_name),

                        details: if details.is_empty() {

                            None

                        } else {

                            Some(details)

                        },

                    })

                }

                "update_qa_status" => Ok(GitOperationResult {

                    success: true,

                    message: format!("已记录 {} 的 QA 状态更新", request.branch_name),

                    details: Some("请在详情栏同步最新回归结果与风险说明".to_string()),

                }),

                "backport_to_develop" => {

                    self.merge_branch_into(

                        repo_path,

                        &request.branch_name,

                        &config.develop_branch,

                    )?;

                    Ok(GitOperationResult {

                        success: true,

                        message: format!(

                            "已将热修 {} 回流到 {}",

                            request.branch_name, config.develop_branch

                        ),

                        details: Some("请确认 develop CI 通过后再清理热修分支".to_string()),

                    })

                }

                "generate_postmortem" => {

                    let digest =

                        self.build_branch_digest(repo_path, &request.branch_name, "事故复盘")?;

                    Ok(GitOperationResult {

                        success: true,

                        message: format!("已生成 {} 的复盘草稿", request.branch_name),

                        details: Some(digest),

                    })

                }

                "request_code_review" => {

                    let push_details =

                        self.push_branch_with_upstream(repo_path, &request.branch_name)?;

                    Ok(GitOperationResult {

                        success: true,

                        message: format!("已准备好 {} 的代码评审", request.branch_name),

                        details: Some(format!("建议在评审描述中引用最新推送：{}", push_details)),

                    })

                }

                "generate_retrospective" => {

                    let digest =

                        self.build_branch_digest(repo_path, &request.branch_name, "复盘提纲")?;

                    Ok(GitOperationResult {

                        success: true,

                        message: format!("已生成 {} 的复盘提纲", request.branch_name),

                        details: Some(digest),

                    })

                }

                other => Err(anyhow!("未支持的 Gitflow 操作: {}", other)),

            }

        })();



        // 尝试切回原分支

        if self

            .checkout_branch_internal(repo_path, &current_branch)

            .is_err()

        {

            let _ = self.checkout_branch_internal(repo_path, &config.develop_branch);

            if config.develop_branch != config.main_branch {

                let _ = self.checkout_branch_internal(repo_path, &config.main_branch);

            }

        }



        result

    }



    fn build_branch_digest(

        &self,

        repo_path: &str,

        branch_name: &str,

        template_label: &str,

    ) -> Result<String> {

        let commits = self.collect_recent_commits(repo_path, branch_name, 5)?;

        let divergence = self.describe_branch_divergence(branch_name);



        let mut buffer = String::new();

        write!(&mut buffer, "{}：{}\n", template_label, branch_name).ok();



        if let Some(div_text) = divergence {

            write!(&mut buffer, "{}\n", div_text).ok();

        }



        write!(&mut buffer, "最近提交：\n").ok();

        for line in commits {

            write!(&mut buffer, "{}\n", line).ok();

        }



        buffer.push_str("建议：检查待办和差异，必要时更新 QA 清单或准备 PR。\n");

        Ok(buffer)

    }



    fn collect_recent_commits(

        &self,

        repo_path: &str,

        branch_name: &str,

        limit: usize,

    ) -> Result<Vec<String>> {

        let git_command = self.get_git_command();

        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args([

                "log",

                branch_name,

                "--max-count",

                &limit.to_string(),

                "--pretty=format:%h|%s|%an|%ar",

            ])

            .output()?;



        if !output.status.success() {

            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(anyhow!("获取提交摘要失败: {}", stderr.trim()));

        }



        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut lines = Vec::new();

        for raw in stdout.lines() {

            if raw.trim().is_empty() {

                continue;

            }

            let parts: Vec<&str> = raw.split('|').collect();

            if parts.len() >= 4 {

                lines.push(format!(

                    "- {} \"{}\" · {} · {}",

                    parts[0],

                    parts[1].trim(),

                    parts[2].trim(),

                    parts[3].trim()

                ));

            } else {

                lines.push(format!("- {}", raw.trim()));

            }

        }



        if lines.is_empty() {

            lines.push("- 最近没有新的提交".to_string());

        }



        Ok(lines)

    }



    fn describe_branch_divergence(&self, branch_name: &str) -> Option<String> {

        if let Ok(summary) = self.list_gitflow_branches() {

            if let Some(info) = summary.branches.into_iter().find(|b| b.name == branch_name) {

                let ahead = info.divergence.ahead;

                let behind = info.divergence.behind;

                if ahead > 0 || behind > 0 {

                    return Some(format!(

                        "差异：领先 {} 提交，落后 {} 提交（基线：{}）",

                        ahead, behind, info.base

                    ));

                }

            }

        }

        None

    }



    fn push_branch_with_upstream(&self, repo_path: &str, branch_name: &str) -> Result<String> {

        let git_command = self.get_git_command();



        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(["push", "-u", "origin", branch_name])

            .output()?;



        if output.status.success() {

            let stdout = String::from_utf8_lossy(&output.stdout);

            let stderr = String::from_utf8_lossy(&output.stderr);

            let details = if !stderr.trim().is_empty() {

                stderr.trim().to_string()

            } else {

                stdout.trim().to_string()

            };

            Ok(details)

        } else {

            let stderr = String::from_utf8_lossy(&output.stderr);

            Err(anyhow!("推送分支失败: {}", stderr.trim()))

        }

    }



    fn push_branch_to_remote(

        &self,

        repo_path: &str,

        branch_name: &str,

        force: bool,

    ) -> Result<String> {

        let git_command = self.get_git_command();

        let mut args = vec!["push", "origin"];

        if force {

            args.push("--force-with-lease");

        }

        args.push(branch_name);



        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(&args)

            .output()?;



        if output.status.success() {

            let stdout = String::from_utf8_lossy(&output.stdout);

            let stderr = String::from_utf8_lossy(&output.stderr);

            let details = if !stderr.trim().is_empty() {

                stderr.trim().to_string()

            } else {

                stdout.trim().to_string()

            };

            Ok(details)

        } else {

            let stderr = String::from_utf8_lossy(&output.stderr);

            Err(anyhow!(format!(

                "推送分支 {} 失败: {}",

                branch_name,

                stderr.trim()

            )))

        }

    }



    fn delete_remote_branch(&self, repo_path: &str, branch_name: &str) -> Result<()> {

        let git_command = self.get_git_command();

        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(["push", "origin", "--delete", branch_name])

            .output()?;



        if output.status.success() {

            return Ok(());

        }



        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("remote ref does not exist") || stderr.contains("unknown revision") {

            return Ok(());

        }



        Err(anyhow!(format!(

            "删除远程分支 {} 失败: {}",

            branch_name,

            stderr.trim()

        )))

    }



    fn delete_local_branch(&self, repo_path: &str, branch_name: &str) -> Result<()> {

        let git_command = self.get_git_command();

        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(["branch", "-D", branch_name])

            .output()?;



        if output.status.success() {

            return Ok(());

        }



        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("not found") || stderr.contains("Unknown branch") {

            return Ok(());

        }



        Err(anyhow!(format!(

            "删除本地分支 {} 失败: {}",

            branch_name,

            stderr.trim()

        )))

    }



    fn get_remote_url(&self, repo_path: &str, remote: &str) -> Result<Option<String>> {

        let git_command = self.get_git_command();

        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(["remote", "get-url", remote])

            .output()?;



        if !output.status.success() {

            return Ok(None);

        }



        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if url.is_empty() {

            Ok(None)

        } else {

            Ok(Some(url))

        }

    }



    fn build_pr_url_hint(remote_url: &str, branch_name: &str) -> Option<String> {

        let cleaned = remote_url.trim().trim_end_matches(".git");

        let encoded_branch = Self::encode_branch_for_url(branch_name);



        if cleaned.contains("github.com") {

            let repo_path = if cleaned.starts_with("git@github.com:") {

                cleaned.trim_start_matches("git@github.com:").to_string()

            } else if let Some(idx) = cleaned.find("github.com/") {

                cleaned[idx + "github.com/".len()..].to_string()

            } else {

                return None;

            };

            Some(format!(

                "https://github.com/{}/compare/{}?expand=1",

                repo_path, encoded_branch

            ))

        } else if cleaned.contains("gitlab.com") {

            let repo_path = if cleaned.starts_with("git@gitlab.com:") {

                cleaned.trim_start_matches("git@gitlab.com:").to_string()

            } else if let Some(idx) = cleaned.find("gitlab.com/") {

                cleaned[idx + "gitlab.com/".len()..].to_string()

            } else {

                return None;

            };

            Some(format!(

                "https://gitlab.com/{}/-/merge_requests/new?merge_request[source_branch]={}",

                repo_path, encoded_branch

            ))

        } else {

            None

        }

    }



    fn encode_branch_for_url(branch: &str) -> String {

        let mut encoded = String::new();

        for ch in branch.chars() {

            match ch {

                '/' => encoded.push_str("%2F"),

                ' ' => encoded.push_str("%20"),

                '#' => encoded.push_str("%23"),

                '?' => encoded.push_str("%3F"),

                '%' => encoded.push_str("%25"),

                '+' => encoded.push_str("%2B"),

                _ => encoded.push(ch),

            }

        }

        encoded

    }



    pub fn create_gitflow_branch(

        &self,

        request: &GitflowCreateRequest,

    ) -> Result<GitOperationResult> {

        let repo_path = self

            .repo_path

            .as_ref()

            .ok_or_else(|| anyhow!("No repository opened"))?;

        let repo = Repository::open(repo_path)?;

        let branch_name = request.branch_name.trim();

        if branch_name.is_empty() {

            return Err(anyhow!("分支名称不能为空"));

        }



        if repo.find_branch(branch_name, BranchType::Local).is_ok() {

            return Err(anyhow!("分支 {} 已存在", branch_name));

        }



        let config = self.get_gitflow_config();

        let default_base = Self::resolve_base_branch(&request.branch_type, &config);

        let base_branch_name = request

            .base_branch

            .as_ref()

            .filter(|name| !name.trim().is_empty())

            .cloned()

            .unwrap_or(default_base.clone());



        let base_branch = repo

            .find_branch(&base_branch_name, BranchType::Local)

            .map_err(|_| anyhow!("找不到基线分支 {}", base_branch_name))?;

        let base_commit = base_branch

            .into_reference()

            .peel_to_commit()

            .map_err(|e| anyhow!("无法获取基线分支提交: {}", e))?;



        repo.branch(branch_name, &base_commit, false)

            .map_err(|e| anyhow!("创建分支失败: {}", e))?;



        if request.auto_push {

            let git_command = self.get_git_command();

            let output = Self::create_hidden_command(&git_command)

                .current_dir(repo_path)

                .args(["push", "-u", "origin", branch_name])

                .output()

                .map_err(|e| anyhow!("推送分支失败: {}", e))?;



            if !output.status.success() {

                let stderr = String::from_utf8_lossy(&output.stderr);

                return Err(anyhow!("推送分支失败: {}", stderr.trim()));

            }

        }



        Ok(GitOperationResult {

            success: true,

            message: format!("已创建分支 {}", branch_name),

            details: Some(format!(

                "基线：{}；类型：{:?}",

                base_branch_name, request.branch_type

            )),

        })

    }



    fn get_gitflow_config(&self) -> GitflowConfig {

        let mut config = GitflowConfig::default();



        if let Ok(repo) = self.get_repository() {

            config.main_branch =

                Self::detect_branch_name(&repo, &config.main_branch, &["main", "master", "trunk"]);



            config.develop_branch = Self::detect_branch_name(

                &repo,

                &config.develop_branch,

                &[

                    "develop",

                    "development",

                    "dev",

                    config.main_branch.as_str(),

                    "master",

                ],

            );

        }



        config

    }



    fn detect_branch_name(repo: &Repository, preferred: &str, fallbacks: &[&str]) -> String {

        if repo.find_branch(preferred, BranchType::Local).is_ok() {

            return preferred.to_string();

        }

        for candidate in fallbacks {

            if repo.find_branch(candidate, BranchType::Local).is_ok() {

                return candidate.to_string();

            }

        }



        if let Ok(head) = repo.head() {

            if let Some(name) = head.shorthand() {

                if repo.find_branch(name, BranchType::Local).is_ok() {

                    return name.to_string();

                }

            }

        }



        preferred.to_string()

    }



    fn classify_gitflow_branch(

        branch_name: &str,

        config: &GitflowConfig,

    ) -> Option<GitflowBranchType> {

        if branch_name.starts_with(&config.feature_prefix) {

            Some(GitflowBranchType::Feature)

        } else if branch_name.starts_with(&config.release_prefix) {

            Some(GitflowBranchType::Release)

        } else if branch_name.starts_with(&config.bugfix_prefix) {

            Some(GitflowBranchType::Bugfix)

        } else if branch_name.starts_with(&config.hotfix_prefix) {

            Some(GitflowBranchType::Hotfix)

        } else {

            None

        }

    }



    fn resolve_base_branch(branch_type: &GitflowBranchType, config: &GitflowConfig) -> String {

        match branch_type {

            GitflowBranchType::Hotfix => config.main_branch.clone(),

            _ => config.develop_branch.clone(),

        }

    }



    fn infer_gitflow_status(

        branch_type: &GitflowBranchType,

        divergence: &GitflowDivergence,

    ) -> GitflowBranchStatus {

        if divergence.ahead == 0 && divergence.behind == 0 {

            match branch_type {

                GitflowBranchType::Feature | GitflowBranchType::Bugfix => GitflowBranchStatus::Idle,

                GitflowBranchType::Release | GitflowBranchType::Hotfix => {

                    GitflowBranchStatus::AwaitingMerge

                }

            }

        } else if divergence.behind > 0 {

            GitflowBranchStatus::AwaitingMerge

        } else {

            GitflowBranchStatus::InProgress

        }

    }



    fn format_git_time(time: git2::Time) -> String {

        let seconds = time.seconds();

        let offset_minutes = time.offset_minutes();

        let offset = FixedOffset::east_opt(offset_minutes * 60)

            .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());



        if let Some(datetime_utc) = chrono::DateTime::<Utc>::from_timestamp(seconds, 0) {

            datetime_utc.with_timezone(&offset).to_rfc3339()

        } else {

            Utc::now().to_rfc3339()

        }

    }



    fn should_emit_event(kind: &EventKind) -> bool {

        matches!(

            kind,

            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any

        )

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



    /// 检测Git路径

    /// Author: Evilek, Date: 2025-01-08

    /// 复用系统启动时的Git检测逻辑

    fn detect_git_path() -> Option<String> {

        // 尝试不同的git命令名称（Windows兼容性）

        let git_commands = if cfg!(windows) {

            vec!["git.exe", "git"]

        } else {

            vec!["git"]

        };



        // 首先尝试直接执行git命令

        for git_cmd in &git_commands {

            if let Ok(output) = Self::create_hidden_command(git_cmd)

                .arg("--version")

                .output()

            {

                if output.status.success() {

                    let version = String::from_utf8_lossy(&output.stdout);

                    if !version.trim().is_empty() {

                        debug_log!("[DEBUG] 找到系统Git: {}", git_cmd);

                        return Some(git_cmd.to_string());

                    }

                }

            }

        }



        // 如果直接执行失败，尝试常见的Git安装路径

        let common_paths = if cfg!(windows) {

            vec![

                "C:\\Program Files\\Git\\bin\\git.exe",

                "C:\\Program Files (x86)\\Git\\bin\\git.exe",

                "D:\\Soft\\Git\\bin\\git.exe", // 用户的Git路径

                "C:\\Git\\bin\\git.exe",

            ]

        } else {

            vec![

                "/usr/bin/git",

                "/usr/local/bin/git",

                "/opt/homebrew/bin/git",

            ]

        };



        for path in common_paths {

            if let Ok(output) = Self::create_hidden_command(path).arg("--version").output() {

                if output.status.success() {

                    let version = String::from_utf8_lossy(&output.stdout);

                    if !version.trim().is_empty() {

                        debug_log!("[DEBUG] 找到Git路径: {}", path);

                        return Some(path.to_string());

                    }

                }

            }

        }



        debug_log!("[WARN] 未找到可用的Git路径");

        None

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

            debug_log!("[ERROR] ❌ 内置Git不可用");

        }



        // 3. 不再自动降级到Git2库API，强制要求Git命令

        debug_log!("[DEBUG] 步骤3: 强制返回SystemGit，要求用户安装Git");

        debug_log!("[ERROR] ⚠️  系统Git和内置Git都不可用，GitMentor需要Git命令行工具才能正常工作");

        debug_log!("[ERROR] 请安装Git并确保在PATH中可用，或者在设置中手动配置为Git2Api模式");

        debug_log!("[DEBUG] ========================================");

        GitMethod::SystemGit // 强制返回SystemGit，让错误在实际使用时暴露

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

        match Self::create_hidden_command("git").arg("--version").output() {

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

            match Self::create_hidden_command(git_path)

                .arg("--version")

                .output()

            {

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



    /// 创建隐藏窗口的命令（Windows 下避免黑色 CMD 闪窗）

    /// 作者：Evilek

    /// 编写日期：2025-08-11

    /// 说明：统一封装外部 git 命令创建，Windows 使用 CREATE_NO_WINDOW 隐藏控制台窗口；其他平台保持默认

    /// Confirmed via 寸止(ID:WIN-CMD-HIDE-20250811)

    fn create_hidden_command(program: &str) -> Command {

        #[cfg(windows)]

        {

            use std::os::windows::process::CommandExt;

            // CREATE_NO_WINDOW 常量，避免创建控制台窗口

            const CREATE_NO_WINDOW: u32 = 0x0800_0000;

            let mut cmd = Command::new(program);

            cmd.creation_flags(CREATE_NO_WINDOW);

            return cmd;

        }

        #[cfg(not(windows))]

        {

            Command::new(program)

        }

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



        let output = Self::create_hidden_command(&git_command)

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

            let output = Self::create_hidden_command(&git_command)

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

        let output = Self::create_hidden_command(&git_command)

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

        let output = Self::create_hidden_command(&git_command)

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

        let output = Self::create_hidden_command(&git_command)

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

        self.stop_repo_watcher();

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

        println!("🔍 [get_simple_file_diff] 开始处理文件: {}", file_path);

        let start_time = std::time::Instant::now();



        let repo_path = self

            .repo_path

            .as_ref()

            .ok_or_else(|| anyhow::anyhow!("No repository opened"))?;



        println!("🔍 [get_simple_file_diff] 打开Git仓库: {}", repo_path);

        let repo = Repository::open(repo_path)

            .map_err(|e| anyhow::anyhow!("无法打开Git仓库 {}: {}", repo_path, e))?;

        println!(

            "🔍 [get_simple_file_diff] 仓库打开耗时: {:?}",

            start_time.elapsed()

        );



        // 性能优化：使用git命令行工具，比libgit2更快

        let git_diff_start = std::time::Instant::now();

        let result = self.get_file_diff_via_command(repo_path, file_path);

        println!(

            "🔍 [get_simple_file_diff] Git命令耗时: {:?}",

            git_diff_start.elapsed()

        );



        if result.is_ok() {

            println!(

                "🔍 [get_simple_file_diff] 文件 {} 处理完成，总耗时: {:?}",

                file_path,

                start_time.elapsed()

            );

            return result;

        }



        println!("⚠️ [get_simple_file_diff] Git命令失败，回退到libgit2方法");

        let libgit2_start = std::time::Instant::now();



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



        println!(

            "🔍 [get_simple_file_diff] 文件 {} libgit2处理完成，libgit2耗时: {:?}, 总耗时: {:?}",

            file_path,

            libgit2_start.elapsed(),

            start_time.elapsed()

        );

        Ok(file_diff)

    }



    /// 使用Git命令行工具获取文件diff（性能优化）

    /// Author: Evilek, Date: 2025-01-08

    fn get_file_diff_via_command(&self, repo_path: &str, file_path: &str) -> Result<String> {

        // use std::process::Command;



        // 使用缓存的Git路径，如果没有则回退到检测

        let git_cmd = if let Some(ref git_path) = self.git_path {

            git_path.clone()

        } else {

            // 回退到简单检测

            if cfg!(windows) {

                "git.exe".to_string()

            } else {

                "git".to_string()

            }

        };



        println!(

            "🔍 [get_file_diff_via_command] 使用缓存的Git路径: {} diff HEAD -- {}",

            git_cmd, file_path

        );



        // 首先尝试获取工作目录相对于HEAD的diff

        let output = Self::create_hidden_command(&git_cmd)

            .arg("diff")

            .arg("HEAD")

            .arg("--")

            .arg(file_path)

            .current_dir(repo_path)

            .output()

            .map_err(|e| anyhow::anyhow!("执行git命令失败: {}", e))?;



        println!(

            "🔍 [get_file_diff_via_command] git diff HEAD 状态: {}, stdout长度: {}, stderr: {}",

            output.status.success(),

            output.stdout.len(),

            String::from_utf8_lossy(&output.stderr)

        );



        if output.status.success() {

            let diff_content = String::from_utf8_lossy(&output.stdout);

            if !diff_content.trim().is_empty() {

                println!(

                    "✅ [get_file_diff_via_command] 成功获取diff，长度: {}",

                    diff_content.len()

                );

                return Ok(diff_content.to_string());

            }

        }



        // 如果HEAD diff为空，尝试获取staged diff

        println!("🔍 [get_file_diff_via_command] 尝试staged diff");

        let staged_output = Self::create_hidden_command(&git_cmd)

            .arg("diff")

            .arg("--cached")

            .arg("--")

            .arg(file_path)

            .current_dir(repo_path)

            .output()

            .map_err(|e| anyhow::anyhow!("执行git diff --cached失败: {}", e))?;



        println!(

            "🔍 [get_file_diff_via_command] git diff --cached 状态: {}, stdout长度: {}",

            staged_output.status.success(),

            staged_output.stdout.len()

        );



        if staged_output.status.success() {

            let diff_content = String::from_utf8_lossy(&staged_output.stdout);

            if !diff_content.trim().is_empty() {

                println!(

                    "✅ [get_file_diff_via_command] 成功获取staged diff，长度: {}",

                    diff_content.len()

                );

                return Ok(diff_content.to_string());

            }

        }



        // 最后尝试获取工作目录的变更（不与HEAD比较）

        println!("🔍 [get_file_diff_via_command] 尝试工作目录diff");

        let workdir_output = Self::create_hidden_command(&git_cmd)

            .arg("diff")

            .arg("--")

            .arg(file_path)

            .current_dir(repo_path)

            .output()

            .map_err(|e| anyhow::anyhow!("执行git diff工作目录失败: {}", e))?;



        println!(

            "🔍 [get_file_diff_via_command] git diff 状态: {}, stdout长度: {}",

            workdir_output.status.success(),

            workdir_output.stdout.len()

        );



        if workdir_output.status.success() {

            let diff_content = String::from_utf8_lossy(&workdir_output.stdout);

            if !diff_content.trim().is_empty() {

                println!(

                    "✅ [get_file_diff_via_command] 成功获取工作目录diff，长度: {}",

                    diff_content.len()

                );

                return Ok(diff_content.to_string());

            }

        }



        // 尝试检查文件状态

        println!("🔍 [get_file_diff_via_command] 检查文件状态");

        let status_output = Self::create_hidden_command(&git_cmd)

            .arg("status")

            .arg("--porcelain")

            .arg("--")

            .arg(file_path)

            .current_dir(repo_path)

            .output()

            .map_err(|e| anyhow::anyhow!("执行git status失败: {}", e))?;



        let status_content = String::from_utf8_lossy(&status_output.stdout);

        println!(

            "🔍 [get_file_diff_via_command] 文件状态: '{}'",

            status_content.trim()

        );



        if status_content.trim().is_empty() {

            return Err(anyhow::anyhow!("文件无变更"));

        } else {

            return Err(anyhow::anyhow!(

                "Git命令无法获取diff，但文件有状态变更: {}",

                status_content.trim()

            ));

        }

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



        // 强制优先使用Git命令，只有在完全不可用时才降级

        match self.get_status_with_git_command() {

            Ok(result) => {

                println!("[DEBUG] Git命令方式成功");

                return Ok(result);

            }

            Err(e) => {

                println!("[ERROR] Git命令执行失败: {}", e);

                println!("[ERROR] 请检查Git是否正确安装并在PATH中可用");



                // 只有在明确配置为Git2Api时才使用，否则返回错误

                match self.git_method {

                    GitMethod::Git2Api => {

                        println!("[WARN] 强制使用Git2库API作为最后手段");

                        self.get_status_with_git2_api()

                    }

                    _ => Err(anyhow!("Git命令不可用: {}。请安装Git或检查PATH配置", e)),

                }

            }

        }

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

        // 强制优先使用Git命令

        match self.stage_files_with_command(request) {

            Ok(result) => Ok(result),

            Err(e) => {

                println!("[ERROR] Git stage命令失败: {}", e);

                println!("[ERROR] 文件暂存失败，请检查文件状态");



                // 只有在明确配置为Git2Api时才使用

                match self.git_method {

                    GitMethod::Git2Api => {

                        println!("[WARN] 尝试使用Git2库API进行文件暂存");

                        self.stage_files_with_git2_api(request)

                    }

                    _ => Err(anyhow!("文件暂存失败: {}。请使用Git命令行工具", e)),

                }

            }

        }

    }



    /// 使用Git命令暂存文件

    fn stage_files_with_command(&self, request: &StageRequest) -> Result<GitOperationResult> {

        let repo_path = self

            .get_repository_path()

            .ok_or_else(|| anyhow!("仓库路径未设置"))?;

        let git_command = self.get_git_command();



        let mut success_count = 0;

        let mut failed_files = Vec::new();



        for file_path in &request.file_paths {

            let mut args = vec!["add"];



            if request.stage {

                // 暂存文件

                args.push(file_path);

            } else {

                // 取消暂存文件

                args = vec!["reset", "HEAD", file_path];

            }



            let output = Self::create_hidden_command(&git_command)

                .current_dir(&repo_path)

                .args(&args)

                .output()?;



            if output.status.success() {

                success_count += 1;

            } else {

                let error_msg = String::from_utf8_lossy(&output.stderr);

                failed_files.push(format!("{}: {}", file_path, error_msg));

            }

        }



        if failed_files.is_empty() {

            Ok(GitOperationResult {

                success: true,

                message: if request.stage {

                    format!("成功暂存 {} 个文件", success_count)

                } else {

                    format!("成功取消暂存 {} 个文件", success_count)

                },

                details: None,

            })

        } else {

            Err(anyhow!("部分文件操作失败: {}", failed_files.join(", ")))

        }

    }



    /// 使用Git2库API暂存文件（备选方案）

    fn stage_files_with_git2_api(&self, request: &StageRequest) -> Result<GitOperationResult> {

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

        // 强制优先使用Git命令

        match self.commit_with_command(request) {

            Ok(result) => Ok(result),

            Err(e) => {

                println!("[ERROR] Git commit命令失败: {}", e);

                println!("[ERROR] 提交失败，请检查暂存区状态和提交信息");



                // 只有在明确配置为Git2Api时才使用

                match self.git_method {

                    GitMethod::Git2Api => {

                        println!("[WARN] 尝试使用Git2库API进行提交");

                        self.commit_with_git2_api(request)

                    }

                    _ => Err(anyhow!("提交失败: {}。请使用Git命令行工具", e)),

                }

            }

        }

    }



    /// 使用Git命令提交

    fn commit_with_command(&self, request: &CommitRequest) -> Result<GitOperationResult> {

        let repo_path = self

            .get_repository_path()

            .ok_or_else(|| anyhow!("仓库路径未设置"))?;

        let git_command = self.get_git_command();



        // 如果指定了文件，先暂存这些文件

        if !request.selected_files.is_empty() {

            for file_path in &request.selected_files {

                let output = Self::create_hidden_command(&git_command)

                    .current_dir(&repo_path)

                    .args(&["add", file_path])

                    .output()?;



                if !output.status.success() {

                    let error_msg = String::from_utf8_lossy(&output.stderr);

                    return Err(anyhow!("暂存文件 {} 失败: {}", file_path, error_msg));

                }

            }

        }



        // 构建提交命令

        let mut args = vec!["commit", "-m", &request.message];



        if request.amend {

            args.insert(1, "--amend");

        }



        let output = Self::create_hidden_command(&git_command)

            .current_dir(&repo_path)

            .args(&args)

            .output()?;



        if output.status.success() {

            let stdout = String::from_utf8_lossy(&output.stdout);

            Ok(GitOperationResult {

                success: true,

                message: "提交成功".to_string(),

                details: Some(stdout.to_string()),

            })

        } else {

            let error_msg = String::from_utf8_lossy(&output.stderr);

            Err(anyhow!("提交失败: {}", error_msg))

        }

    }



    /// 使用Git2库API提交（备选方案）

    fn commit_with_git2_api(&self, request: &CommitRequest) -> Result<GitOperationResult> {

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

    fn get_signature(&self, repo: &Repository) -> Result<Signature<'_>> {

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

                // 回滚暂存区更改 - 将暂存区文件重置到HEAD状态，但保留工作区更改

                let head = repo.head()?;

                let head_commit = head.peel_to_commit()?;



                // 将指定文件从暂存区重置到HEAD状态（取消暂存）

                repo.reset_default(Some(head_commit.as_object()), request.file_paths.iter())?;



                Ok(GitOperationResult {

                    success: true,

                    message: format!(

                        "Unstaged {} file(s) from staging area",

                        request.file_paths.len()

                    ),

                    details: None,

                })

            }

            RevertType::DiscardAll => {

                // 撤销所有更改 - 先取消暂存，再回滚工作区到HEAD状态

                let head = repo.head()?;

                let head_commit = head.peel_to_commit()?;

                let head_tree = head_commit.tree()?;



                // 1. 先取消暂存（重置索引到HEAD）

                repo.reset_default(Some(head_commit.as_object()), request.file_paths.iter())?;



                // 2. 再回滚工作区到HEAD状态

                let mut checkout_builder = git2::build::CheckoutBuilder::new();

                checkout_builder.force();



                for file_path in &request.file_paths {

                    checkout_builder.path(file_path);

                }



                repo.checkout_tree(head_tree.as_object(), Some(&mut checkout_builder))?;



                Ok(GitOperationResult {

                    success: true,

                    message: format!(

                        "Discarded all changes for {} file(s)",

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







        let current_branch = repo.head()?.shorthand().unwrap_or_default().to_string();







        for branch in branches {



            let (branch, branch_type) = branch?;



            if let Some(name) = branch.name()? {



                let mut upstream_name = None;



                let mut ahead = 0;



                let mut behind = 0;







                if branch_type == BranchType::Local {



                    if let Ok(upstream_branch) = branch.upstream() {



                        if let Some(up_name) = upstream_branch.name()? {



                            upstream_name = Some(up_name.to_string());



                        }







                        if let (Some(local_target), Some(remote_target)) =



                            (branch.get().target(), upstream_branch.get().target())



                        {



                            if let Ok((ahead_count, behind_count)) =



                                repo.graph_ahead_behind(local_target, remote_target)



                            {



                                ahead = ahead_count as u32;



                                behind = behind_count as u32;



                            }



                        }



                    }



                } else if branch_type == BranchType::Remote {



                    if let Some(local_name) = name.splitn(2, '/').nth(1) {



                        if let Ok(local_branch) = repo.find_branch(local_name, BranchType::Local) {



                            upstream_name = Some(name.to_string());







                            if let (Some(remote_target), Some(local_target)) =



                                (branch.get().target(), local_branch.get().target())



                            {



                                if let Ok((ahead_count, behind_count)) =



                                    repo.graph_ahead_behind(local_target, remote_target)



                                {



                                    // Remote divergence is inverted to express local vs remote.



                                    ahead = behind_count as u32;



                                    behind = ahead_count as u32;



                                }



                            }



                        }



                    }



                }







                let branch_info = BranchInfo {



                    name: name.to_string(),



                    is_current: name == current_branch,



                    is_remote: branch_type == BranchType::Remote,



                    upstream: upstream_name,



                    ahead,



                    behind,



                };



                branch_list.push(branch_info);



            }



        }







        Ok(branch_list)



    }







    /// 切换分支

    /// 作者：Evilek

    /// 编写日期：2025-08-12

    pub fn checkout_branch(

        &self,

        branch_name: &str,

        is_remote: bool,

    ) -> Result<GitOperationResult> {

        // 强制优先使用Git命令

        match self.checkout_branch_with_command(branch_name, is_remote) {

            Ok(result) => Ok(result),

            Err(e) => {

                println!("[ERROR] Git checkout命令失败: {}", e);

                println!("[ERROR] 分支切换失败，请检查分支名称和Git状态");



                // 只有在明确配置为Git2Api时才使用

                match self.git_method {

                    GitMethod::Git2Api => {

                        println!("[WARN] 尝试使用Git2库API进行分支切换");

                        self.checkout_branch_with_git2_api(branch_name, is_remote)

                    }

                    _ => Err(anyhow!("分支切换失败: {}。请使用Git命令行工具", e)),

                }

            }

        }

    }



    /// 使用Git命令切换分支

    fn checkout_branch_with_command(

        &self,

        branch_name: &str,

        is_remote: bool,

    ) -> Result<GitOperationResult> {

        let repo_path = self

            .get_repository_path()

            .ok_or_else(|| anyhow!("仓库路径未设置"))?;

        let git_command = self.get_git_command();



        if is_remote {

            // 检出远程分支，创建本地跟踪分支

            let local_branch_name = if branch_name.starts_with("origin/") {

                branch_name.strip_prefix("origin/").unwrap_or(branch_name)

            } else {

                branch_name

            };



            let output = Self::create_hidden_command(&git_command)

                .current_dir(&repo_path)

                .args(&["checkout", "-b", local_branch_name, branch_name])

                .output()?;



            if output.status.success() {

                Ok(GitOperationResult {

                    success: true,

                    message: format!(

                        "成功检出远程分支 {} 并创建本地分支 {}",

                        branch_name, local_branch_name

                    ),

                    details: Some(String::from_utf8_lossy(&output.stdout).to_string()),

                })

            } else {

                let error_msg = String::from_utf8_lossy(&output.stderr);

                Err(anyhow!("检出远程分支失败: {}", error_msg))

            }

        } else {

            // 切换本地分支

            let output = Self::create_hidden_command(&git_command)

                .current_dir(&repo_path)

                .args(&["checkout", branch_name])

                .output()?;



            if output.status.success() {

                Ok(GitOperationResult {

                    success: true,

                    message: format!("成功切换到分支 {}", branch_name),

                    details: Some(String::from_utf8_lossy(&output.stdout).to_string()),

                })

            } else {

                let error_msg = String::from_utf8_lossy(&output.stderr);

                Err(anyhow!("切换分支失败: {}", error_msg))

            }

        }

    }



    /// 使用Git2库API切换分支

    fn checkout_branch_with_git2_api(

        &self,

        branch_name: &str,

        is_remote: bool,

    ) -> Result<GitOperationResult> {

        let repo = self.get_repository()?;



        if is_remote {

            // 检出远程分支，创建本地跟踪分支

            let local_branch_name = if branch_name.starts_with("origin/") {

                branch_name.strip_prefix("origin/").unwrap_or(branch_name)

            } else {

                branch_name

            };



            // 查找远程分支

            let remote_branch = repo.find_branch(branch_name, git2::BranchType::Remote)?;

            let remote_commit = remote_branch.get().peel_to_commit()?;



            // 创建本地分支

            let mut local_branch = repo.branch(local_branch_name, &remote_commit, false)?;



            // 设置上游分支

            local_branch.set_upstream(Some(branch_name))?;



            // 检出新创建的本地分支

            let obj = repo.revparse_single(&("refs/heads/".to_owned() + local_branch_name))?;

            repo.checkout_tree(&obj, None)?;

            repo.set_head(&("refs/heads/".to_owned() + local_branch_name))?;



            Ok(GitOperationResult {

                success: true,

                message: format!(

                    "成功检出远程分支 {} 并创建本地分支 {}",

                    branch_name, local_branch_name

                ),

                details: None,

            })

        } else {

            // 切换本地分支

            let obj = repo.revparse_single(&("refs/heads/".to_owned() + branch_name))?;

            repo.checkout_tree(&obj, None)?;

            repo.set_head(&("refs/heads/".to_owned() + branch_name))?;



            Ok(GitOperationResult {

                success: true,

                message: format!("成功切换到分支 {}", branch_name),

                details: None,

            })

        }

    }



    /// 拉取当前分支

    /// 作者：Evilek

    /// 编写日期：2025-08-12

    pub fn pull_current_branch(&self) -> Result<GitOperationResult> {

        // 强制优先使用Git命令

        match self.pull_with_command() {

            Ok(result) => Ok(result),

            Err(e) => {

                println!("[ERROR] Git pull命令失败: {}", e);

                println!("[ERROR] 拉取失败，可能存在合并冲突或网络问题");



                // 只有在明确配置为Git2Api时才使用

                match self.git_method {

                    GitMethod::Git2Api => {

                        println!("[WARN] 尝试使用Git2库API进行拉取（功能有限）");

                        self.pull_with_git2_api()

                    }

                    _ => Err(anyhow!("拉取失败: {}。请使用Git命令行解决冲突", e)),

                }

            }

        }

    }



    /// 使用Git命令拉取

    fn pull_with_command(&self) -> Result<GitOperationResult> {

        let repo_path = self

            .get_repository_path()

            .ok_or_else(|| anyhow!("仓库路径未设置"))?;

        let git_command = self.get_git_command();



        let output = Self::create_hidden_command(&git_command)

            .current_dir(&repo_path)

            .args(&["pull"])

            .output()?;



        if output.status.success() {

            let stdout = String::from_utf8_lossy(&output.stdout);

            Ok(GitOperationResult {

                success: true,

                message: "成功拉取远程更改".to_string(),

                details: Some(stdout.to_string()),

            })

        } else {

            let error_msg = String::from_utf8_lossy(&output.stderr);

            Err(anyhow!("拉取失败: {}", error_msg))

        }

    }



    /// 使用Git2库API拉取

    fn pull_with_git2_api(&self) -> Result<GitOperationResult> {

        let repo = self.get_repository()?;



        // 第一步：Fetch

        let mut remote = repo.find_remote("origin").or_else(|_| {

            let remotes = repo.remotes()?;

            if let Some(remote_name) = remotes.get(0) {

                repo.find_remote(remote_name)

            } else {

                Err(git2::Error::from_str("没有找到远程仓库"))

            }

        })?;



        // 设置fetch回调

        let mut callbacks = git2::RemoteCallbacks::new();

        callbacks.credentials(|_url, username_from_url, _allowed_types| {

            if let Some(username) = username_from_url {

                git2::Cred::ssh_key_from_agent(username)

            } else {

                git2::Cred::default()

            }

        });



        let mut fetch_options = git2::FetchOptions::new();

        fetch_options.remote_callbacks(callbacks);



        // 执行fetch

        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;



        // 第二步：Merge

        let head = repo.head()?;

        let branch_name = head.shorthand().unwrap_or("HEAD");

        let upstream_branch = format!("refs/remotes/origin/{}", branch_name);



        // 检查是否有upstream分支

        let upstream_ref = repo.find_reference(&upstream_branch)?;

        let upstream_commit = repo.reference_to_annotated_commit(&upstream_ref)?;



        // 执行merge分析

        let analysis = repo.merge_analysis(&[&upstream_commit])?;



        if analysis.0.is_up_to_date() {

            Ok(GitOperationResult {

                success: true,

                message: "当前分支已是最新".to_string(),

                details: Some("无需拉取".to_string()),

            })

        } else if analysis.0.is_fast_forward() {

            // 快进合并

            let mut reference = repo.find_reference(&format!("refs/heads/{}", branch_name))?;

            reference.set_target(upstream_commit.id(), "Fast-forward merge")?;

            repo.set_head(&format!("refs/heads/{}", branch_name))?;

            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;



            Ok(GitOperationResult {

                success: true,

                message: format!("成功快进合并分支 {}", branch_name),

                details: Some("执行了快进合并".to_string()),

            })

        } else {

            // 需要普通合并，这比较复杂，建议使用Git命令

            Err(anyhow!("需要合并提交，建议使用Git命令执行pull操作"))

        }

    }



    /// 推送当前分支

    /// 作者：Evilek

    /// 编写日期：2025-08-12

    pub fn push_current_branch(&self, force: bool) -> Result<GitOperationResult> {

        // 强制优先使用Git命令

        match self.push_with_command(force) {

            Ok(result) => Ok(result),

            Err(e) => {

                println!("[ERROR] Git push命令失败: {}", e);

                println!("[ERROR] 推送失败，可能需要先拉取或存在权限问题");



                // 只有在明确配置为Git2Api时才使用

                match self.git_method {

                    GitMethod::Git2Api => {

                        println!("[WARN] 尝试使用Git2库API进行推送（需要正确的认证配置）");

                        self.push_with_git2_api(force)

                    }

                    _ => Err(anyhow!("推送失败: {}。请检查网络连接和权限配置", e)),

                }

            }

        }

    }



    /// 使用Git命令推送

    fn push_with_command(&self, force: bool) -> Result<GitOperationResult> {

        let repo_path = self

            .get_repository_path()

            .ok_or_else(|| anyhow!("仓库路径未设置"))?;

        let git_command = self.get_git_command();



        let mut args = vec!["push"];

        if force {

            args.push("--force");

        }



        let output = Self::create_hidden_command(&git_command)

            .current_dir(&repo_path)

            .args(&args)

            .output()?;



        if output.status.success() {

            let stdout = String::from_utf8_lossy(&output.stdout);

            let stderr = String::from_utf8_lossy(&output.stderr);

            let details = if !stderr.is_empty() {

                stderr.to_string()

            } else {

                stdout.to_string()

            };



            Ok(GitOperationResult {

                success: true,

                message: if force {

                    "成功强制推送到远程仓库".to_string()

                } else {

                    "成功推送到远程仓库".to_string()

                },

                details: Some(details),

            })

        } else {

            let error_msg = String::from_utf8_lossy(&output.stderr);

            Err(anyhow!("推送失败: {}", error_msg))

        }

    }



    /// 使用Git2库API推送

    fn push_with_git2_api(&self, force: bool) -> Result<GitOperationResult> {

        let repo = self.get_repository()?;



        // 获取当前分支

        let head = repo.head()?;

        let branch_name = head.shorthand().unwrap_or("HEAD");



        // 获取远程仓库

        let mut remote = repo.find_remote("origin").or_else(|_| {

            // 如果没有origin，尝试获取第一个远程仓库

            let remotes = repo.remotes()?;

            if let Some(remote_name) = remotes.get(0) {

                repo.find_remote(remote_name)

            } else {

                Err(git2::Error::from_str("没有找到远程仓库"))

            }

        })?;



        // 构建推送引用规范

        let refspec = if force {

            format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name)

        } else {

            format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)

        };



        // 设置推送选项和回调

        let mut push_options = git2::PushOptions::new();



        // 设置认证回调

        let mut callbacks = git2::RemoteCallbacks::new();

        callbacks.credentials(|_url, username_from_url, _allowed_types| {

            // 尝试使用SSH密钥

            if let Some(username) = username_from_url {

                git2::Cred::ssh_key_from_agent(username)

            } else {

                // 尝试使用默认凭据

                git2::Cred::default()

            }

        });



        // 设置推送进度回调

        callbacks.push_update_reference(|refname, status| {

            if let Some(msg) = status {

                println!("推送更新失败 {}: {}", refname, msg);

                Err(git2::Error::from_str("推送更新失败"))

            } else {

                println!("推送更新成功: {}", refname);

                Ok(())

            }

        });



        push_options.remote_callbacks(callbacks);



        // 执行推送

        match remote.push(&[&refspec], Some(&mut push_options)) {

            Ok(_) => Ok(GitOperationResult {

                success: true,

                message: if force {

                    format!("成功强制推送分支 {} 到远程仓库", branch_name)

                } else {

                    format!("成功推送分支 {} 到远程仓库", branch_name)

                },

                details: Some(format!("推送引用: {}", refspec)),

            }),

            Err(e) => {

                // 如果Git2推送失败，提供更详细的错误信息

                let error_msg = format!("Git2推送失败: {}。建议使用系统Git命令进行推送", e);

                Err(anyhow!(error_msg))

            }

        }

    }



    /// 获取远程更新（fetch）

    /// 作者：Evilek

    /// 编写日期：2025-08-12

    pub fn fetch_remote(&self, remote_name: Option<&str>) -> Result<GitOperationResult> {

        // 强制优先使用Git命令

        match self.fetch_with_command(remote_name) {

            Ok(result) => Ok(result),

            Err(e) => {

                println!("[ERROR] Git fetch命令失败: {}", e);

                println!("[ERROR] 获取远程更新失败，请检查网络连接");



                // 只有在明确配置为Git2Api时才使用

                match self.git_method {

                    GitMethod::Git2Api => {

                        println!("[WARN] 尝试使用Git2库API进行fetch");

                        self.fetch_with_git2_api(remote_name)

                    }

                    _ => Err(anyhow!("获取远程更新失败: {}。请检查网络连接", e)),

                }

            }

        }

    }



    /// 使用Git命令获取远程更新

    fn fetch_with_command(&self, remote_name: Option<&str>) -> Result<GitOperationResult> {

        let repo_path = self

            .get_repository_path()

            .ok_or_else(|| anyhow!("仓库路径未设置"))?;

        let git_command = self.get_git_command();



        let mut args = vec!["fetch"];

        if let Some(remote) = remote_name {

            args.push(remote);

        }



        let output = Self::create_hidden_command(&git_command)

            .current_dir(&repo_path)

            .args(&args)

            .output()?;



        if output.status.success() {

            let stderr = String::from_utf8_lossy(&output.stderr);

            Ok(GitOperationResult {

                success: true,

                message: "成功获取远程更新".to_string(),

                details: Some(stderr.to_string()),

            })

        } else {

            let error_msg = String::from_utf8_lossy(&output.stderr);

            Err(anyhow!("获取远程更新失败: {}", error_msg))

        }

    }



    /// 使用Git2库API获取远程更新

    fn fetch_with_git2_api(&self, remote_name: Option<&str>) -> Result<GitOperationResult> {

        let repo = self.get_repository()?;

        let remote_name = remote_name.unwrap_or("origin");



        let mut remote = repo.find_remote(remote_name)?;

        remote.fetch(&[] as &[&str], None, None)?;



        Ok(GitOperationResult {

            success: true,

            message: format!("成功从 {} 获取远程更新", remote_name),

            details: None,

        })

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



    /// 添加文件到 .gitignore

    /// 作者：Evilek

    /// 编写日期：2025-08-11

    pub fn add_to_gitignore(&self, file_paths: &[String]) -> Result<GitOperationResult> {

        let repo_path = self

            .repo_path

            .as_ref()

            .ok_or_else(|| anyhow!("No repository opened"))?;



        let gitignore_path = std::path::Path::new(repo_path).join(".gitignore");



        // 读取现有的 .gitignore 内容

        let mut existing_content = if gitignore_path.exists() {

            std::fs::read_to_string(&gitignore_path)?

        } else {

            String::new()

        };



        // 确保内容以换行符结尾

        if !existing_content.is_empty() && !existing_content.ends_with('\n') {

            existing_content.push('\n');

        }



        let mut added_count = 0;

        let mut already_ignored = Vec::new();



        for file_path in file_paths {

            // 检查文件是否已经在 .gitignore 中

            if existing_content.lines().any(|line| {

                let trimmed = line.trim();

                !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed == file_path

            }) {

                already_ignored.push(file_path.clone());

                continue;

            }



            // 添加到 .gitignore

            existing_content.push_str(file_path);

            existing_content.push('\n');

            added_count += 1;

        }



        // 写入 .gitignore 文件

        if added_count > 0 {

            std::fs::write(&gitignore_path, existing_content)?;

        }



        let message = if added_count > 0 {

            format!("Added {} file(s) to .gitignore", added_count)

        } else {

            "No new files added to .gitignore".to_string()

        };



        Ok(GitOperationResult {

            success: true,

            message,

            details: if !already_ignored.is_empty() {

                Some(format!("Already ignored: {}", already_ignored.join(", ")))

            } else {

                None

            },

        })

    }



    /// 删除未跟踪文件

    /// 作者：Evilek

    /// 编写日期：2025-08-11

    pub fn delete_untracked_files(&self, file_paths: &[String]) -> Result<GitOperationResult> {

        let repo_path = self

            .repo_path

            .as_ref()

            .ok_or_else(|| anyhow!("No repository opened"))?;



        let mut deleted_count = 0;

        let mut failed_files = Vec::new();



        for file_path in file_paths {

            let full_path = std::path::Path::new(repo_path).join(file_path);



            match std::fs::remove_file(&full_path) {

                Ok(_) => deleted_count += 1,

                Err(e) => {

                    // 如果是目录，尝试删除目录

                    if full_path.is_dir() {

                        match std::fs::remove_dir_all(&full_path) {

                            Ok(_) => deleted_count += 1,

                            Err(_) => failed_files.push(format!("{} ({})", file_path, e)),

                        }

                    } else {

                        failed_files.push(format!("{} ({})", file_path, e));

                    }

                }

            }

        }



        let mut message = format!("Deleted {} untracked file(s)", deleted_count);

        if !failed_files.is_empty() {

            message.push_str(&format!(

                ", failed to delete {} file(s)",

                failed_files.len()

            ));

        }



        Ok(GitOperationResult {

            success: deleted_count > 0,

            message,

            details: if failed_files.is_empty() {

                None

            } else {

                Some(format!("Failed to delete:\n{}", failed_files.join("\n")))

            },

        })

    }



    /// 删除已跟踪文件（从Git和文件系统中移除）

    /// 作者：Evilek

    /// 编写日期：2025-08-11

    pub fn delete_tracked_files(&self, file_paths: &[String]) -> Result<GitOperationResult> {

        let repo = self.get_repository()?;

        let repo_path = self

            .repo_path

            .as_ref()

            .ok_or_else(|| anyhow!("No repository opened"))?;



        let mut deleted_count = 0;

        let mut failed_files = Vec::new();

        let mut index = repo.index()?;



        for file_path in file_paths {

            let full_path = std::path::Path::new(repo_path).join(file_path);



            // 1. 从Git索引中移除文件

            match index.remove_path(std::path::Path::new(file_path)) {

                Ok(_) => {

                    // 2. 删除物理文件

                    match std::fs::remove_file(&full_path) {

                        Ok(_) => deleted_count += 1,

                        Err(e) => {

                            // 如果是目录，尝试删除目录

                            if full_path.is_dir() {

                                match std::fs::remove_dir_all(&full_path) {

                                    Ok(_) => deleted_count += 1,

                                    Err(_) => failed_files.push(format!("{} ({})", file_path, e)),

                                }

                            } else {

                                failed_files.push(format!("{} ({})", file_path, e));

                            }

                        }

                    }

                }

                Err(e) => {

                    failed_files.push(format!("{} (Git remove failed: {})", file_path, e));

                }

            }

        }



        // 写入索引更改

        if deleted_count > 0 {

            index.write()?;

        }



        let mut message = format!("Deleted {} tracked file(s)", deleted_count);

        if !failed_files.is_empty() {

            message.push_str(&format!(

                ", failed to delete {} file(s)",

                failed_files.len()

            ));

        }



        Ok(GitOperationResult {

            success: deleted_count > 0,

            message,

            details: if failed_files.is_empty() {

                None

            } else {

                Some(format!("Failed to delete:\n{}", failed_files.join("\n")))

            },

        })

    }



    // 日报生成相关方法 - Author: Evilek, Date: 2025-08-21



    /// 获取可用仓库列表

    pub fn get_available_repositories(

        &self,

        repo_paths: Vec<String>,

    ) -> Result<Vec<crate::types::git_types::Repository>> {

        let mut repositories = Vec::new();



        let git_command = self.get_git_command();



        for path in repo_paths {

            // 检查是否为Git仓库

            if let Ok(output) = Self::create_hidden_command(&git_command)

                .current_dir(&path)

                .args(&["rev-parse", "--git-dir"])

                .output()

            {

                if output.status.success() {

                    let name = Path::new(&path)

                        .file_name()

                        .and_then(|n| n.to_str())

                        .unwrap_or("Unknown")

                        .to_string();



                    // 检查是否为bare仓库

                    let is_bare = Self::create_hidden_command(&git_command)

                        .current_dir(&path)

                        .args(&["rev-parse", "--is-bare-repository"])

                        .output()

                        .map(|out| String::from_utf8_lossy(&out.stdout).trim() == "true")

                        .unwrap_or(false);



                    let status = if is_bare {

                        "Bare Repository".to_string()

                    } else {

                        "Ready".to_string()

                    };



                    repositories.push(crate::types::git_types::Repository { name, path, status });

                }

            }

        }



        Ok(repositories)

    }



    /// 获取仓库贡献者列表

    pub fn get_repo_contributors(

        &self,

        repo_paths: Vec<String>,

    ) -> Result<Vec<crate::types::git_types::Contributor>> {

        let mut contributors = std::collections::HashMap::new();



        let git_command = self.get_git_command();



        for repo_path in repo_paths {

            // 使用git log命令获取提交者信息

            if let Ok(output) = Self::create_hidden_command(&git_command)

                .current_dir(&repo_path)

                .args(&["log", "--format=%an|%ae", "--all"])

                .output()

            {

                if output.status.success() {

                    let log_output = String::from_utf8_lossy(&output.stdout);

                    for line in log_output.lines() {

                        if let Some((name, email)) = line.split_once('|') {

                            let name = name.trim().to_string();

                            let email = email.trim().to_string();



                            if !email.is_empty() && !name.is_empty() {

                                let entry = contributors.entry(email.clone()).or_insert(

                                    crate::types::git_types::Contributor {

                                        name,

                                        email,

                                        commit_count: 0,

                                    },

                                );

                                entry.commit_count += 1;

                            }

                        }

                    }

                }

            }

        }



        Ok(contributors.into_values().collect())

    }



    /// 分析提交记录

    pub fn analyze_commits(

        &self,

        config: crate::types::git_types::AnalysisConfig,

    ) -> Result<crate::types::git_types::CommitAnalysis> {

        // 日期处理相关导入已移除，使用Git命令行的日期过滤



        let mut commits_by_user = std::collections::HashMap::new();

        let mut commits_by_repo = std::collections::HashMap::new();

        let mut file_changes = std::collections::HashMap::new();

        let mut total_commits = 0;

        let git_command = self.get_git_command();



        for repo_path in &config.repoPaths {

            let repo_name = Path::new(repo_path)

                .file_name()

                .and_then(|n| n.to_str())

                .unwrap_or("Unknown")

                .to_string();



            let mut repo_commits = Vec::new();



            // 构建git log命令参数

            let mut args = vec!["log", "--format=%H|%h|%an|%ae|%at|%s", "--all"];



            // 添加日期过滤

            if !config.startDate.is_empty() && !config.endDate.is_empty() {

                args.push("--since");

                args.push(&config.startDate);

                args.push("--until");

                args.push(&config.endDate);

            }



            // 执行git log命令

            if let Ok(output) = Self::create_hidden_command(&git_command)

                .current_dir(repo_path)

                .args(&args)

                .output()

            {

                if output.status.success() {

                    let log_output = String::from_utf8_lossy(&output.stdout);

                    for line in log_output.lines() {

                        if let Some(parts) = Self::parse_commit_line(line) {

                            let (hash, short_hash, author, email, timestamp_str, message) = parts;



                            // 检查用户过滤

                            if config.userEmails.is_empty() || config.userEmails.contains(&email) {

                                let timestamp = timestamp_str.parse::<i64>().unwrap_or(0);



                                // 获取提交涉及的文件

                                let files_changed =

                                    self.get_commit_files_with_command(repo_path, &hash)?;



                                let commit_info = CommitInfo {

                                    hash,

                                    short_hash,

                                    message,

                                    author,

                                    email: email.clone(),

                                    timestamp,

                                    files_changed: files_changed.clone(),

                                };



                                // 统计文件变更

                                for file in &files_changed {

                                    *file_changes.entry(file.clone()).or_insert(0) += 1;

                                }



                                // 按用户分组

                                commits_by_user

                                    .entry(email)

                                    .or_insert_with(Vec::new)

                                    .push(commit_info.clone());



                                repo_commits.push(commit_info);

                                total_commits += 1;

                            }

                        }

                    }

                }

            }



            commits_by_repo.insert(repo_name, repo_commits);

        }



        Ok(crate::types::git_types::CommitAnalysis {

            total_commits,

            commits_by_user,

            commits_by_repo,

            file_changes,

            analysis_period: format!("{} to {}", config.startDate, config.endDate),

        })

    }



    /// 解析git log输出行

    fn parse_commit_line(line: &str) -> Option<(String, String, String, String, String, String)> {

        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() >= 6 {

            Some((

                parts[0].to_string(),             // hash

                parts[1].to_string(),             // short_hash

                parts[2].to_string(),             // author

                parts[3].to_string(),             // email

                parts[4].to_string(),             // timestamp

                parts[5..].join("|").to_string(), // message (可能包含|字符)

            ))

        } else {

            None

        }

    }



    /// 使用Git命令获取提交涉及的文件列表

    fn get_commit_files_with_command(

        &self,

        repo_path: &str,

        commit_hash: &str,

    ) -> Result<Vec<String>> {

        let git_command = self.get_git_command();

        let mut files = Vec::new();



        // 使用git show命令获取提交涉及的文件

        if let Ok(output) = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(&["show", "--name-only", "--format=", commit_hash])

            .output()

        {

            if output.status.success() {

                let files_output = String::from_utf8_lossy(&output.stdout);

                for line in files_output.lines() {

                    let line = line.trim();

                    if !line.is_empty() {

                        files.push(line.to_string());

                    }

                }

            }

        }



        Ok(files)

    }



    /// 获取指定仓库的提交信息

    pub fn get_commit_info(&self, repo_path: &str, commit_id: &str) -> Result<CommitInfo> {

        let git_command = self.get_git_command();



        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(&[

                "show",

                "--format=%H|%h|%an|%ae|%ct|%s",

                "--no-patch",

                commit_id,

            ])

            .output()

            .map_err(|e| anyhow!("Failed to get commit info: {}", e))?;



        if !output.status.success() {

            return Err(anyhow!(

                "Git command failed: {}",

                String::from_utf8_lossy(&output.stderr)

            ));

        }



        let output = String::from_utf8_lossy(&output.stdout);

        let line = output

            .lines()

            .next()

            .ok_or_else(|| anyhow!("No commit info found"))?;



        if let Some((hash, short_hash, author, email, timestamp, message)) =

            Self::parse_commit_line(line)

        {

            let timestamp = timestamp.parse().unwrap_or(0);

            let files_changed = self.get_commit_files_with_command(repo_path, commit_id)?;



            Ok(CommitInfo {

                hash,

                short_hash,

                message,

                author,

                email,

                timestamp,

                files_changed,

            })

        } else {

            Err(anyhow!("Failed to parse commit info"))

        }

    }



    /// 获取指定仓库的提交差异

    pub fn get_commit_diff(&self, repo_path: &str, commit_id: &str) -> Result<FileDiffResult> {

        let git_command = self.get_git_command();



        // 获取文件变更列表

        let files_output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(&["show", "--name-only", "--format=", commit_id])

            .output()

            .map_err(|e| anyhow!("Failed to get changed files: {}", e))?;



        let mut files_changed = Vec::new();

        if files_output.status.success() {

            let output = String::from_utf8_lossy(&files_output.stdout);

            for line in output.lines() {

                let line = line.trim();

                if !line.is_empty() {

                    files_changed.push(line.to_string());

                }

            }

        }



        // 简化处理：返回基本的差异信息

        Ok(FileDiffResult {

            file_path: files_changed.first().cloned().unwrap_or_default(),

            old_content: None,

            new_content: None,

            old_file_name: None,

            new_file_name: None,

            file_language: None,

            hunks: Vec::new(),

            is_binary: false,

            is_new_file: false,

            is_deleted_file: false,

        })

    }



    /// 获取日期范围内的提交列表

    pub fn get_commits_in_date_range(

        &self,

        repo_path: &str,

        start_date: &str,

        end_date: &str,

    ) -> Result<Vec<CommitInfo>> {

        let git_command = self.get_git_command();



        println!(

            "执行 git log 命令获取 {} 至 {} 的提交",

            start_date, end_date

        );



        // 尝试使用更宽松的日期格式，添加时间部分

        let start_with_time = format!("{} 00:00:00", start_date);

        let end_with_time = format!("{} 23:59:59", end_date);



        println!(

            "使用带时间的日期格式: {} 至 {}",

            start_with_time, end_with_time

        );



        let output = Self::create_hidden_command(&git_command)

            .current_dir(repo_path)

            .args(&[

                "log",

                &format!("--since={}", start_with_time),

                &format!("--until={}", end_with_time),

                "--format=%H|%h|%an|%ae|%ct|%s",

                "--date=iso",

            ])

            .output()

            .map_err(|e| anyhow!("Failed to get commits in date range: {}", e))?;



        if !output.status.success() {

            let stderr = String::from_utf8_lossy(&output.stderr);

            println!("Git log 命令失败: {}", stderr);

            return Err(anyhow!("Git command failed: {}", stderr));

        }



        let mut commits = Vec::new();

        let output = String::from_utf8_lossy(&output.stdout);



        println!("Git log 输出行数: {}", output.lines().count());



        // 如果没有找到提交，输出一些调试信息

        if output.lines().count() == 0 {

            println!("调试信息：尝试获取最近的提交...");

            let debug_output = Self::create_hidden_command(&git_command)

                .current_dir(repo_path)

                .args(&["log", "--oneline", "-5"])

                .output()

                .map_err(|e| anyhow!("Failed to get recent commits: {}", e))?;



            if debug_output.status.success() {

                let recent_commits = String::from_utf8_lossy(&debug_output.stdout);

                println!("最近的5个提交：");

                for line in recent_commits.lines().take(5) {

                    println!("  {}", line);

                }

            }



            // 尝试不使用日期过滤获取总提交数

            let total_output = Self::create_hidden_command(&git_command)

                .current_dir(repo_path)

                .args(&["rev-list", "--count", "HEAD"])

                .output()

                .map_err(|e| anyhow!("Failed to get total commits: {}", e))?;



            if total_output.status.success() {

                let total_output_str = String::from_utf8_lossy(&total_output.stdout);

                let total_count = total_output_str.trim();

                println!("仓库总提交数: {}", total_count);

            }

        }



        for line in output.lines() {

            if let Some((hash, short_hash, author, email, timestamp, message)) =

                Self::parse_commit_line(line)

            {

                let timestamp = timestamp.parse().unwrap_or(0);

                let files_changed = self.get_commit_files_with_command(repo_path, &hash)?;



                commits.push(CommitInfo {

                    hash,

                    short_hash,

                    message,

                    author,

                    email,

                    timestamp,

                    files_changed,

                });

            }

        }



        Ok(commits)

    }

}

