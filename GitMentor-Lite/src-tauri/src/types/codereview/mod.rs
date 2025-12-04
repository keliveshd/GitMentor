/**
 * Code Review 模块 Rust 类型定义
 * 作者：Evilek
 * 日期：2025-01-04
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 审查深度枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewDepth {
    #[serde(rename = "BASIC")]
    Basic,
    #[serde(rename = "STANDARD")]
    Standard,
    #[serde(rename = "DEEP")]
    Deep,
}

/// 审查类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewType {
    #[serde(rename = "AI")]
    AI,
    #[serde(rename = "MANUAL")]
    Manual,
    #[serde(rename = "LINT")]
    Lint,
    #[serde(rename = "MIXED")]
    Mixed,
}

/// 审查范围枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewScope {
    #[serde(rename = "SINGLE_FILE")]
    SingleFile,
    #[serde(rename = "MULTIPLE_FILES")]
    MultipleFiles,
    #[serde(rename = "COMMIT")]
    Commit,
    #[serde(rename = "BRANCH")]
    Branch,
}

/// 审查状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "IN_PROGRESS")]
    InProgress,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "APPROVED")]
    Approved,
    #[serde(rename = "CHANGES_REQUESTED")]
    ChangesRequested,
    #[serde(rename = "REJECTED")]
    Rejected,
}

/// 问题严重性枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    #[serde(rename = "CRITICAL")]
    Critical,
    #[serde(rename = "HIGH")]
    High,
    #[serde(rename = "MEDIUM")]
    Medium,
    #[serde(rename = "LOW")]
    Low,
    #[serde(rename = "INFO")]
    Info,
}

/// 问题类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    #[serde(rename = "CODE_SMELL")]
    CodeSmell,
    #[serde(rename = "COMPLEXITY")]
    Complexity,
    #[serde(rename = "PERFORMANCE")]
    Performance,
    #[serde(rename = "INEFFICIENT")]
    Inefficient,
    #[serde(rename = "SECURITY")]
    Security,
    #[serde(rename = "VULNERABILITY")]
    Vulnerability,
    #[serde(rename = "BEST_PRACTICE")]
    BestPractice,
    #[serde(rename = "STYLE")]
    Style,
    #[serde(rename = "BUG")]
    Bug,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "MAINTAINABILITY")]
    Maintainability,
    #[serde(rename = "READABILITY")]
    Readability,
    #[serde(rename = "DOCUMENTATION")]
    Documentation,
    #[serde(rename = "DEPRECATED")]
    Deprecated,
    #[serde(rename = "UNUSED")]
    Unused,
}

/// 建议类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionType {
    #[serde(rename = "REFACTOR")]
    Refactor,
    #[serde(rename = "OPTIMIZE")]
    Optimize,
    #[serde(rename = "FIX")]
    Fix,
    #[serde(rename = "DOCUMENT")]
    Document,
    #[serde(rename = "TEST")]
    Test,
    #[serde(rename = "REMOVE")]
    Remove,
    #[serde(rename = "EXTRACT")]
    Extract,
}

/// 审查文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFile {
    pub path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub diff: Option<String>,
    pub language: String,
    pub size: usize,
}

/// 代码问题结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub file: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub issue_type: IssueType,
    pub title: String,
    pub description: String,
    pub code_snippet: String,
    pub suggestion: Option<String>,
    pub references: Option<Vec<String>>,
    pub cwe_id: Option<String>,
}

/// 代码建议结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub id: String,
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub effort: String, // "LOW" | "MEDIUM" | "HIGH"
    pub files_affected: Vec<String>,
}

/// 审查指标结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewMetrics {
    pub total_files: u32,
    pub total_lines: u32,
    pub issues_found: u32,
    pub issues_by_severity: IssuesBySeverity,
    pub issues_by_type: IssuesByType,
    pub complexity_score: Option<f64>,
    pub maintainability_index: Option<f64>,
}

/// 按严重性分类的问题数量
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssuesBySeverity {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub info: u32,
}

/// 按类型分类的问题数量
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssuesByType {
    pub code_smell: Option<u32>,
    pub performance: Option<u32>,
    pub security: Option<u32>,
    pub bug: Option<u32>,
    pub best_practice: Option<u32>,
    pub style: Option<u32>,
    pub error: Option<u32>,
    pub maintainability: Option<u32>,
    pub readability: Option<u32>,
    pub documentation: Option<u32>,
    pub deprecated: Option<u32>,
    pub unused: Option<u32>,
    pub complexity: Option<u32>,
    pub inefficient: Option<u32>,
    pub vulnerability: Option<u32>,
}

/// AI 审查结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReviewResult {
    pub provider: String,
    pub model: String,
    pub depth: ReviewDepth,
    pub token_count: u32,
    pub response_time: u64,
    pub summary: String,
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<CodeSuggestion>,
    pub metrics: ReviewMetrics,
}

/// AI 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String,
    pub model: String,
    pub default_depth: ReviewDepth,
    pub timeout: u64,
    pub retries: u32,
    pub streaming: bool,
    pub max_tokens: u32,
    pub temperature: f64,
    pub custom_prompts: HashMap<String, String>,
}

/// 审查配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    pub version: String,
    pub ai: AIConfig,
}

/// 审查记录结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecord {
    pub id: String,
    pub timestamp: String,
    pub review_type: ReviewType,
    pub scope: ReviewScope,
    pub branch: String,
    pub commit_hash: Option<String>,
    pub files: Vec<ReviewFile>,
    pub config: ReviewConfig,
    pub status: ReviewStatus,
    pub duration: u64,
    pub results: ReviewResults,
    pub creator: String,
    pub summary: Option<String>,
}

/// 审查结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResults {
    pub ai: Option<AIReviewResult>,
}

/// 审查筛选条件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub review_types: Option<Vec<ReviewType>>,
    pub statuses: Option<Vec<ReviewStatus>>,
    pub files: Option<Vec<String>>,
    pub query: Option<String>,
}

/// 分页选项结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationOptions {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// 分页结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub data: Vec<T>,
}

/// 审查统计结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStatistics {
    pub total_reviews: u32,
    pub total_files: u32,
    pub total_issues: u32,
    pub issues_by_severity: IssuesBySeverity,
    pub issues_by_type: IssuesByType,
    pub average_review_time: f64,
    pub token_usage: TokenUsage,
}

/// Token 使用统计结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub total: u32,
    pub by_provider: HashMap<String, u32>,
    pub by_model: HashMap<String, u32>,
    pub cost: f64,
}

/// 审查错误枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewError {
    #[serde(rename = "REPOSITORY_NOT_SELECTED")]
    RepositoryNotSelected,
    #[serde(rename = "FILES_NOT_FOUND")]
    FilesNotFound,
    #[serde(rename = "AI_PROVIDER_ERROR")]
    AIProviderError,
    #[serde(rename = "TOKEN_LIMIT_EXCEEDED")]
    TokenLimitExceeded,
    #[serde(rename = "INVALID_CONFIG")]
    InvalidConfig,
    #[serde(rename = "PERMISSION_DENIED")]
    PermissionDenied,
    #[serde(rename = "CACHE_ERROR")]
    CacheError,
    #[serde(rename = "SERIALIZATION_ERROR")]
    SerializationError,
    #[serde(rename = "FILESYSTEM_ERROR")]
    FilesystemError,
    #[serde(rename = "TIMEOUT")]
    Timeout,
    #[serde(rename = "INTERRUPTED")]
    Interrupted,
}

/// 审查错误信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewErrorInfo {
    pub code: ReviewError,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: String,
}
