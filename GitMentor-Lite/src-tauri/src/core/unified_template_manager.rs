use crate::core::prompt_manager::PromptTemplate;
use crate::core::versioned_template_manager::*;
use crate::types::template_types::*;
use anyhow::Result;
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 统一模板管理器
///
/// 整合了 PromptManager 和 VersionedTemplateManager 的功能
/// 提供统一的模板版本管理接口
pub struct UnifiedTemplateManager {
    /// 版本化模板管理器（用于日报和AI分析模板）
    versioned_manager: VersionedTemplateManager,
    /// 提交模板缓存（兼容旧的 PromptTemplate 接口）
    commit_templates: HashMap<String, PromptTemplate>,
    /// 配置路径
    config_path: PathBuf,
    /// 当前系统版本
    current_version: String,
}

/// 统一模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTemplateConfig {
    pub version: String,
    pub last_updated: String,
    pub commit_templates: HashMap<String, PromptTemplate>,
    pub versioned_templates_migration: Option<bool>, // 是否已迁移到版本化模板
}

impl Default for UnifiedTemplateConfig {
    fn default() -> Self {
        Self {
            version: "2.0.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            commit_templates: HashMap::new(),
            versioned_templates_migration: Some(false),
        }
    }
}

impl UnifiedTemplateManager {
    /// 创建新的统一模板管理器
    pub fn new(base_dir: &Path) -> Result<Self> {
        let templates_dir = base_dir.join(".gitmentor").join("templates");
        let config_path = base_dir.join(".gitmentor").join("unified_templates.json");

        // 创建目录
        fs::create_dir_all(&templates_dir)?;

        let versioned_manager = VersionedTemplateManager::new(base_dir)?;
        let current_version = "2.0.0".to_string();

        let mut manager = Self {
            versioned_manager,
            commit_templates: HashMap::new(),
            config_path,
            current_version,
        };

        // 加载或初始化提交模板
        manager.load_commit_templates()?;

        // 确保系统模板存在
        manager.ensure_system_templates()?;

        Ok(manager)
    }

    /// 加载提交模板配置
    fn load_commit_templates(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let config: UnifiedTemplateConfig = serde_json::from_str(&content)?;

            // 检查版本
            if config.version != self.current_version {
                // 版本升级，需要迁移
                self.migrate_commit_templates(&config)?;
            } else {
                self.commit_templates = config.commit_templates;
            }
        } else {
            // 首次创建，初始化默认提交模板
            self.initialize_default_commit_templates()?;
            self.save_commit_templates()?;
        }

        Ok(())
    }

    /// 保存提交模板配置
    fn save_commit_templates(&self) -> Result<()> {
        let config = UnifiedTemplateConfig {
            version: self.current_version.clone(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            commit_templates: self.commit_templates.clone(),
            versioned_templates_migration: Some(true),
        };

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// 初始化默认提交模板（从版本化模板管理器加载）
    fn initialize_default_commit_templates(&mut self) -> Result<()> {
        let commit_template_ids = vec![
            "commit_standard",
            "commit_chinese",
            "commit_detailed",
            "commit_conventional",
        ];

        for template_id in commit_template_ids {
            if let Some(content) = self
                .versioned_manager
                .get_template_content(template_id)
                .ok()
            {
                let prompt_template = self.convert_to_prompt_template(template_id, &content)?;
                self.commit_templates
                    .insert(template_id.to_string(), prompt_template);
            }
        }

        Ok(())
    }

    /// 将版本化模板转换为 PromptTemplate
    fn convert_to_prompt_template(
        &self,
        template_id: &str,
        content: &str,
    ) -> Result<PromptTemplate> {
        let (name, description, version) = match template_id {
            "commit_standard" => ("标准提交消息", "生成符合常规规范的提交消息", "1.0.0"),
            "commit_chinese" => ("简洁提交消息", "生成简洁明了的中文提交消息", "1.0.0"),
            "commit_detailed" => ("详细提交消息", "生成包含详细描述的提交消息", "1.0.0"),
            "commit_conventional" => ("约定式提交", "生成符合约定式提交规范的消息", "1.0.0"),
            _ => ("未知模板", "未知模板描述", "1.0.0"),
        };

        Ok(PromptTemplate {
            id: template_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            system_prompt: content.to_string(),
            user_prompt_template: String::new(), // 使用单文件模板
            file_analysis_system_prompt: String::new(),
            file_analysis_user_prompt: String::new(),
            summary_system_prompt: String::new(),
            summary_user_prompt: String::new(),
            language: "FOLLOW_GLOBAL".to_string(),
            max_tokens: Some(2000),
            temperature: Some(0.3),
            enable_emoji: Some(template_id == "commit_conventional"),
            enable_body: Some(template_id != "commit_conventional"),
            enable_merge_commit: Some(false),
            use_recent_commits: Some(template_id == "commit_detailed"),
            commit_types: Some(self.get_default_commit_types()),
            is_custom: Some(false),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            version: Some(version.to_string()),
            template_hash: None,
        })
    }

    /// 获取默认的提交类型配置
    fn get_default_commit_types(&self) -> Vec<crate::core::prompt_manager::CommitType> {
        vec![
            crate::core::prompt_manager::CommitType {
                name: "feat".to_string(),
                emoji: Some("✨".to_string()),
                description: "New feature".to_string(),
                example_scopes: vec![
                    "user".to_string(),
                    "payment".to_string(),
                    "auth".to_string(),
                ],
            },
            crate::core::prompt_manager::CommitType {
                name: "fix".to_string(),
                emoji: Some("🐛".to_string()),
                description: "Bug fix".to_string(),
                example_scopes: vec!["auth".to_string(), "data".to_string(), "ui".to_string()],
            },
            crate::core::prompt_manager::CommitType {
                name: "docs".to_string(),
                emoji: Some("📝".to_string()),
                description: "Documentation change".to_string(),
                example_scopes: vec!["README".to_string(), "API".to_string(), "guide".to_string()],
            },
            crate::core::prompt_manager::CommitType {
                name: "style".to_string(),
                emoji: Some("💄".to_string()),
                description: "Code format change".to_string(),
                example_scopes: vec!["formatting".to_string(), "lint".to_string()],
            },
            crate::core::prompt_manager::CommitType {
                name: "refactor".to_string(),
                emoji: Some("♻️".to_string()),
                description: "Code refactoring".to_string(),
                example_scopes: vec![
                    "utils".to_string(),
                    "helpers".to_string(),
                    "core".to_string(),
                ],
            },
            crate::core::prompt_manager::CommitType {
                name: "perf".to_string(),
                emoji: Some("⚡️".to_string()),
                description: "Performance optimization".to_string(),
                example_scopes: vec![
                    "query".to_string(),
                    "cache".to_string(),
                    "render".to_string(),
                ],
            },
            crate::core::prompt_manager::CommitType {
                name: "test".to_string(),
                emoji: Some("✅".to_string()),
                description: "Test related".to_string(),
                example_scopes: vec![
                    "unit".to_string(),
                    "e2e".to_string(),
                    "integration".to_string(),
                ],
            },
            crate::core::prompt_manager::CommitType {
                name: "build".to_string(),
                emoji: Some("📦️".to_string()),
                description: "Build system".to_string(),
                example_scopes: vec![
                    "webpack".to_string(),
                    "npm".to_string(),
                    "docker".to_string(),
                ],
            },
            crate::core::prompt_manager::CommitType {
                name: "ci".to_string(),
                emoji: Some("👷".to_string()),
                description: "CI configuration".to_string(),
                example_scopes: vec![
                    "travis".to_string(),
                    "jenkins".to_string(),
                    "github".to_string(),
                ],
            },
            crate::core::prompt_manager::CommitType {
                name: "chore".to_string(),
                emoji: Some("🔧".to_string()),
                description: "Other changes".to_string(),
                example_scopes: vec![
                    "scripts".to_string(),
                    "config".to_string(),
                    "deps".to_string(),
                ],
            },
        ]
    }

    /// 确保系统模板存在
    fn ensure_system_templates(&mut self) -> Result<()> {
        // 这个方法在 versioned_manager 初始化时已经调用
        // 这里主要是确保提交模板也正确加载
        if self.commit_templates.is_empty() {
            self.initialize_default_commit_templates()?;
            self.save_commit_templates()?;
        }
        Ok(())
    }

    /// 迁移旧的提交模板配置
    fn migrate_commit_templates(&mut self, old_config: &UnifiedTemplateConfig) -> Result<()> {
        // 保留用户自定义的模板
        self.commit_templates = old_config.commit_templates.clone();

        // 检查是否有新的系统模板需要添加
        self.initialize_default_commit_templates()?;

        // 保存新配置
        self.save_commit_templates()?;

        Ok(())
    }

    // ==================== 提交模板管理接口 ====================

    /// 获取提交模板
    pub fn get_commit_template(&self, template_id: &str) -> Option<&PromptTemplate> {
        self.commit_templates.get(template_id)
    }

    /// 获取所有提交模板
    pub fn get_all_commit_templates(&self) -> Vec<&PromptTemplate> {
        self.commit_templates.values().collect()
    }

    /// 更新提交模板并创建版本
    pub fn update_commit_template(
        &mut self,
        template_id: &str,
        content: String,
        version_name: String,
        version_description: String,
    ) -> Result<String> {
        // 更新版本化模板管理器
        let version_id = self.versioned_manager.update_template(
            template_id,
            content.clone(),
            version_name,
            version_description,
        )?;

        // 更新提交模板缓存
        if let Some(template) = self.commit_templates.get_mut(template_id) {
            template.system_prompt = content;
            template.updated_at = Some(chrono::Utc::now().to_rfc3339());
            template.is_custom = Some(true);
        }

        // 保存配置
        self.save_commit_templates()?;

        Ok(version_id)
    }

    /// 切换提交模板版本
    pub fn switch_commit_template_version(
        &mut self,
        template_id: &str,
        version_id: &str,
    ) -> Result<()> {
        // 切换版本化模板管理器中的版本
        self.versioned_manager
            .switch_template_version(template_id, version_id)?;

        // 获取新版本内容并更新缓存
        if let Some(content) = self
            .versioned_manager
            .get_template_content(template_id)
            .ok()
        {
            if let Some(template) = self.commit_templates.get_mut(template_id) {
                template.system_prompt = content;
                template.updated_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        // 保存配置
        self.save_commit_templates()?;

        Ok(())
    }

    /// 获取提交模板版本历史
    pub fn get_commit_template_versions(&self, template_id: &str) -> Result<Vec<&TemplateVersion>> {
        self.versioned_manager.get_template_versions(template_id)
    }

    /// 检查提交模板更新
    pub fn check_commit_template_updates(&self) -> Vec<TemplateSystemUpdate> {
        let commit_template_ids = vec![
            "commit_standard",
            "commit_chinese",
            "commit_detailed",
            "commit_conventional",
        ];

        let mut updates = Vec::new();

        for template_id in commit_template_ids {
            // 检查版本化模板管理器中的更新
            let all_updates = self.versioned_manager.get_system_template_updates();
            for update in all_updates {
                if update.system_template_id == template_id {
                    updates.push(update);
                }
            }
        }

        updates
    }

    /// 应用提交模板更新
    pub fn apply_commit_template_update(&mut self, template_id: &str) -> Result<()> {
        // 应用版本化模板管理器的更新
        self.versioned_manager
            .apply_system_template_update(template_id)?;

        // 更新提交模板缓存
        if let Some(content) = self
            .versioned_manager
            .get_template_content(template_id)
            .ok()
        {
            if let Some(template) = self.commit_templates.get_mut(template_id) {
                template.system_prompt = content;
                template.updated_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        // 保存配置
        self.save_commit_templates()?;

        Ok(())
    }

    // ==================== 版本化模板管理接口（委托） ====================

    /// 获取所有版本化模板
    pub fn get_all_versioned_templates(&self) -> Vec<&TemplateConfigWithVersions> {
        self.versioned_manager.get_all_templates()
    }

    /// 获取版本化模板
    pub fn get_versioned_template(&self, template_id: &str) -> Option<&TemplateConfigWithVersions> {
        self.versioned_manager.get_template(template_id)
    }

    /// 更新版本化模板
    pub fn update_versioned_template(
        &mut self,
        template_id: &str,
        content: String,
        version_name: String,
        version_description: String,
    ) -> Result<String> {
        self.versioned_manager.update_template(
            template_id,
            content,
            version_name,
            version_description,
        )
    }

    /// 切换版本化模板版本
    pub fn switch_versioned_template_version(
        &mut self,
        template_id: &str,
        version_id: &str,
    ) -> Result<()> {
        self.versioned_manager
            .switch_template_version(template_id, version_id)
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
        self.versioned_manager.create_custom_template(
            name,
            description,
            template_type,
            content,
            base_template_id,
        )
    }

    /// 删除自定义模板
    pub fn delete_custom_template(&mut self, template_id: &str) -> Result<()> {
        self.versioned_manager.delete_custom_template(template_id)
    }

    /// 检查系统模板更新
    pub fn check_system_template_updates(&self) -> Vec<TemplateSystemUpdate> {
        self.versioned_manager.get_system_template_updates()
    }

    /// 应用系统模板更新
    pub fn apply_system_template_update(&mut self, template_id: &str) -> Result<()> {
        self.versioned_manager
            .apply_system_template_update(template_id)
    }
}
