use serde::{Deserialize, Serialize};

/// 文件状态枚举，类似VSCode Git面板
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatusType {
    Modified,      // 已修改
    Added,         // 新增
    Deleted,       // 已删除
    Renamed,       // 重命名
    Copied,        // 复制
    Untracked,     // 未跟踪
    Ignored,       // 忽略
    Conflicted,    // 冲突
}

/// 文件在工作区和暂存区的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub working_tree_status: Option<FileStatusType>,  // 工作区状态
    pub index_status: Option<FileStatusType>,         // 暂存区状态
    pub selected: bool,
    pub is_staged: bool,                              // 是否已暂存
}

/// Git仓库状态结果，类似VSCode Git面板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResult {
    pub branch: String,
    pub has_changes: bool,
    pub staged_files: Vec<FileStatus>,      // 暂存区文件
    pub unstaged_files: Vec<FileStatus>,    // 工作区文件
    pub untracked_files: Vec<FileStatus>,   // 未跟踪文件
    pub conflicted_files: Vec<FileStatus>,  // 冲突文件
    pub ahead: u32,                         // 领先远程分支的提交数
    pub behind: u32,                        // 落后远程分支的提交数
}

/// 提交请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub message: String,
    pub selected_files: Vec<String>,
    pub additional_context: Option<String>,
    pub amend: bool,                        // 是否修正上次提交
}

/// 提交消息生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessageResult {
    pub message: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

/// 提交历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
    pub files_changed: Vec<String>,
}

/// 暂存操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRequest {
    pub file_paths: Vec<String>,
    pub stage: bool,  // true为暂存，false为取消暂存
}

/// 回滚操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertRequest {
    pub file_paths: Vec<String>,
    pub revert_type: RevertType,
}

/// 回滚类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevertType {
    WorkingTree,  // 回滚工作区更改
    Staged,       // 回滚暂存区更改
    Commit,       // 回滚提交
}

/// 分支信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
}

/// Git操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOperationResult {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}
