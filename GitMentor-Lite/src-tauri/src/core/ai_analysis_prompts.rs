/// AI分析提示模板模块
/// 作者：Evilek
/// 编写日期：2025-09-16
///
/// 此模块提供了AI分析所需的各类提示模板
/// 包含单体提交分析和汇总报告生成的模板
use crate::types::git_types::{AIAnalysisTemplate, AnalysisDepth, CommitDetailAnalysis};
use chrono;
use handlebars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 提示模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template_type: AIAnalysisTemplate,
    pub template_content: String,
    pub variables: Vec<String>,
    pub version: String,
}

/// 提示模板管理器
pub struct PromptTemplateManager {
    templates: HashMap<String, PromptTemplate>,
    cache_dir: std::path::PathBuf,
}

impl PromptTemplateManager {
    /// 创建新的模板管理器
    pub fn new() -> Self {
        let cache_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(".config")
            .join("ai_templates");

        let mut manager = Self {
            templates: HashMap::new(),
            cache_dir,
        };

        // 初始化默认模板
        manager.init_default_templates();

        manager
    }

    /// 创建新的模板管理器（指定应用目录）
    pub fn with_app_dir(app_dir: &std::path::Path) -> Self {
        let cache_dir = app_dir.join("ai_templates");

        let mut manager = Self {
            templates: HashMap::new(),
            cache_dir,
        };

        // 初始化默认模板
        manager.init_default_templates();

        // 尝试加载自定义模板
        manager.load_custom_templates();

        manager
    }

    /// 初始化默认模板
    fn init_default_templates(&mut self) {
        // 单体提交分析模板 - 简单版本
        self.templates.insert(
            "commit_simple".to_string(),
            PromptTemplate {
                id: "commit_simple".to_string(),
                name: "单体提交分析（简单版）".to_string(),
                description: "对单个提交进行基础分析，识别主要变更和影响".to_string(),
                template_type: AIAnalysisTemplate::CommitAnalysis {
                    depth: AnalysisDepth::Simple,
                    include_code_review: false,
                },
                template_content: include_str!("../templates/commit_analysis_simple.hbs")
                    .to_string(),
                variables: vec![
                    "commit_id".to_string(),
                    "author".to_string(),
                    "timestamp".to_string(),
                    "message".to_string(),
                    "files_changed".to_string(),
                    "diff_content".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        );

        // 单体提交分析模板 - 详细版本
        self.templates.insert(
            "commit_detailed".to_string(),
            PromptTemplate {
                id: "commit_detailed".to_string(),
                name: "单体提交分析（详细版）".to_string(),
                description: "对单个提交进行深入分析，包含代码质量和建议".to_string(),
                template_type: AIAnalysisTemplate::CommitAnalysis {
                    depth: AnalysisDepth::Detailed,
                    include_code_review: true,
                },
                template_content: include_str!("../templates/commit_analysis_detailed.hbs")
                    .to_string(),
                variables: vec![
                    "commit_id".to_string(),
                    "author".to_string(),
                    "timestamp".to_string(),
                    "message".to_string(),
                    "files_changed".to_string(),
                    "diff_content".to_string(),
                    "code_language".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        );

        // 单体提交分析模板 - 深度版本
        self.templates.insert(
            "commit_deep".to_string(),
            PromptTemplate {
                id: "commit_deep".to_string(),
                name: "单体提交分析（深度版）".to_string(),
                description: "最深入的分析，包含架构影响和性能考虑".to_string(),
                template_type: AIAnalysisTemplate::CommitAnalysis {
                    depth: AnalysisDepth::Deep,
                    include_code_review: true,
                },
                template_content: include_str!("../templates/commit_analysis_deep.hbs").to_string(),
                variables: vec![
                    "commit_id".to_string(),
                    "author".to_string(),
                    "timestamp".to_string(),
                    "message".to_string(),
                    "files_changed".to_string(),
                    "diff_content".to_string(),
                    "code_language".to_string(),
                    "repo_context".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        );

        // 日报汇总模板 - 基础版本
        self.templates.insert(
            "daily_summary_basic".to_string(),
            PromptTemplate {
                id: "daily_summary_basic".to_string(),
                name: "日报汇总（基础版）".to_string(),
                description: "基于缓存的提交分析生成基础日报汇总".to_string(),
                template_type: AIAnalysisTemplate::DailySummary {
                    include_tech_analysis: false,
                    include_risk_assessment: false,
                },
                template_content: include_str!("../templates/daily_summary_basic.hbs").to_string(),
                variables: vec![
                    "analyses".to_string(),
                    "start_date".to_string(),
                    "end_date".to_string(),
                    "total_commits".to_string(),
                    "contributors".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        );

        // 日报汇总模板 - 增强版本
        self.templates.insert(
            "daily_summary_enhanced".to_string(),
            PromptTemplate {
                id: "daily_summary_enhanced".to_string(),
                name: "日报汇总（增强版）".to_string(),
                description: "包含技术分析和风险评估的完整日报汇总".to_string(),
                template_type: AIAnalysisTemplate::DailySummary {
                    include_tech_analysis: true,
                    include_risk_assessment: true,
                },
                template_content: include_str!("../templates/daily_summary_enhanced.hbs")
                    .to_string(),
                variables: vec![
                    "analyses".to_string(),
                    "start_date".to_string(),
                    "end_date".to_string(),
                    "total_commits".to_string(),
                    "contributors".to_string(),
                    "tech_stack".to_string(),
                    "risk_factors".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        );

        // 日报汇总模板 - 优化版本
        self.templates.insert(
            "daily_summary_optimized".to_string(),
            PromptTemplate {
                id: "daily_summary_optimized".to_string(),
                name: "日报汇总（优化版）".to_string(),
                description: "经过优化的日报汇总模板，提供更智能的分析归纳和结构化输出".to_string(),
                template_type: AIAnalysisTemplate::DailySummary {
                    include_tech_analysis: true,
                    include_risk_assessment: true,
                },
                template_content: include_str!("../../templates/daily_summary_optimized.hbs")
                    .to_string(),
                variables: vec![
                    "analyses".to_string(),
                    "start_date".to_string(),
                    "end_date".to_string(),
                    "total_commits".to_string(),
                    "contributors".to_string(),
                    "tech_stack".to_string(),
                    "risk_factors".to_string(),
                ],
                version: "2.0.0".to_string(),
            },
        );

        // 日报汇总模板 - 执行摘要版
        self.templates.insert(
            "daily_summary_executive".to_string(),
            PromptTemplate {
                id: "daily_summary_executive".to_string(),
                name: "日报汇总（执行摘要版）".to_string(),
                description: "为管理层和技术决策者准备的简洁执行摘要".to_string(),
                template_type: AIAnalysisTemplate::DailySummary {
                    include_tech_analysis: true,
                    include_risk_assessment: true,
                },
                template_content: include_str!("../../templates/daily_summary_executive.hbs")
                    .to_string(),
                variables: vec![
                    "analyses".to_string(),
                    "start_date".to_string(),
                    "end_date".to_string(),
                    "total_commits".to_string(),
                    "contributors".to_string(),
                    "tech_stack".to_string(),
                    "risk_factors".to_string(),
                ],
                version: "2.0.0".to_string(),
            },
        );
    }

    /// 加载自定义模板
    fn load_custom_templates(&mut self) {
        let custom_dir = self.cache_dir.join("custom_templates");

        if !custom_dir.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(&custom_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(template) = serde_json::from_str::<PromptTemplate>(&content) {
                            // 只更新已存在模板的内容，不添加新模板
                            if self.templates.contains_key(&template.id) {
                                if let Some(existing_template) =
                                    self.templates.get_mut(&template.id)
                                {
                                    existing_template.template_content = template.template_content;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 获取所有模板
    pub fn get_all_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }

    /// 根据ID获取模板
    pub fn get_template(&self, template_id: &str) -> Option<&PromptTemplate> {
        self.templates.get(template_id)
    }

    /// 根据模板类型获取模板
    pub fn get_templates_by_type(
        &self,
        template_type: &AIAnalysisTemplate,
    ) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|t| match (&t.template_type, template_type) {
                (
                    AIAnalysisTemplate::CommitAnalysis {
                        depth: d1,
                        include_code_review: c1,
                    },
                    AIAnalysisTemplate::CommitAnalysis {
                        depth: d2,
                        include_code_review: c2,
                    },
                ) => d1 == d2 && c1 == c2,
                (
                    AIAnalysisTemplate::DailySummary {
                        include_tech_analysis: t1,
                        include_risk_assessment: r1,
                    },
                    AIAnalysisTemplate::DailySummary {
                        include_tech_analysis: t2,
                        include_risk_assessment: r2,
                    },
                ) => t1 == t2 && r1 == r2,
                _ => false,
            })
            .collect()
    }

    /// 添加自定义模板
    pub fn add_template(&mut self, template: PromptTemplate) -> Result<(), String> {
        if self.templates.contains_key(&template.id) {
            return Err("模板ID已存在".to_string());
        }
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// 更新模板
    pub fn update_template(
        &mut self,
        template_id: &str,
        template: PromptTemplate,
    ) -> Result<(), String> {
        if !self.templates.contains_key(template_id) {
            return Err("模板不存在".to_string());
        }
        self.templates.insert(template_id.to_string(), template);
        Ok(())
    }

    /// 删除模板
    pub fn delete_template(&mut self, template_id: &str) -> Result<(), String> {
        if !self.templates.contains_key(template_id) {
            return Err("模板不存在".to_string());
        }
        self.templates.remove(template_id);
        Ok(())
    }

    /// 渲染模板
    pub fn render_template(
        &self,
        template_id: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, String> {
        let template = self.get_template(template_id).ok_or("模板不存在")?;

        // 使用handlebars渲染模板
        let mut handlebars = handlebars::Handlebars::new();
        handlebars
            .register_template_string("template", &template.template_content)
            .map_err(|e| format!("模板解析失败: {}", e))?;

        handlebars
            .render("template", context)
            .map_err(|e| format!("模板渲染失败: {}", e))
    }

    /// 获取提交分析的提示词
    pub fn get_commit_analysis_prompt(
        &self,
        analysis: &CommitDetailAnalysis,
        depth: AnalysisDepth,
        include_code_review: bool,
        diff_content: &str,
    ) -> Result<String, String> {
        let template_id = match (depth, include_code_review) {
            (AnalysisDepth::Simple, _) => "commit_simple",
            (AnalysisDepth::Detailed, true) => "commit_detailed",
            (AnalysisDepth::Deep, true) => "commit_deep",
            (AnalysisDepth::Detailed, false) => "commit_simple",
            (AnalysisDepth::Deep, false) => "commit_detailed",
        };

        let mut context = HashMap::new();
        context.insert("commit_id".to_string(), analysis.commit_id.clone());
        context.insert(
            "author".to_string(),
            format!("{} <{}>", analysis.author, analysis.email),
        );
        context.insert(
            "timestamp".to_string(),
            chrono::DateTime::from_timestamp(analysis.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        );
        context.insert("message".to_string(), analysis.message.clone());

        // 格式化文件变更信息
        let files_changed = analysis
            .files_changed
            .iter()
            .map(|f| {
                format!(
                    "{} ({})",
                    f.file_path,
                    self.format_change_type(&f.change_type)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        context.insert("files_changed".to_string(), files_changed);

        context.insert("diff_content".to_string(), diff_content.to_string());

        // 添加代码语言信息
        let code_languages: Vec<String> = analysis
            .files_changed
            .iter()
            .filter_map(|f| f.language.clone())
            .collect();
        context.insert("code_language".to_string(), code_languages.join(", "));

        self.render_template(template_id, &context)
    }

    /// 获取日报汇总的提示词
    pub fn get_daily_summary_prompt(
        &self,
        analyses: &[CommitDetailAnalysis],
        start_date: &str,
        end_date: &str,
        include_tech_analysis: bool,
        include_risk_assessment: bool,
    ) -> Result<String, String> {
        self.get_daily_summary_prompt_with_template(
            analyses,
            start_date,
            end_date,
            include_tech_analysis,
            include_risk_assessment,
            "daily_summary_enhanced",
        )
    }

    /// 使用指定模板获取日报汇总的提示词
    pub fn get_daily_summary_prompt_with_template(
        &self,
        analyses: &[CommitDetailAnalysis],
        start_date: &str,
        end_date: &str,
        include_tech_analysis: bool,
        include_risk_assessment: bool,
        template_id: &str,
    ) -> Result<String, String> {
        // 如果用户指定了有效的模板ID，使用指定的模板
        // 否则根据参数自动选择
        let template_id_to_use = if self.templates.contains_key(template_id) {
            template_id
        } else {
            match (include_tech_analysis, include_risk_assessment) {
                (false, false) => "daily_summary_basic",
                (true, false) | (false, true) => "daily_summary_enhanced",
                (true, true) => "daily_summary_optimized",
            }
        };

        let mut context = HashMap::new();
        context.insert("start_date".to_string(), start_date.to_string());
        context.insert("end_date".to_string(), end_date.to_string());
        context.insert("total_commits".to_string(), analyses.len().to_string());

        // 序列化分析结果
        let analyses_json =
            serde_json::to_string(analyses).map_err(|e| format!("序列化分析结果失败: {}", e))?;
        context.insert("analyses".to_string(), analyses_json);

        // 提取贡献者
        let contributors: Vec<String> = analyses
            .iter()
            .map(|a| format!("{} <{}>", a.author, a.email))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        context.insert("contributors".to_string(), contributors.join(", "));

        // 技术栈分析
        let tech_stack = self.analyze_tech_stack(analyses);
        context.insert("tech_stack".to_string(), tech_stack);

        // 风险因素
        let risk_factors = self.analyze_risk_factors(analyses);
        context.insert("risk_factors".to_string(), risk_factors);

        self.render_template(template_id_to_use, &context)
    }

    /// 格式化变更类型
    fn format_change_type(&self, change_type: &crate::types::git_types::FileChangeType) -> String {
        use crate::types::git_types::FileChangeType;
        match change_type {
            FileChangeType::Added => "新增".to_string(),
            FileChangeType::Modified => "修改".to_string(),
            FileChangeType::Deleted => "删除".to_string(),
            FileChangeType::Renamed => "重命名".to_string(),
            FileChangeType::Copied => "复制".to_string(),
        }
    }

    /// 分析技术栈
    fn analyze_tech_stack(&self, analyses: &[CommitDetailAnalysis]) -> String {
        let mut languages = std::collections::HashMap::new();
        let mut frameworks = std::collections::HashMap::new();

        for analysis in analyses {
            for file in &analysis.files_changed {
                // 统计编程语言
                if let Some(lang) = &file.language {
                    *languages.entry(lang.clone()).or_insert(0) += 1;
                }

                // 检测框架（简单实现）
                let path = &file.file_path.to_lowercase();
                if path.contains("vue") || path.contains("react") || path.contains("angular") {
                    *frameworks.entry("前端框架".to_string()).or_insert(0) += 1;
                } else if path.contains("spring")
                    || path.contains("express")
                    || path.contains("django")
                {
                    *frameworks.entry("后端框架".to_string()).or_insert(0) += 1;
                } else if path.contains("sql") || path.contains("mongodb") || path.contains("redis")
                {
                    *frameworks.entry("数据库相关".to_string()).or_insert(0) += 1;
                }
            }
        }

        let mut result = String::new();

        if !languages.is_empty() {
            result.push_str("主要编程语言：");
            let mut langs: Vec<_> = languages.into_iter().collect();
            langs.sort_by(|a, b| b.1.cmp(&a.1));
            for (lang, count) in langs {
                result.push_str(&format!("{}({}) ", lang, count));
            }
            result.push_str("\n");
        }

        if !frameworks.is_empty() {
            result.push_str("涉及技术栈：");
            for (framework, count) in frameworks {
                result.push_str(&format!("{}({}) ", framework, count));
            }
        }

        result
    }

    /// 分析风险因素
    fn analyze_risk_factors(&self, analyses: &[CommitDetailAnalysis]) -> String {
        let mut risks = Vec::new();

        for analysis in analyses {
            // 检查高影响提交
            if analysis.impact_level == crate::types::git_types::ImpactLevel::Critical {
                risks.push(format!("发现关键变更：{}", analysis.commit_id));
            }

            // 检查大量代码变更
            if analysis.insertions > 1000 || analysis.deletions > 1000 {
                risks.push(format!(
                    "大量代码变更：+{} / -{}",
                    analysis.insertions, analysis.deletions
                ));
            }

            // 检查配置文件变更
            for file in &analysis.files_changed {
                if file.file_path.contains("config") || file.file_path.contains(".env") {
                    risks.push(format!("配置文件变更：{}", file.file_path));
                }
            }
        }

        if risks.is_empty() {
            "未发现明显风险因素".to_string()
        } else {
            risks.join("\n")
        }
    }

    /// 更新模板内容
    pub fn update_template_content(
        &mut self,
        template_id: &str,
        template_content: &str,
    ) -> Result<(), String> {
        if let Some(template) = self.templates.get_mut(template_id) {
            template.template_content = template_content.to_string();
            // 保存到文件
            self.save_templates_to_file()?;
            Ok(())
        } else {
            Err("模板不存在".to_string())
        }
    }

    /// 重置模板为默认内容
    pub fn reset_template(&mut self, template_id: &str) -> Result<(), String> {
        // 获取默认模板内容
        let default_content = match template_id {
            "commit_simple" => include_str!("../templates/commit_analysis_simple.hbs"),
            "commit_detailed" => include_str!("../templates/commit_analysis_detailed.hbs"),
            "commit_deep" => include_str!("../templates/commit_analysis_deep.hbs"),
            "daily_summary_basic" => include_str!("../templates/daily_summary_basic.hbs"),
            "daily_summary_enhanced" => include_str!("../templates/daily_summary_enhanced.hbs"),
            "daily_summary_optimized" => {
                include_str!("../../templates/daily_summary_optimized.hbs")
            }
            "daily_summary_executive" => {
                include_str!("../../templates/daily_summary_executive.hbs")
            }
            _ => return Err("未知的模板ID".to_string()),
        };

        if let Some(template) = self.templates.get_mut(template_id) {
            template.template_content = default_content.to_string();
            // 保存到文件
            self.save_templates_to_file()?;
            Ok(())
        } else {
            Err("模板不存在".to_string())
        }
    }

    /// 保存模板到文件
    fn save_templates_to_file(&self) -> Result<(), String> {
        // 创建自定义模板目录
        let custom_dir = self.cache_dir.join("custom_templates");
        std::fs::create_dir_all(&custom_dir)
            .map_err(|e| format!("Failed to create custom templates directory: {}", e))?;

        // 保存每个自定义模板
        for (id, template) in &self.templates {
            let template_file = custom_dir.join(format!("{}.json", id));
            let content = serde_json::to_string_pretty(template)
                .map_err(|e| format!("Failed to serialize template: {}", e))?;
            std::fs::write(&template_file, content)
                .map_err(|e| format!("Failed to write template file: {}", e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git_types::*;

    #[test]
    fn test_template_manager_creation() {
        let manager = PromptTemplateManager::new();
        assert!(manager.get_all_templates().len() > 0);
    }

    #[test]
    fn test_get_template_by_id() {
        let manager = PromptTemplateManager::new();
        let template = manager.get_template("commit_simple");
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "单体提交分析（简单版）");
    }

    #[test]
    fn test_tech_stack_analysis() {
        let manager = PromptTemplateManager::new();
        let analysis = CommitDetailAnalysis {
            commit_id: "test".to_string(),
            repo_path: "/test".to_string(),
            author: "Test".to_string(),
            email: "test@example.com".to_string(),
            timestamp: 0,
            message: "Test commit".to_string(),
            files_changed: vec![
                CommitFileChange {
                    file_path: "src/main.ts".to_string(),
                    change_type: FileChangeType::Modified,
                    insertions: 10,
                    deletions: 5,
                    is_binary: false,
                    language: Some("TypeScript".to_string()),
                },
                CommitFileChange {
                    file_path: "src/App.vue".to_string(),
                    change_type: FileChangeType::Added,
                    insertions: 20,
                    deletions: 0,
                    is_binary: false,
                    language: Some("Vue".to_string()),
                },
            ],
            insertions: 30,
            deletions: 5,
            summary: "Test".to_string(),
            impact_level: ImpactLevel::Low,
            tags: vec!["frontend".to_string()],
        };

        let tech_stack = manager.analyze_tech_stack(&[analysis]);
        assert!(tech_stack.contains("TypeScript"));
        assert!(tech_stack.contains("Vue"));
        assert!(tech_stack.contains("前端框架"));
    }
}
