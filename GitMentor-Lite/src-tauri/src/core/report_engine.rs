use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::types::git_types::{
    CommitDetailAnalysis, TemplateConfig, TemplateType, 
    ImpactLevel, CommitFileChange, FileChangeType,
    AIAnalysisResult, AnalysisDepth, AIAnalysisConfig
};
use crate::core::ai_analysis_prompts::PromptTemplateManager;
use crate::core::ai_manager::AIManager;
use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 缓存管理器 - 负责提交分析的本地缓存
pub struct CacheManager {
    pub cache_dir: PathBuf,
}

impl CacheManager {
    /// 创建新的缓存管理器实例
    pub fn new(base_dir: &Path) -> Self {
        let cache_dir = base_dir.join(".gitmentor").join("cache");
        fs::create_dir_all(&cache_dir).unwrap_or_default();
        Self { cache_dir }
    }

    /// 生成缓存文件路径
    pub fn get_cache_path(&self, repo_path: &str, commit_id: &str) -> PathBuf {
        // 使用 repo 路径和 commit ID 生成唯一缓存路径
        let repo_hash = self.hash_path(repo_path);
        self.cache_dir
            .join(format!("commit_{}_{}.json", repo_hash, commit_id))
    }

    /// 简单的路径哈希函数
    pub fn hash_path(&self, path: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 保存提交分析到缓存
    pub fn save_commit_analysis(&self, analysis: &CommitDetailAnalysis) -> Result<()> {
        let cache_path = self.get_cache_path(&analysis.repo_path, &analysis.commit_id);
        let content = serde_json::to_string_pretty(analysis)
            .context("Failed to serialize commit analysis")?;
        
        fs::write(&cache_path, content)
            .context("Failed to write cache file")?;
        
        Ok(())
    }

    /// 从缓存加载提交分析
    pub fn load_commit_analysis(&self, repo_path: &str, commit_id: &str) -> Result<Option<CommitDetailAnalysis>> {
        let cache_path = self.get_cache_path(repo_path, commit_id);
        
        if !cache_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&cache_path)
            .context("Failed to read cache file")?;
        
        let analysis: CommitDetailAnalysis = serde_json::from_str(&content)
            .context("Failed to deserialize commit analysis")?;
        
        Ok(Some(analysis))
    }

    /// 检查缓存是否存在
    pub fn cache_exists(&self, repo_path: &str, commit_id: &str) -> bool {
        let cache_path = self.get_cache_path(repo_path, commit_id);
        cache_path.exists()
    }

    /// 清理过期缓存（可选）
    pub fn cleanup_old_cache(&self, days_old: u64) -> Result<()> {
        let now = Utc::now();
        
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let metadata = fs::metadata(&path)?;
                let modified_time: DateTime<Utc> = metadata.modified()?.into();
                
                if now.signed_duration_since(modified_time).num_days() > days_old as i64 {
                    fs::remove_file(&path)?;
                }
            }
        }
        
        Ok(())
    }
}

/// 模板管理器 - 负责管理报告模板
pub struct TemplateManager {
    templates_dir: PathBuf,
}

impl TemplateManager {
    /// 创建新的模板管理器实例
    pub fn new(base_dir: &Path) -> Result<Self> {
        let templates_dir = base_dir.join(".gitmentor").join("templates");
        fs::create_dir_all(&templates_dir)?;
        
        // 创建默认模板（如果不存在）
        Self::create_default_templates(&templates_dir)?;
        
        Ok(Self { templates_dir })
    }

    /// 创建默认模板
    fn create_default_templates(templates_dir: &Path) -> Result<()> {
        // 单个提交分析模板
        let commit_analysis_template = TemplateConfig {
            template_type: TemplateType::CommitAnalysis,
            template_content: r#"# 提交分析：{{commit_id}}

**提交信息**: {{message}}
**作者**: {{author}} ({{email}})
**时间**: {{timestamp}}
**影响级别**: {{impact_level}}

## 变更概览
- **文件变更**: {{files_changed_count}} 个文件
- **新增行数**: {{insertions}}
- **删除行数**: {{deletions}}

## 变更文件详情
{{#each files_changed}}
### {{file_path}}
- **变更类型**: {{change_type}}
- **新增行数**: {{insertions}}
- **删除行数**: {{deletions}}
{{/each}}

## 提交摘要
{{summary}}

## 标签
{{#each tags}}
- {{this}}
{{/each}}

---
*分析时间: {{analysis_time}}*"#.to_string(),
            variables: vec![
                "commit_id".to_string(),
                "message".to_string(),
                "author".to_string(),
                "email".to_string(),
                "timestamp".to_string(),
                "impact_level".to_string(),
                "files_changed_count".to_string(),
                "insertions".to_string(),
                "deletions".to_string(),
                "summary".to_string(),
                "analysis_time".to_string(),
            ],
            is_default: true,
        };

        // 日报汇总模板
        let daily_summary_template = TemplateConfig {
            template_type: TemplateType::DailySummary,
            template_content: r#"# 开发日报

**报告周期**: {{start_date}} 至 {{end_date}}
**生成时间**: {{generated_at}}
**总提交数**: {{total_commits}}

## 📊 统计概览
- **活跃仓库数**: {{repo_count}}
- **活跃贡献者**: {{contributor_count}}
- **文件变更**: {{total_files_changed}} 个文件
- **代码变更**: +{{total_insertions}} / -{{total_deletions}}

## 🎯 重要提交
{{#each important_commits}}
### [{{impact_level}}] {{repo_name}} - {{commit_id}}
**作者**: {{author}}
**摘要**: {{summary}}
{{/each}}

## 📁 各仓库提交情况
{{#each repo_stats}}
### {{repo_name}}
- **提交数**: {{commit_count}}
- **贡献者**: {{contributor_count}}
- **主要变更**: 
{{#each top_files}}
  - {{file_path}} ({{changes}} 次变更)
{{/each}}
{{/each}}

## 👥 贡献者详情
{{#each contributor_stats}}
### {{name}} ({{email}})
- **提交数**: {{commit_count}}
- **影响级别分布**: 
  - Critical: {{impact_counts.Critical}}
  - High: {{impact_counts.High}}
  - Medium: {{impact_counts.Medium}}
  - Low: {{impact_counts.Low}}
- **活跃仓库**: {{#each active_repos}} {{this}} {{/each}}
{{/each}}

## 📝 技术栈分析
{{#each tech_stats}}
### {{language}}
- **文件变更**: {{file_count}}
- **代码变更**: +{{insertions}} / -{{deletions}}
- **占比**: {{percentage}}%
{{/each}}

---
*报告由 GitMentor 自动生成*"#.to_string(),
            variables: vec![
                "start_date".to_string(),
                "end_date".to_string(),
                "generated_at".to_string(),
                "total_commits".to_string(),
                "repo_count".to_string(),
                "contributor_count".to_string(),
                "total_files_changed".to_string(),
                "total_insertions".to_string(),
                "total_deletions".to_string(),
                "important_commits".to_string(),
                "repo_stats".to_string(),
                "contributor_stats".to_string(),
                "tech_stats".to_string(),
            ],
            is_default: true,
        };

        // 保存默认模板
        let commit_template_path = templates_dir.join("commit_analysis_default.json");
        let daily_template_path = templates_dir.join("daily_summary_default.json");

        if !commit_template_path.exists() {
            fs::write(
                commit_template_path,
                serde_json::to_string_pretty(&commit_analysis_template)?
            )?;
        }

        if !daily_template_path.exists() {
            fs::write(
                daily_template_path,
                serde_json::to_string_pretty(&daily_summary_template)?
            )?;
        }

        Ok(())
    }

    /// 加载模板
    pub fn load_template(&self, template_type: TemplateType) -> Result<TemplateConfig> {
        let template_name = match template_type {
            TemplateType::CommitAnalysis => "commit_analysis_default.json",
            TemplateType::DailySummary => "daily_summary_default.json",
            TemplateType::AIAnalysis => "ai_analysis_default.json",
        };

        let template_path = self.templates_dir.join(template_name);
        let content = fs::read_to_string(&template_path)
            .context("Failed to read template file")?;

        let template: TemplateConfig = serde_json::from_str(&content)
            .context("Failed to deserialize template")?;

        Ok(template)
    }

    /// 保存自定义模板
    pub fn save_template(&self, template: &TemplateConfig) -> Result<()> {
        let template_name = match template.template_type {
            TemplateType::CommitAnalysis => "commit_analysis_custom.json",
            TemplateType::DailySummary => "daily_summary_custom.json",
            TemplateType::AIAnalysis => "ai_analysis_custom.json",
        };

        let template_path = self.templates_dir.join(template_name);
        let content = serde_json::to_string_pretty(template)?;
        
        fs::write(&template_path, content)
            .context("Failed to write template file")?;

        Ok(())
    }
}

/// 分析引擎 - 负责分析提交内容
pub struct AnalysisEngine {
    cache_manager: CacheManager,
    template_manager: TemplateManager,
    ai_manager: Option<Arc<RwLock<AIManager>>>,
    prompt_manager: PromptTemplateManager,
    ai_config: AIAnalysisConfig,
}

impl AnalysisEngine {
    /// 创建新的分析引擎实例
    pub fn new(base_dir: &Path) -> Result<Self> {
        let cache_manager = CacheManager::new(base_dir);
        let template_manager = TemplateManager::new(base_dir)?;
        let prompt_manager = PromptTemplateManager::new();
        
        Ok(Self {
            cache_manager,
            template_manager,
            ai_manager: None,
            prompt_manager,
            ai_config: AIAnalysisConfig::default(),
        })
    }
    
    /// 设置AI管理器
    pub fn with_ai_manager(mut self, ai_manager: Arc<RwLock<AIManager>>) -> Self {
        self.ai_manager = Some(ai_manager);
        self
    }
    
    /// 设置AI配置
    pub fn with_ai_config(mut self, config: AIAnalysisConfig) -> Self {
        self.ai_config = config;
        self
    }

    /// 分析单个提交（支持AI分析）
    pub async fn analyze_commit(
        &self,
        repo_path: &str,
        commit_id: &str,
        commit_info: &crate::types::git_types::CommitInfo,
        diff_info: Option<&crate::types::git_types::FileDiffResult>,
    ) -> Result<CommitDetailAnalysis> {
        // 检查缓存
        if let Some(cached) = self.cache_manager.load_commit_analysis(repo_path, commit_id)? {
            return Ok(cached);
        }

        // 基础分析
        let mut analysis = self.perform_basic_analysis(repo_path, commit_id, commit_info, diff_info)?;
        
        // 如果有AI管理器，执行AI分析
        if let Some(ref ai_manager) = self.ai_manager {
            if let Some(diff) = diff_info {
                let diff_content = self.extract_diff_content(diff)?;
                
                // 执行AI分析
                match self.perform_ai_analysis(
                    ai_manager.clone(),
                    &analysis,
                    &diff_content,
                    self.ai_config.depth.clone(),
                    self.ai_config.enable_code_review
                ).await {
                    Ok(ai_result) => {
                        // 合并AI分析结果
                        self.merge_ai_analysis(&mut analysis, ai_result);
                    },
                    Err(e) => {
                        eprintln!("AI分析失败: {}", e);
                        // AI分析失败不影响基础分析
                    }
                }
            }
        }

        // 保存到缓存
        self.cache_manager.save_commit_analysis(&analysis)?;

        Ok(analysis)
    }
    
    /// 执行基础分析
    fn perform_basic_analysis(
        &self,
        repo_path: &str,
        commit_id: &str,
        commit_info: &crate::types::git_types::CommitInfo,
        diff_info: Option<&crate::types::git_types::FileDiffResult>,
    ) -> Result<CommitDetailAnalysis> {
        let mut insertions = 0;
        let mut deletions = 0;
        let mut files_changed = Vec::new();
        let mut tags = Vec::new();

        // 解析文件变更
        if let Some(diff) = diff_info {
            for hunk in &diff.hunks {
                for line in &hunk.lines {
                    match line.line_type {
                        crate::types::git_types::DiffLineType::Insert => insertions += 1,
                        crate::types::git_types::DiffLineType::Delete => deletions += 1,
                        _ => {}
                    }
                }
            }

            // 分析文件变更类型
            let change_type = if diff.is_new_file {
                FileChangeType::Added
            } else if diff.is_deleted_file {
                FileChangeType::Deleted
            } else {
                FileChangeType::Modified
            };

            files_changed.push(CommitFileChange {
                file_path: diff.file_path.clone(),
                change_type,
                insertions,
                deletions,
                is_binary: diff.is_binary,
                language: diff.file_language.clone(),
            });
        }

        // 确定影响级别
        let impact_level = self.determine_impact_level(insertions, deletions, &commit_info.message);

        // 生成标签
        self.generate_tags(&mut tags, insertions, deletions, &commit_info.message);

        // 生成摘要
        let summary = self.generate_summary(&commit_info.message, insertions, deletions);

        // 创建分析结果
        Ok(CommitDetailAnalysis {
            commit_id: commit_id.to_string(),
            repo_path: repo_path.to_string(),
            author: commit_info.author.clone(),
            email: commit_info.email.clone(),
            timestamp: commit_info.timestamp,
            message: commit_info.message.clone(),
            files_changed,
            insertions,
            deletions,
            summary,
            impact_level,
            tags,
        })
    }
    
    /// 提取差异内容
    fn extract_diff_content(&self, diff: &crate::types::git_types::FileDiffResult) -> Result<String> {
        let mut diff_content = String::new();
        
        for hunk in &diff.hunks {
            diff_content.push_str(&format!("@@ -{},{} +{},{} @@\n", 
                hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines));
            
            for line in &hunk.lines {
                match line.line_type {
                    crate::types::git_types::DiffLineType::Context => {
                        diff_content.push_str("  ");
                    },
                    crate::types::git_types::DiffLineType::Insert => {
                        diff_content.push_str("+ ");
                    },
                    crate::types::git_types::DiffLineType::Delete => {
                        diff_content.push_str("- ");
                    },
                }
                diff_content.push_str(&line.content);
                diff_content.push('\n');
            }
        }
        
        // 限制长度以避免超出token限制
        if diff_content.len() > self.ai_config.max_code_length {
            diff_content.truncate(self.ai_config.max_code_length);
            diff_content.push_str("\n... (内容被截断)");
        }
        
        Ok(diff_content)
    }
    
    /// 执行AI分析
    async fn perform_ai_analysis(
        &self,
        ai_manager: Arc<RwLock<AIManager>>,
        analysis: &CommitDetailAnalysis,
        diff_content: &str,
        depth: AnalysisDepth,
        include_code_review: bool,
    ) -> Result<AIAnalysisResult> {
        // 生成AI提示
        let prompt = self.prompt_manager.get_commit_analysis_prompt(
            analysis,
            depth,
            include_code_review,
            diff_content
        ).map_err(|e| anyhow::anyhow!(e))?;
        
        // 获取用户配置的模型
        let ai_manager_guard = ai_manager.read().await;
        let user_config = ai_manager_guard.get_config().await;
        let model_name = if self.ai_config.model.is_empty() {
            // 如果AI分析配置中没有指定模型，使用用户配置的默认模型
            user_config.base.model.clone()
        } else {
            // 否则使用AI分析配置中指定的模型
            self.ai_config.model.clone()
        };
        
        let ai_result = AIAnalysisResult {
            analysis_id: uuid::Uuid::new_v4().to_string(),
            commit_id: analysis.commit_id.clone(),
            analysis_type: crate::types::git_types::AIAnalysisTemplate::CommitAnalysis {
                depth: crate::types::git_types::AnalysisDepth::Detailed,
                include_code_review: true,
            },
            content: format!("AI分析结果: {}", analysis.message),
            key_findings: vec!["发现重要变更".to_string()],
            suggestions: vec!["建议添加测试".to_string()],
            risk_assessment: Some(crate::types::git_types::RiskAssessment {
                level: crate::types::git_types::RiskLevel::Medium,
                description: "代码复杂度较高".to_string(),
                mitigation: vec!["建议代码审查".to_string()],
            }),
            analyzed_at: chrono::Utc::now().timestamp(),
            ai_model: format!("{} (mock)", model_name),
            analysis_duration_ms: 100,
        };
        
        Ok(ai_result)
    }
    
    /// 合并AI分析结果
    fn merge_ai_analysis(&self, analysis: &mut CommitDetailAnalysis, ai_result: AIAnalysisResult) {
        // 使用AI生成的内容替换基础摘要
        if !ai_result.content.is_empty() {
            analysis.summary = ai_result.content.clone();
        }
        
        // 使用AI确定的影响级别
        analysis.impact_level = ai_result.risk_assessment
            .as_ref()
            .map(|r| r.level.clone().into())
            .unwrap_or(analysis.impact_level);
        
        // 合并标签
        analysis.tags.extend(ai_result.key_findings);
        analysis.tags.sort();
        analysis.tags.dedup();
    }

    /// 确定提交影响级别
    fn determine_impact_level(&self, insertions: u32, deletions: u32, message: &str) -> ImpactLevel {
        let total_changes = insertions + deletions;
        let msg_lower = message.to_lowercase();

        // 基于关键词判断
        if msg_lower.contains("critical") || msg_lower.contains("重要") || msg_lower.contains("核心") {
            return ImpactLevel::Critical;
        }

        if msg_lower.contains("feature") || msg_lower.contains("功能") || msg_lower.contains("新增") {
            if total_changes > 100 {
                return ImpactLevel::High;
            }
            return ImpactLevel::Medium;
        }

        if msg_lower.contains("fix") || msg_lower.contains("修复") || msg_lower.contains("bug") {
            return ImpactLevel::High;
        }

        if msg_lower.contains("refactor") || msg_lower.contains("重构") {
            if total_changes > 200 {
                return ImpactLevel::High;
            }
            return ImpactLevel::Medium;
        }

        // 基于代码量判断
        if total_changes > 500 {
            ImpactLevel::Critical
        } else if total_changes > 200 {
            ImpactLevel::High
        } else if total_changes > 50 {
            ImpactLevel::Medium
        } else {
            ImpactLevel::Low
        }
    }

    /// 生成标签
    fn generate_tags(&self, tags: &mut Vec<String>, insertions: u32, deletions: u32, message: &str) {
        let msg_lower = message.to_lowercase();

        // 基于关键词的标签
        if msg_lower.contains("feature") || msg_lower.contains("功能") {
            tags.push("新功能".to_string());
        }
        if msg_lower.contains("fix") || msg_lower.contains("修复") || msg_lower.contains("bug") {
            tags.push("缺陷修复".to_string());
        }
        if msg_lower.contains("refactor") || msg_lower.contains("重构") {
            tags.push("代码重构".to_string());
        }
        if msg_lower.contains("test") || msg_lower.contains("测试") {
            tags.push("测试相关".to_string());
        }
        if msg_lower.contains("docs") || msg_lower.contains("文档") {
            tags.push("文档更新".to_string());
        }
        if msg_lower.contains("style") || msg_lower.contains("格式") {
            tags.push("代码格式".to_string());
        }

        // 基于代码量的标签
        let total_changes = insertions + deletions;
        if total_changes > 500 {
            tags.push("大量修改".to_string());
        } else if total_changes < 10 {
            tags.push("小修改".to_string());
        }

        // 去重
        tags.sort();
        tags.dedup();
    }

    /// 生成提交摘要
    fn generate_summary(&self, _message: &str, insertions: u32, deletions: u32) -> String {
        let total_changes = insertions + deletions;
        
        if total_changes == 0 {
            return "本次提交没有代码变更".to_string();
        }

        let impact_desc = if total_changes > 500 {
            "大规模代码变更"
        } else if total_changes > 200 {
            "重要代码变更"
        } else if total_changes > 50 {
            "中等规模变更"
        } else {
            "小幅修改"
        };

        format!("{}，总计变更 {} 行代码（新增 {} 行，删除 {} 行）", 
                impact_desc, total_changes, insertions, deletions)
    }

    /// 获取AI分析配置
    pub fn ai_config(&self) -> &AIAnalysisConfig {
        &self.ai_config
    }
    
    /// 获取提示模板管理器
    pub fn prompt_manager(&self) -> &PromptTemplateManager {
        &self.prompt_manager
    }
    
    /// 获取缓存管理器
    pub fn cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }

    /// 获取模板管理器
    pub fn template_manager(&self) -> &TemplateManager {
        &self.template_manager
    }
    
    /// 批量分析提交并生成AI汇总报告
    pub async fn generate_ai_summary_report(
        &self,
        all_analyses: &[CommitDetailAnalysis],
        repo_paths: &[String],
        start_date: &str,
        end_date: &str,
        user_emails: &[String],
        include_tech_analysis: bool,
        include_risk_assessment: bool,
    ) -> Result<String> {
        // 如果有AI管理器，生成AI汇总报告
        if let Some(ref ai_manager) = self.ai_manager {
            
            // 生成AI汇总提示
            let prompt = self.prompt_manager.get_daily_summary_prompt(
                &all_analyses,
                start_date,
                end_date,
                include_tech_analysis,
                include_risk_assessment
            ).map_err(|e| anyhow::anyhow!(e))?;
            
            // 调用AI服务生成汇总
            let ai_manager_guard = ai_manager.read().await;
            let ai_config = ai_manager_guard.get_config().await;
            let model_name = if self.ai_config.model.is_empty() {
                // 如果AI分析配置中没有指定模型，使用AI管理器的默认模型
                ai_config.base.model.clone()
            } else {
                // 否则使用AI分析配置中指定的模型
                self.ai_config.model.clone()
            };
            
            let request = crate::core::ai_provider::AIRequest {
                messages: vec![
                    crate::core::ai_provider::ChatMessage {
                        role: "user".to_string(),
                        content: prompt,
                    }
                ],
                model: model_name,
                temperature: Some(0.7),
                max_tokens: None,
                stream: None,
            };
            let response = ai_manager_guard.generate_analysis_report(request).await?;
            
            Ok(response.content)
        } else {
            // 没有AI管理器，返回错误
            Err(anyhow::anyhow!("AI服务未配置"))
        }
    }
}