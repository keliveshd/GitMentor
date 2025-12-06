/**
 * Code Review 核心引擎
 * 负责协调 AI 审查、存储管理和流程控制
 * 作者：Evilek
 * 日期：2025-01-04
 */

use anyhow::{Result, Context, anyhow};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::ai_manager::AIManager;
use crate::core::ai_provider::{AIRequest, ChatMessage};
use crate::types::codereview::{
    ReviewRecord, ReviewFile, ReviewConfig, ReviewType, ReviewScope,
    ReviewStatus, ReviewDepth, AIReviewResult, CodeIssue, ReviewResults,
    ReviewFilters, PaginationOptions, PaginatedResult
};

use super::storage::ReviewStorage;

/// 审查引擎结构
pub struct CodeReviewEngine {
    ai_manager: Arc<RwLock<AIManager>>,
    storage: Arc<RwLock<ReviewStorage>>,
}

/// 提示词模板
const BASIC_PROMPT: &str = r#"你是一个代码审查助手。请分析以下代码，重点关注：
1. 语法错误
2. 明显的逻辑错误
3. 基本的安全问题

请以 JSON 格式返回结果，包含：
- summary: 总体评价
- issues: 问题列表，每个问题包含 title, description, severity, issue_type
- suggestions: 改进建议"#;

const STANDARD_PROMPT: &str = r#"你是一个资深的代码审查助手。请分析以下代码，重点关注：
1. 代码质量（可读性、可维护性）
2. 性能问题
3. 常见的安全漏洞
4. 最佳实践
5. 代码异味

请以 JSON 格式返回结果，包含：
- summary: 总体评价
- issues: 问题列表，每个问题包含 title, description, severity, issue_type, suggestion
- suggestions: 改进建议"#;

const DEEP_PROMPT: &str = r#"你是一个架构师级别的代码审查专家。请深度分析以下代码，关注：
1. 架构设计问题
2. 安全漏洞和风险
3. 性能瓶颈和优化点
4. 复杂度和可维护性
5. 设计模式应用
6. 潜在的扩展性问题
7. 依赖管理问题

请以 JSON 格式返回结果，包含：
- summary: 详细分析总结
- issues: 问题列表，包含严重性、影响范围、修复建议
- suggestions: 重构和优化建议"#;

impl CodeReviewEngine {
    /// 创建新的审查引擎
    pub fn new(
        ai_manager: Arc<RwLock<AIManager>>,
        storage: Arc<RwLock<ReviewStorage>>,
    ) -> Self {
        Self {
            ai_manager,
            storage,
        }
    }

    /// 执行 AI 代码审查
    pub async fn analyze_with_ai(
        &self,
        files: Vec<String>,
        depth: ReviewDepth,
        config: ReviewConfig,
        current_branch: String,
        creator: String,
    ) -> Result<ReviewRecord> {
        let start_time = Instant::now();

        // 验证输入
        if files.is_empty() {
            return Err(anyhow!("没有选择要审查的文件"));
        }

        // 创建审查记录 ID
        let review_id = Uuid::new_v4().to_string();

        // 读取文件内容
        let review_files = self.read_files(&files).await
            .context("读取文件内容失败")?;

        // 构建提示词
        let prompt = self.build_prompt(&review_files, depth, &config)
            .context("构建提示词失败")?;

        // 调用 AI 管理器进行代码审查
        let ai_manager = self.ai_manager.read().await;
        let request = AIRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            model: config.ai.model.clone(),
            temperature: Some(config.ai.temperature as f32),
            max_tokens: Some(config.ai.max_tokens),
            stream: Some(config.ai.streaming),
        };
        let ai_response = ai_manager.generate_commit_message(request)
            .await
            .context("AI 审查失败")?;

        let duration = start_time.elapsed();

        // 解析 AI 响应
        let ai_result = self.parse_ai_response(ai_response.content, &review_files)
            .context("解析 AI 响应失败")?;

        // 创建审查记录
        let review = ReviewRecord {
            id: review_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            review_type: ReviewType::AI,
            scope: if files.len() == 1 {
                ReviewScope::SingleFile
            } else {
                ReviewScope::MultipleFiles
            },
            branch: current_branch,
            commit_hash: None,
            files: review_files,
            config,
            status: ReviewStatus::Completed,
            duration: duration.as_millis() as u64,
            results: ReviewResults {
                ai: Some(ai_result),
            },
            creator,
            summary: None,
        };

        // 保存审查记录
        {
            let storage = self.storage.write().await;
            storage.save_review(&review).await
                .context("保存审查记录失败")?;
        }

        Ok(review)
    }

    /// 读取文件内容
    async fn read_files(&self, file_paths: &[String]) -> Result<Vec<ReviewFile>> {
        let mut review_files = Vec::new();

        for path in file_paths {
            // 读取文件内容
            let content = tokio::fs::read_to_string(path)
                .await
                .context(format!("读取文件失败: {}", path))?;

            // 检测编程语言
            let language = detect_language(path, &content);

            // 获取文件大小
            let metadata = tokio::fs::metadata(path).await
                .context(format!("获取文件元数据失败: {}", path))?;
            let size = metadata.len() as usize;

            // 检查文件大小限制（1MB）
            if size > 1024 * 1024 {
                return Err(anyhow!("文件过大: {} ({:?})", path, size));
            }

            review_files.push(ReviewFile {
                path: path.clone(),
                old_content: None,
                new_content: Some(content),
                diff: None,
                language,
                size,
            });
        }

        Ok(review_files)
    }

    /// 构建 AI 提示词
    fn build_prompt(
        &self,
        files: &[ReviewFile],
        depth: ReviewDepth,
        config: &ReviewConfig,
    ) -> Result<String> {
        // 选择基础模板
        let template = match depth {
            ReviewDepth::Basic => BASIC_PROMPT,
            ReviewDepth::Standard => STANDARD_PROMPT,
            ReviewDepth::Deep => DEEP_PROMPT,
        };

        // 格式化文件内容
        let formatted_content = files
            .iter()
            .map(|f| {
                format!(
                    "文件: {}\n语言: {}\n大小: {} 字节\n内容:\n{}",
                    f.path, f.language, f.size, f.new_content.as_ref().unwrap_or(&String::new())
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        // 检查是否有自定义提示词
        let custom_prompt = config.ai.custom_prompts.get(&depth.to_string());

        let prompt = if let Some(custom) = custom_prompt {
            format!("{}\n\n{}\n\n待审查的代码:\n{}", custom, template, formatted_content)
        } else {
            format!("{}\n\n待审查的代码:\n{}", template, formatted_content)
        };

        Ok(prompt)
    }

    /// 解析 AI 响应
    fn parse_ai_response(
        &self,
        ai_response: String,
        files: &[ReviewFile],
    ) -> Result<AIReviewResult> {
        // 尝试解析 JSON 响应
        let parsed: serde_json::Value = serde_json::from_str(&ai_response)
            .context("AI 响应不是有效的 JSON")?;

        // 提取摘要
        let summary = parsed
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("无摘要")
            .to_string();

        // 提取问题列表
        let mut issues = Vec::new();
        if let Some(issues_array) = parsed.get("issues") {
            if let Some(issues_list) = issues_array.as_array() {
                for (idx, issue_value) in issues_list.iter().enumerate() {
                    let issue = self.parse_code_issue(issue_value, &files, idx)?;
                    issues.push(issue);
                }
            }
        }

        // 提取建议
        let suggestions = Vec::new(); // TODO: 实现建议解析

        // 计算指标
        let metrics = self.calculate_metrics(&issues);

        Ok(AIReviewResult {
            provider: "unknown".to_string(), // TODO: 从 AI 管理器获取
            model: "unknown".to_string(),    // TODO: 从配置获取
            depth: ReviewDepth::Standard,    // TODO: 从参数获取
            token_count: 0,                  // TODO: 从 AI 响应获取
            response_time: 0,                // TODO: 记录实际时间
            summary,
            issues,
            suggestions,
            metrics,
        })
    }

    /// 解析单个代码问题
    fn parse_code_issue(
        &self,
        issue_value: &serde_json::Value,
        files: &[ReviewFile],
        index: usize,
    ) -> Result<CodeIssue> {
        let title = issue_value
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("未命名问题")
            .to_string();

        let description = issue_value
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("无描述")
            .to_string();

        let severity_str = issue_value
            .get("severity")
            .and_then(|s| s.as_str())
            .unwrap_or("medium");

        let severity = match severity_str.to_lowercase().as_str() {
            "critical" => crate::types::codereview::IssueSeverity::Critical,
            "high" => crate::types::codereview::IssueSeverity::High,
            "medium" => crate::types::codereview::IssueSeverity::Medium,
            "low" => crate::types::codereview::IssueSeverity::Low,
            "info" => crate::types::codereview::IssueSeverity::Info,
            _ => crate::types::codereview::IssueSeverity::Medium,
        };

        let issue_type_str = issue_value
            .get("issue_type")
            .and_then(|t| t.as_str())
            .unwrap_or("code_smell");

        let issue_type = match issue_type_str.to_lowercase().as_str() {
            "performance" => crate::types::codereview::IssueType::Performance,
            "security" => crate::types::codereview::IssueType::Security,
            "bug" => crate::types::codereview::IssueType::Bug,
            "style" => crate::types::codereview::IssueType::Style,
            "error" => crate::types::codereview::IssueType::Error,
            _ => crate::types::codereview::IssueType::CodeSmell,
        };

        let suggestion = issue_value
            .get("suggestion")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        // 从文件中获取代码片段
        let code_snippet = self.extract_code_snippet(issue_value, files)?;

        Ok(CodeIssue {
            id: format!("issue-{}", index),
            severity,
            file: "unknown".to_string(), // TODO: 从 AI 响应中提取
            line: None,                  // TODO: 从 AI 响应中提取
            column: None,                // TODO: 从 AI 响应中提取
            issue_type,
            title,
            description,
            code_snippet,
            suggestion,
            references: None,
            cwe_id: None,
        })
    }

    /// 从文件中提取代码片段
    fn extract_code_snippet(
        &self,
        issue_value: &serde_json::Value,
        files: &[ReviewFile],
    ) -> Result<String> {
        // 尝试从 issue 中获取文件和行号信息
        if let Some(file_path) = issue_value.get("file").and_then(|f| f.as_str()) {
            if let Some(line) = issue_value.get("line").and_then(|l| l.as_u64()) {
                // 查找文件并提取代码片段
                for review_file in files {
                    if review_file.path.contains(file_path) {
                        let content = review_file.new_content.as_ref().unwrap();
                        let lines: Vec<&str> = content.lines().collect();
                        let start_line = line.saturating_sub(1) as usize;
                        let end_line = (start_line + 3).min(lines.len());
                        let snippet = lines[start_line..end_line].join("\n");
                        return Ok(snippet);
                    }
                }
            }
        }

        // 默认返回第一个文件的前几行
        if let Some(file) = files.first() {
            let content = file.new_content.as_ref().unwrap();
            let lines: Vec<&str> = content.lines().take(5).collect();
            return Ok(lines.join("\n"));
        }

        Ok("无法提取代码片段".to_string())
    }

    /// 计算审查指标
    fn calculate_metrics(&self, issues: &[CodeIssue]) -> crate::types::codereview::ReviewMetrics {
        use crate::types::codereview::{IssuesBySeverity, IssuesByType};

        let mut severity_counts = IssuesBySeverity::default();
        let mut type_counts = IssuesByType::default();

        for issue in issues {
            match issue.severity {
                crate::types::codereview::IssueSeverity::Critical => severity_counts.critical += 1,
                crate::types::codereview::IssueSeverity::High => severity_counts.high += 1,
                crate::types::codereview::IssueSeverity::Medium => severity_counts.medium += 1,
                crate::types::codereview::IssueSeverity::Low => severity_counts.low += 1,
                crate::types::codereview::IssueSeverity::Info => severity_counts.info += 1,
            }

            match issue.issue_type {
                crate::types::codereview::IssueType::CodeSmell => {
                    if let Some(count) = type_counts.code_smell.as_mut() {
                        *count += 1;
                    } else {
                        type_counts.code_smell = Some(1);
                    }
                }
                crate::types::codereview::IssueType::Performance => {
                    if let Some(count) = type_counts.performance.as_mut() {
                        *count += 1;
                    } else {
                        type_counts.performance = Some(1);
                    }
                }
                crate::types::codereview::IssueType::Security => {
                    if let Some(count) = type_counts.security.as_mut() {
                        *count += 1;
                    } else {
                        type_counts.security = Some(1);
                    }
                }
                crate::types::codereview::IssueType::Bug => {
                    if let Some(count) = type_counts.bug.as_mut() {
                        *count += 1;
                    } else {
                        type_counts.bug = Some(1);
                    }
                }
                crate::types::codereview::IssueType::Style => {
                    if let Some(count) = type_counts.style.as_mut() {
                        *count += 1;
                    } else {
                        type_counts.style = Some(1);
                    }
                }
                crate::types::codereview::IssueType::Error => {
                    if let Some(count) = type_counts.error.as_mut() {
                        *count += 1;
                    } else {
                        type_counts.error = Some(1);
                    }
                }
                _ => {}
            }
        }

        crate::types::codereview::ReviewMetrics {
            total_files: 0, // TODO: 传入文件数
            total_lines: 0, // TODO: 计算总行数
            issues_found: issues.len() as u32,
            issues_by_severity: severity_counts,
            issues_by_type: type_counts,
            complexity_score: None,
            maintainability_index: None,
        }
    }

    /// 获取审查历史
    pub async fn get_review_history(
        &self,
        filters: ReviewFilters,
        pagination: PaginationOptions,
    ) -> Result<PaginatedResult<ReviewRecord>> {
        let storage = self.storage.read().await;
        storage.search_reviews(&filters, &pagination).await
    }

    /// 获取单个审查记录
    pub async fn get_review_record(&self, id: &str) -> Result<Option<ReviewRecord>> {
        let storage = self.storage.read().await;
        storage.load_review(id).await
    }

    /// 删除审查记录
    pub async fn delete_review(&self, id: &str) -> Result<()> {
        let storage = self.storage.read().await;
        storage.delete_review(id).await
    }

    /// 清除所有审查记录
    pub async fn clear_all_reviews(&self) -> Result<()> {
        let storage = self.storage.read().await;
        storage.clear_all_reviews().await
    }

    /// 获取审查统计
    pub async fn get_review_statistics(&self) -> Result<crate::types::codereview::ReviewStatistics> {
        let storage = self.storage.read().await;
        let reviews = storage.load_all_reviews().await?;

        let total_reviews = reviews.len() as u32;
        let total_files = reviews.iter().map(|r| r.files.len()).sum::<usize>() as u32;
        let total_issues = reviews.iter()
            .map(|r| r.results.ai.as_ref().map(|ai| ai.issues.len()).unwrap_or(0))
            .sum::<usize>() as u32;

        use crate::types::codereview::{IssuesBySeverity, IssuesByType, TokenUsage};

        let mut severity_counts = IssuesBySeverity::default();
        let type_counts = IssuesByType::default();
        let mut by_provider = std::collections::HashMap::new();
        let mut by_model = std::collections::HashMap::new();
        let mut total_tokens = 0u32;
        let mut total_cost = 0.0f64;

        for review in reviews {
            if let Some(ai_result) = &review.results.ai {
                // 统计问题
                for issue in &ai_result.issues {
                    match issue.severity {
                        crate::types::codereview::IssueSeverity::Critical => severity_counts.critical += 1,
                        crate::types::codereview::IssueSeverity::High => severity_counts.high += 1,
                        crate::types::codereview::IssueSeverity::Medium => severity_counts.medium += 1,
                        crate::types::codereview::IssueSeverity::Low => severity_counts.low += 1,
                        crate::types::codereview::IssueSeverity::Info => severity_counts.info += 1,
                    }
                }

                // 统计 Token 使用
                *by_provider.entry(ai_result.provider.clone()).or_insert(0) += ai_result.token_count;
                *by_model.entry(ai_result.model.clone()).or_insert(0) += ai_result.token_count;
                total_tokens += ai_result.token_count;
                total_cost += ai_result.token_count as f64 * 0.0001; // 假设每个 Token 成本
            }
        }

        let token_usage = TokenUsage {
            total: total_tokens,
            by_provider,
            by_model,
            cost: total_cost,
        };

        Ok(crate::types::codereview::ReviewStatistics {
            total_reviews,
            total_files,
            total_issues,
            issues_by_severity: severity_counts,
            issues_by_type: type_counts,
            average_review_time: 0.0, // TODO: 计算平均时间
            token_usage,
        })
    }
}

/// 检测编程语言
fn detect_language(file_path: &str, content: &str) -> String {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "js" | "jsx" => "javascript".to_string(),
        "ts" | "tsx" => "typescript".to_string(),
        "vue" => "vue".to_string(),
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "java" => "java".to_string(),
        "cpp" | "cc" | "cxx" => "cpp".to_string(),
        "c" => "c".to_string(),
        "cs" => "csharp".to_string(),
        "php" => "php".to_string(),
        "go" => "go".to_string(),
        "rb" => "ruby".to_string(),
        "swift" => "swift".to_string(),
        "kt" => "kotlin".to_string(),
        "sql" => "sql".to_string(),
        "html" => "html".to_string(),
        "css" => "css".to_string(),
        "scss" => "scss".to_string(),
        "json" => "json".to_string(),
        "xml" => "xml".to_string(),
        "yaml" | "yml" => "yaml".to_string(),
        "md" => "markdown".to_string(),
        _ => {
            // 根据内容推断
            if content.starts_with("<?php") {
                "php".to_string()
            } else if content.starts_with("#!/usr/bin/env python") {
                "python".to_string()
            } else {
                "text".to_string()
            }
        }
    }
}
