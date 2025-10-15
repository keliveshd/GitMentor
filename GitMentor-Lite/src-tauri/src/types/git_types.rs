use serde::{Deserialize, Serialize};

/// 文件状态枚举，类似VSCode Git面板
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatusType {
    Modified,   // 已修改
    Added,      // 新增
    Deleted,    // 已删除
    Renamed,    // 重命名
    Copied,     // 复制
    Untracked,  // 未跟踪
    Ignored,    // 忽略
    Conflicted, // 冲突
}

/// 文件在工作区和暂存区的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub working_tree_status: Option<FileStatusType>, // 工作区状态
    pub index_status: Option<FileStatusType>,        // 暂存区状态
    pub selected: bool,
    pub is_staged: bool, // 是否已暂存
}

/// Git仓库状态结果，类似VSCode Git面板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResult {
    pub branch: String,
    pub has_changes: bool,
    pub staged_files: Vec<FileStatus>,     // 暂存区文件
    pub unstaged_files: Vec<FileStatus>,   // 工作区文件
    pub untracked_files: Vec<FileStatus>,  // 未跟踪文件
    pub conflicted_files: Vec<FileStatus>, // 冲突文件
    pub ahead: u32,                        // 领先远程分支的提交数
    pub behind: u32,                       // 落后远程分支的提交数
}

/// 提交请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub message: String,
    pub selected_files: Vec<String>,
    pub additional_context: Option<String>,
    pub amend: bool, // 是否修正上次提交
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
    pub stage: bool, // true为暂存，false为取消暂存
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
    WorkingTree, // 回滚工作区更改到HEAD状态
    Staged,      // 取消暂存（将暂存区重置到HEAD，保留工作区更改）
    DiscardAll,  // 撤销所有更改（工作区和暂存区都重置到HEAD状态）
    Commit,      // 回滚提交
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

/// 文件差异请求
/// 作者：Evilek
/// 编写日期：2025-01-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiffRequest {
    pub file_path: String,
    pub diff_type: DiffType,
}

/// 差异类型
/// 作者：Evilek
/// 编写日期：2025-01-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffType {
    WorkingTree,   // 工作区与暂存区的差异
    Staged,        // 暂存区与HEAD的差异
    HeadToWorking, // HEAD与工作区的差异
}

/// 差异行类型
/// 作者：Evilek
/// 编写日期：2025-01-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLineType {
    Context, // 上下文行（未修改）
    Delete,  // 删除行
    Insert,  // 新增行
}

/// 差异行
/// 作者：Evilek
/// 编写日期：2025-01-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_number: Option<u32>,
    pub new_line_number: Option<u32>,
}

/// 差异块（Hunk）
/// 作者：Evilek
/// 编写日期：2025-01-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

/// 文件差异结果
/// 作者：Evilek
/// 编写日期：2025-01-18
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiffResult {
    pub file_path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub old_file_name: Option<String>,
    pub new_file_name: Option<String>,
    pub file_language: Option<String>,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
    pub is_new_file: bool,
    pub is_deleted_file: bool,
}

// 日报生成相关数据结构 - Author: Evilek, Date: 2025-08-21

/// 仓库信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub path: String,
    pub status: String,
}

/// 贡献者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub name: String,
    pub email: String,
    pub commit_count: u32,
}

/// 分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub repoPaths: Vec<String>,
    pub userEmails: Vec<String>,
    pub startDate: String,
    pub endDate: String,
}

/// 提交分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAnalysis {
    pub total_commits: u32,
    pub commits_by_user: std::collections::HashMap<String, Vec<CommitInfo>>,
    pub commits_by_repo: std::collections::HashMap<String, Vec<CommitInfo>>,
    pub file_changes: std::collections::HashMap<String, u32>,
    pub analysis_period: String,
}

/// 生成的报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub title: String,
    pub content: String,
    pub format: String,
    pub created_at: String,
    pub config: AnalysisConfig,
}

/// 报告元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMeta {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub repos: Vec<String>,
    pub users: Vec<String>,
    pub day_count: u32,
}

/// 单个提交的详细分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetailAnalysis {
    pub commit_id: String,
    pub repo_path: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
    pub message: String,
    pub files_changed: Vec<CommitFileChange>,
    pub insertions: u32,
    pub deletions: u32,
    pub summary: String,
    pub impact_level: ImpactLevel,
    pub tags: Vec<String>,
}

/// 文件变更详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFileChange {
    pub file_path: String,
    pub change_type: FileChangeType,
    pub insertions: u32,
    pub deletions: u32,
    pub is_binary: bool,
    pub language: Option<String>,
}

/// 文件变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

/// 提交影响级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum ImpactLevel {
    Low,      // 小改动（格式、注释等）
    Medium,   // 一般功能改动
    High,     // 重要功能改动
    Critical, // 核心功能或架构改动
}

/// 从RiskLevel转换为ImpactLevel
impl From<crate::types::git_types::RiskLevel> for ImpactLevel {
    fn from(risk_level: crate::types::git_types::RiskLevel) -> Self {
        match risk_level {
            crate::types::git_types::RiskLevel::Low => ImpactLevel::Low,
            crate::types::git_types::RiskLevel::Medium => ImpactLevel::Medium,
            crate::types::git_types::RiskLevel::High => ImpactLevel::High,
            crate::types::git_types::RiskLevel::Critical => ImpactLevel::Critical,
        }
    }
}

/// AI分析模板类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIAnalysisTemplate {
    /// 单个提交AI分析模板
    CommitAnalysis {
        /// 分析深度
        depth: AnalysisDepth,
        /// 是否包含代码审查
        include_code_review: bool,
    },
    /// 日报汇总AI分析模板
    DailySummary {
        /// 是否包含技术栈分析
        include_tech_analysis: bool,
        /// 是否包含风险评估
        include_risk_assessment: bool,
    },
}

/// 分析深度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisDepth {
    /// 简单分析：主要变更和影响
    Simple,
    /// 详细分析：包含代码质量、建议等
    Detailed,
    /// 深度分析：包含架构影响、性能考虑等
    Deep,
}

/// AI分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    /// 分析ID
    pub analysis_id: String,
    /// 提交ID
    pub commit_id: String,
    /// 分析类型
    pub analysis_type: AIAnalysisTemplate,
    /// 分析内容
    pub content: String,
    /// 关键发现
    pub key_findings: Vec<String>,
    /// 建议和改进点
    pub suggestions: Vec<String>,
    /// 风险评估
    pub risk_assessment: Option<RiskAssessment>,
    /// 分析时间
    pub analyzed_at: i64,
    /// AI模型信息
    pub ai_model: String,
    /// 分析耗时（毫秒）
    pub analysis_duration_ms: u64,
}

/// 风险评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// 风险级别
    pub level: RiskLevel,
    /// 风险描述
    pub description: String,
    /// 缓解措施
    pub mitigation: Vec<String>,
}

/// 风险级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,      // 低风险
    Medium,   // 中等风险
    High,     // 高风险
    Critical, // 严重风险
}

/// 分析进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProgress {
    /// 分析会话ID
    pub session_id: String,
    /// 总步骤数
    pub total_steps: u32,
    /// 当前步骤
    pub current_step: u32,
    /// 当前状态描述
    pub current_status: String,
    /// 进度百分比
    pub progress_percentage: u32,
    /// 正在分析的文件（如果有）
    pub current_file: Option<String>,
}

/// AI分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisConfig {
    /// 使用的AI模型
    pub model: String,
    /// 分析深度
    pub depth: AnalysisDepth,
    /// 是否启用代码审查
    pub enable_code_review: bool,
    /// 最大代码长度限制
    pub max_code_length: usize,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
}

/// 默认AI分析配置（模型名称将在运行时从AI管理器获取）
impl Default for AIAnalysisConfig {
    fn default() -> Self {
        Self {
            model: String::new(), // 空字符串表示使用默认AI配置中的模型
            depth: AnalysisDepth::Detailed,
            enable_code_review: true,
            max_code_length: 50000,
            timeout_seconds: 60,
        }
    }
}

/// 模板类型 - 保持向后兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateType {
    CommitAnalysis, // 单个提交分析模板
    DailySummary,   // 日报汇总模板
    AIAnalysis,     // AI分析模板（新增）
}

/// 模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub template_type: TemplateType,
    pub template_content: String,
    pub variables: Vec<String>,
    pub is_default: bool,
}
