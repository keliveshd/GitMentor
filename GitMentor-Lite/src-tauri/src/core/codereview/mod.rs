/**
 * Code Review 模块
 * 作者：Evilek
 * 日期：2025-01-04
 */

pub mod storage;
pub mod engine;

pub use storage::ReviewStorage;
pub use storage::GlobalReviewStorage;
pub use storage::create_global_review_storage;

pub use engine::CodeReviewEngine;

pub use crate::types::codereview::{
    ReviewRecord, ReviewFile, AIReviewResult, CodeIssue,
    ReviewType, ReviewScope, ReviewStatus, ReviewDepth,
    IssueSeverity, IssueType, SuggestionType,
    ReviewFilters, PaginationOptions, PaginatedResult,
    ReviewStatistics, TokenUsage, ReviewConfig,
};
