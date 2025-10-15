use crate::types::template_types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};

/// 版本化模板管理器
///
/// 功能：
/// 1. 管理模板的多个版本
/// 2. 支持用户编辑和保存新版本
/// 3. 支持版本切换
/// 4. 支持系统模板更新
pub struct VersionedTemplateManager {
    /// 模板存储目录
    templates_dir: PathBuf,
    /// 模板配置缓存
    template_cache: HashMap<String, TemplateConfigWithVersions>,
    /// 系统内置模板定义
    builtin_templates: HashMap<String, SystemTemplateDefinition>,
}

/// 系统模板定义
#[derive(Debug, Clone)]
pub struct SystemTemplateDefinition {
    /// 模板ID
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 模板类型
    pub template_type: String,
    /// 当前版本内容
    pub current_content: String,
    /// 当前版本号
    pub current_version: String,
    /// 版本历史
    pub version_history: Vec<SystemTemplateVersion>,
}

/// 系统模板版本
#[derive(Debug, Clone)]
pub struct SystemTemplateVersion {
    /// 版本号
    pub version: String,
    /// 版本名称
    pub name: String,
    /// 版本描述
    pub description: String,
    /// 模板内容
    pub content: String,
    /// 发布时间
    pub release_date: String,
}

impl VersionedTemplateManager {
    /// 创建新的版本化模板管理器
    pub fn new(base_dir: &Path) -> Result<Self> {
        let templates_dir = base_dir.join(".gitmentor").join("templates");
        fs::create_dir_all(&templates_dir)?;

        let mut manager = Self {
            templates_dir,
            template_cache: HashMap::new(),
            builtin_templates: Self::initialize_builtin_templates(),
        };

        // 加载现有模板
        manager.load_all_templates()?;

        // 确保系统模板存在
        manager.ensure_builtin_templates()?;

        Ok(manager)
    }

    /// 初始化系统内置模板定义
    fn initialize_builtin_templates() -> HashMap<String, SystemTemplateDefinition> {
        let mut templates = HashMap::new();

        // 日报模板 - 基础版
        templates.insert(
            "daily_summary_basic".to_string(),
            SystemTemplateDefinition {
                id: "daily_summary_basic".to_string(),
                name: "基础日报模板".to_string(),
                description: "提供基本的Git提交统计和汇总".to_string(),
                template_type: "daily_summary".to_string(),
                current_content: include_str!("../../templates/daily_summary_basic.hbs")
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "基础日报模板初始版本".to_string(),
                    content: include_str!("../../templates/daily_summary_basic.hbs").to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // 日报模板 - 增强版
        templates.insert(
            "daily_summary_enhanced".to_string(),
            SystemTemplateDefinition {
                id: "daily_summary_enhanced".to_string(),
                name: "增强日报模板".to_string(),
                description: "包含技术分析和风险评估的详细日报".to_string(),
                template_type: "daily_summary".to_string(),
                current_content: include_str!("../../templates/daily_summary_enhanced.hbs")
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "增强日报模板初始版本".to_string(),
                    content: include_str!("../../templates/daily_summary_enhanced.hbs").to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // 日报模板 - 优化版
        templates.insert(
            "daily_summary_optimized".to_string(),
            SystemTemplateDefinition {
                id: "daily_summary_optimized".to_string(),
                name: "优化日报模板".to_string(),
                description: "智能归纳和结构化输出的优化日报".to_string(),
                template_type: "daily_summary".to_string(),
                current_content: include_str!("../../templates/daily_summary_optimized.hbs")
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "优化日报模板初始版本".to_string(),
                    content: include_str!("../../templates/daily_summary_optimized.hbs")
                        .to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // 日报模板 - 执行摘要版
        templates.insert(
            "daily_summary_executive".to_string(),
            SystemTemplateDefinition {
                id: "daily_summary_executive".to_string(),
                name: "执行摘要模板".to_string(),
                description: "为管理层准备的简洁报告".to_string(),
                template_type: "daily_summary".to_string(),
                current_content: include_str!("../../templates/daily_summary_executive.hbs")
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "执行摘要模板初始版本".to_string(),
                    content: include_str!("../../templates/daily_summary_executive.hbs")
                        .to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // AI分析模板 - 提交分析
        templates.insert(
            "commit_analysis".to_string(),
            SystemTemplateDefinition {
                id: "commit_analysis".to_string(),
                name: "提交分析模板".to_string(),
                description: "用于分析Git提交内容的AI模板".to_string(),
                template_type: "commit_analysis".to_string(),
                current_content: r#"请分析以下Git提交：

提交信息：{{commit.message}}
作者：{{commit.author}}
时间：{{commit.timestamp}}

变更文件：
{{#each files}}
- {{this.path}} ({{this.type}})
{{/each}}

请从技术角度分析这次提交：
1. 主要变更内容
2. 技术实现质量
3. 潜在问题和建议
4. 相关性分析（与最近其他提交的关系）"#
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "提交分析模板初始版本".to_string(),
                    content: r#"请分析以下Git提交：

提交信息：{{commit.message}}
作者：{{commit.author}}
时间：{{commit.timestamp}}

变更文件：
{{#each files}}
- {{this.path}} ({{this.type}})
{{/each}}

请从技术角度分析这次提交：
1. 主要变更内容
2. 技术实现质量
3. 潜在问题和建议
4. 相关性分析（与最近其他提交的关系）"#
                        .to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // AI分析模板 - 代码审查
        templates.insert(
            "code_review".to_string(),
            SystemTemplateDefinition {
                id: "code_review".to_string(),
                name: "代码审查模板".to_string(),
                description: "用于自动化代码审查的AI模板".to_string(),
                template_type: "code_review".to_string(),
                current_content: r#"请对以下代码进行审查：

文件路径：{{file.path}}
代码内容：
```
{{file.content}}
```

请从以下角度进行审查：
1. 代码质量和规范性
2. 潜在的bug和安全问题
3. 性能优化建议
4. 代码可维护性
5. 最佳实践遵循情况

请提供具体的改进建议。"#
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "代码审查模板初始版本".to_string(),
                    content: r#"请对以下代码进行审查：

文件路径：{{file.path}}
代码内容：
```
{{file.content}}
```

请从以下角度进行审查：
1. 代码质量和规范性
2. 潜在的bug和安全问题
3. 性能优化建议
4. 代码可维护性
5. 最佳实践遵循情况

请提供具体的改进建议。"#
                        .to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // AI分析模板 - 技术分析
        templates.insert(
            "tech_analysis".to_string(),
            SystemTemplateDefinition {
                id: "tech_analysis".to_string(),
                name: "技术分析模板".to_string(),
                description: "用于深入技术分析的AI模板".to_string(),
                template_type: "tech_analysis".to_string(),
                current_content: r#"请对以下代码/项目进行技术分析：

分析目标：{{target}}
上下文信息：
{{context}}

技术要求：
{{requirements}}

请提供详细的技术分析报告：
1. 架构设计分析
2. 技术栈评估
3. 性能和扩展性分析
4. 技术风险评估
5. 改进建议和最佳实践"#
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "技术分析模板初始版本".to_string(),
                    content: r#"请对以下代码/项目进行技术分析：

分析目标：{{target}}
上下文信息：
{{context}}

技术要求：
{{requirements}}

请提供详细的技术分析报告：
1. 架构设计分析
2. 技术栈评估
3. 性能和扩展性分析
4. 技术风险评估
5. 改进建议和最佳实践"#
                        .to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // 提交消息模板 - 标准版
        templates.insert(
            "commit_standard".to_string(),
            SystemTemplateDefinition {
                id: "commit_standard".to_string(),
                name: "标准提交消息".to_string(),
                description: "生成符合常规规范的提交消息".to_string(),
                template_type: "commit_message".to_string(),
                current_content: include_str!("../../templates/commit_standard.hbs").to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "标准提交消息模板初始版本".to_string(),
                    content: include_str!("../../templates/commit_standard.hbs").to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // 提交消息模板 - 简洁版
        templates.insert(
            "commit_chinese".to_string(),
            SystemTemplateDefinition {
                id: "commit_chinese".to_string(),
                name: "简洁提交消息".to_string(),
                description: "生成简洁明了的中文提交消息".to_string(),
                template_type: "commit_message".to_string(),
                current_content: include_str!("../../templates/commit_chinese.hbs").to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "简洁提交消息模板初始版本".to_string(),
                    content: include_str!("../../templates/commit_chinese.hbs").to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // 提交消息模板 - 详细版
        templates.insert(
            "commit_detailed".to_string(),
            SystemTemplateDefinition {
                id: "commit_detailed".to_string(),
                name: "详细提交消息".to_string(),
                description: "生成包含详细描述的提交消息".to_string(),
                template_type: "commit_message".to_string(),
                current_content: include_str!("../../templates/commit_detailed.hbs").to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "详细提交消息模板初始版本".to_string(),
                    content: include_str!("../../templates/commit_detailed.hbs").to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        // 提交消息模板 - 约定式提交
        templates.insert(
            "commit_conventional".to_string(),
            SystemTemplateDefinition {
                id: "commit_conventional".to_string(),
                name: "约定式提交".to_string(),
                description: "生成符合约定式提交规范的消息".to_string(),
                template_type: "commit_message".to_string(),
                current_content: include_str!("../../templates/commit_conventional.hbs")
                    .to_string(),
                current_version: "1.0.0".to_string(),
                version_history: vec![SystemTemplateVersion {
                    version: "1.0.0".to_string(),
                    name: "初始版本".to_string(),
                    description: "约定式提交模板初始版本".to_string(),
                    content: include_str!("../../templates/commit_conventional.hbs").to_string(),
                    release_date: "2024-01-01T00:00:00Z".to_string(),
                }],
            },
        );

        templates
    }

    /// 加载所有模板
    fn load_all_templates(&mut self) -> Result<()> {
        if !self.templates_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(template) =
                        serde_json::from_str::<TemplateConfigWithVersions>(&content)
                    {
                        self.template_cache.insert(template.id.clone(), template);
                    }
                }
            }
        }

        Ok(())
    }

    /// 确保系统模板存在
    fn ensure_builtin_templates(&mut self) -> Result<()> {
        self.normalize_builtin_templates()?;

        for (template_id, builtin_def) in &self.builtin_templates {
            if !self.template_cache.contains_key(template_id) {
                let initial_version = TemplateVersion::builtin(
                    builtin_def.current_content.clone(),
                    format!("v{}", builtin_def.current_version),
                    "系统内置版本".to_string(),
                );

                let mut template_config = TemplateConfigWithVersions::new(
                    builtin_def.name.clone(),
                    builtin_def.description.clone(),
                    builtin_def.template_type.clone(),
                    initial_version,
                );

                template_config.id = template_id.clone();
                template_config.original_template_id = Some(template_id.clone());
                template_config.system_version = Some(builtin_def.current_version.clone());

                self.template_cache
                    .insert(template_id.clone(), template_config);
            }
        }

        self.persist_template_cache()?;

        Ok(())
    }

    fn normalize_builtin_templates(&mut self) -> Result<()> {
        let mut normalized_cache: HashMap<String, TemplateConfigWithVersions> = HashMap::new();
        let mut existing_cache = mem::take(&mut self.template_cache);

        for (_, mut template) in existing_cache.drain() {
            if template.is_custom {
                let id = template.id.clone();
                normalized_cache.insert(id, template);
                continue;
            }

            let builtin_key = if self.builtin_templates.contains_key(&template.id) {
                Some(template.id.clone())
            } else if let Some(ref original_id) = template.original_template_id {
                if self.builtin_templates.contains_key(original_id) {
                    Some(original_id.clone())
                } else {
                    None
                }
            } else {
                self.builtin_templates
                    .iter()
                    .find(|(_, def)| {
                        def.name == template.name && def.template_type == template.template_type
                    })
                    .map(|(id, _)| id.clone())
            };

            if let Some(builtin_id) = builtin_key {
                if let Some(builtin_def) = self.builtin_templates.get(&builtin_id) {
                    template.id = builtin_id.clone();
                    template.original_template_id = Some(builtin_id.clone());
                    template
                        .system_version
                        .get_or_insert(builtin_def.current_version.clone());

                    match normalized_cache.entry(builtin_id.clone()) {
                        Entry::Vacant(entry) => {
                            entry.insert(template);
                        }
                        Entry::Occupied(mut entry) => {
                            if Self::should_replace_existing(entry.get(), &template) {
                                entry.insert(template);
                            }
                        }
                    }
                    continue;
                }
            }

            let id = template.id.clone();
            normalized_cache.insert(id, template);
        }

        self.template_cache = normalized_cache;
        Ok(())
    }

    fn should_replace_existing(
        existing: &TemplateConfigWithVersions,
        candidate: &TemplateConfigWithVersions,
    ) -> bool {
        let existing_time = Self::parse_timestamp(&existing.updated_at);
        let candidate_time = Self::parse_timestamp(&candidate.updated_at);

        match (existing_time, candidate_time) {
            (Some(existing_ts), Some(candidate_ts)) => candidate_ts > existing_ts,
            (None, Some(_)) => true,
            (Some(_), None) => false,
            (None, None) => candidate.versions.len() > existing.versions.len(),
        }
    }

    fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
        chrono::DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    }

    fn persist_template_cache(&self) -> Result<()> {
        fs::create_dir_all(&self.templates_dir)?;

        let mut desired_files: HashSet<String> = HashSet::new();
        for template in self.template_cache.values() {
            desired_files.insert(format!("{}.json", template.id));
            self.save_template(template)?;
        }

        for entry in fs::read_dir(&self.templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                if !desired_files.contains(&file_name) {
                    fs::remove_file(path)?;
                }
            }
        }

        Ok(())
    }

    /// 保存模板配置
    fn save_template(&self, template: &TemplateConfigWithVersions) -> Result<()> {
        let file_path = self.templates_dir.join(format!("{}.json", template.id));
        let content = serde_json::to_string_pretty(template)?;
        fs::write(file_path, content)?;
        Ok(())
    }

    /// 获取所有模板列表
    pub fn get_all_templates(&self) -> Vec<&TemplateConfigWithVersions> {
        self.template_cache.values().collect()
    }

    /// 获取指定模板
    pub fn get_template(&self, template_id: &str) -> Option<&TemplateConfigWithVersions> {
        self.template_cache.get(template_id)
    }

    /// 获取模板的可变引用
    pub fn get_template_mut(
        &mut self,
        template_id: &str,
    ) -> Option<&mut TemplateConfigWithVersions> {
        self.template_cache.get_mut(template_id)
    }

    /// 更新模板内容并创建新版本
    pub fn update_template(
        &mut self,
        template_id: &str,
        content: String,
        version_name: String,
        version_description: String,
    ) -> Result<String> {
        let parent_id = {
            let template = self
                .get_template_mut(template_id)
                .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;
            // 获取当前版本作为父版本
            Some(template.current_version_id.clone())
        };

        let version_id = {
            let template = self
                .get_template_mut(template_id)
                .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;

            // 创建新版本
            let new_version =
                TemplateVersion::custom(content, version_name, version_description, parent_id);

            let version_id = new_version.id.clone();

            // 添加新版本
            template.add_version(new_version)?;

            // 切换到新版本
            template.switch_to_version(&version_id)?;

            // 标记为用户自定义模板
            template.is_custom = true;

            version_id
        };

        // 保存到文件
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;
        self.save_template(template)?;

        Ok(version_id)
    }

    /// 切换模板版本
    pub fn switch_template_version(&mut self, template_id: &str, version_id: &str) -> Result<()> {
        {
            let template = self
                .get_template_mut(template_id)
                .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;
            template.switch_to_version(version_id)?;
        }

        // 保存到文件
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;
        self.save_template(template)?;

        Ok(())
    }

    /// 获取模板的版本历史
    pub fn get_template_versions(&self, template_id: &str) -> Result<Vec<&TemplateVersion>> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;

        Ok(template.get_version_history())
    }

    /// 获取系统模板更新信息
    pub fn get_system_template_updates(&self) -> Vec<TemplateSystemUpdate> {
        let mut updates = Vec::new();

        for (template_id, builtin_def) in &self.builtin_templates {
            if let Some(user_template) = self.get_template(template_id) {
                if let Some(current_version) = user_template.get_current_version() {
                    // 检查是否有更新
                    if current_version.is_builtin
                        && current_version.version != builtin_def.current_version
                    {
                        updates.push(TemplateSystemUpdate {
                            system_template_id: template_id.clone(),
                            new_version: builtin_def.current_version.clone(),
                            update_description: format!(
                                "系统模板已更新到版本 {}",
                                builtin_def.current_version
                            ),
                            update_time: chrono::Utc::now().to_rfc3339(),
                            requires_confirmation: true,
                        });
                    }
                }
            }
        }

        updates
    }

    /// 应用系统模板更新
    pub fn apply_system_template_update(&mut self, template_id: &str) -> Result<()> {
        let (current_content, current_version, version_name) = {
            let builtin_def = self
                .builtin_templates
                .get(template_id)
                .ok_or_else(|| anyhow::anyhow!("系统模板不存在"))?;
            (
                builtin_def.current_content.clone(),
                builtin_def.current_version.clone(),
                format!("系统更新到版本 {}", builtin_def.current_version),
            )
        };

        {
            let template = self
                .get_template_mut(template_id)
                .ok_or_else(|| anyhow::anyhow!("用户模板不存在"))?;

            // 创建新版本
            let new_version = TemplateVersion::builtin(
                current_content,
                format!("v{}", current_version),
                version_name,
            );

            let version_id = new_version.id.clone();

            // 添加新版本
            template.add_version(new_version)?;

            // 切换到新版本
            template.switch_to_version(&version_id)?;
        }

        // 保存到文件
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("用户模板不存在"))?;
        self.save_template(template)?;

        Ok(())
    }

    /// 还原到系统模板的初始版本
    pub fn revert_to_builtin_version(&mut self, template_id: &str) -> Result<()> {
        self.apply_system_template_update(template_id)
    }

    /// 获取模板的当前内容
    pub fn get_template_content(&self, template_id: &str) -> Result<String> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;

        template
            .get_current_content()
            .ok_or_else(|| anyhow::anyhow!("模板内容为空"))
            .map(|s| s.to_string())
    }

    /// 创建自定义模板
    pub fn create_custom_template(
        &mut self,
        name: String,
        description: String,
        template_type: String,
        content: String,
        base_template_id: Option<String>,
    ) -> Result<String> {
        let initial_version = TemplateVersion::custom(
            content,
            "初始版本".to_string(),
            "用户创建的自定义模板".to_string(),
            None,
        );

        let template = TemplateConfigWithVersions::custom(
            name,
            description,
            template_type,
            initial_version,
            base_template_id,
        );

        let template_id = template.id.clone();

        // 保存模板
        self.save_template(&template)?;
        self.template_cache.insert(template_id.clone(), template);

        Ok(template_id)
    }

    /// 删除自定义模板
    pub fn delete_custom_template(&mut self, template_id: &str) -> Result<()> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("模板不存在"))?;

        if !template.is_custom {
            return Err(anyhow::anyhow!("不能删除系统模板"));
        }

        // 删除文件
        let file_path = self.templates_dir.join(format!("{}.json", template_id));
        fs::remove_file(file_path)?;

        // 从缓存中移除
        self.template_cache.remove(template_id);

        Ok(())
    }
}
