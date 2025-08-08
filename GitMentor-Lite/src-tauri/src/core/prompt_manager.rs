use anyhow::Result;
use chrono;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::ai_provider::ChatMessage;

/**
 * AI提示模板管理器
 * 作者：Evilek
 * 编写日期：2025-07-25
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,

    // 原有字段（保持向后兼容，但标记为废弃）
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub user_prompt_template: String,

    // 新增：两段式提示词模板
    /// 单文件分析阶段的系统提示词
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    #[serde(default)]
    pub file_analysis_system_prompt: String,
    /// 单文件分析阶段的用户提示词模板
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    #[serde(default)]
    pub file_analysis_user_prompt: String,
    /// 总结阶段的系统提示词
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    #[serde(default)]
    pub summary_system_prompt: String,
    /// 总结阶段的用户提示词模板
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    #[serde(default)]
    pub summary_user_prompt: String,

    pub language: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    // 新增配置选项，参考dish-ai-commit
    pub enable_emoji: Option<bool>,
    pub enable_body: Option<bool>,
    pub enable_merge_commit: Option<bool>,
    pub use_recent_commits: Option<bool>,
    pub commit_types: Option<Vec<CommitType>>,
    pub is_custom: Option<bool>, // 标识是否为用户自定义模板
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    // 新增版本管理
    pub version: Option<String>,       // 模板版本号
    pub template_hash: Option<String>, // 模板内容哈希，用于检测变更
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitType {
    pub name: String,
    pub emoji: Option<String>,
    pub description: String,
    pub example_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitContext {
    pub diff: String,
    pub staged_files: Vec<String>,
    pub branch_name: Option<String>,
    pub commit_type: Option<String>,
    pub max_length: Option<usize>,
    pub language: String,
}

/// 模板配置文件结构
/// 作者：Evilek
/// 编写日期：2025-01-29
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub version: String,
    pub last_updated: String,
    pub templates: HashMap<String, PromptTemplate>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            version: "1.1.0".to_string(), // 更新版本号以支持两段式提示词
            last_updated: chrono::Utc::now().to_rfc3339(),
            templates: HashMap::new(),
        }
    }
}

pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
    config_path: Option<PathBuf>, // 独立的模板配置文件路径
    current_version: String,
}

impl PromptManager {
    #[allow(dead_code)] // 备用构造函数，项目中使用new_with_config
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            config_path: None,
            current_version: "1.1.0".to_string(), // 更新版本号以支持两段式提示词
        };

        // 加载默认模板
        manager.load_default_templates();
        manager
    }

    /// 创建带配置文件的PromptManager
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub fn new_with_config(config_path: PathBuf) -> Result<Self> {
        let mut manager = Self {
            templates: HashMap::new(),
            config_path: Some(config_path.clone()),
            current_version: "1.1.0".to_string(), // 更新版本号以支持两段式提示词
        };

        // 尝试加载现有配置
        if config_path.exists() {
            manager.load_from_config()?;
        } else {
            // 加载默认模板并保存
            manager.load_default_templates();
            manager.save_to_config()?;
        }

        Ok(manager)
    }

    /// 从配置文件加载模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    fn load_from_config(&mut self) -> Result<()> {
        if let Some(config_path) = &self.config_path {
            let content = fs::read_to_string(config_path)?;
            let config: TemplateConfig = serde_json::from_str(&content)?;

            // 检查版本是否需要更新
            if config.version != self.current_version {
                // 版本不匹配，需要更新默认模板
                self.load_default_templates();
                self.save_to_config()?;
            } else {
                self.templates = config.templates;
            }
        }
        Ok(())
    }

    /// 保存模板到配置文件
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    fn save_to_config(&self) -> Result<()> {
        if let Some(config_path) = &self.config_path {
            // 确保目录存在
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let config = TemplateConfig {
                version: self.current_version.clone(),
                last_updated: chrono::Utc::now().to_rfc3339(),
                templates: self.templates.clone(),
            };

            let content = serde_json::to_string_pretty(&config)?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }

    /// 计算模板内容哈希
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    fn calculate_template_hash(template: &PromptTemplate) -> String {
        let mut hasher = Sha256::new();
        hasher.update(template.system_prompt.as_bytes());
        hasher.update(template.user_prompt_template.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 获取默认的提交类型配置
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    fn get_default_commit_types(&self) -> Vec<CommitType> {
        vec![
            CommitType {
                name: "feat".to_string(),
                emoji: Some("✨".to_string()),
                description: "新功能".to_string(),
                example_scopes: vec![
                    "user".to_string(),
                    "payment".to_string(),
                    "auth".to_string(),
                ],
            },
            CommitType {
                name: "fix".to_string(),
                emoji: Some("🐛".to_string()),
                description: "修复bug".to_string(),
                example_scopes: vec!["auth".to_string(), "data".to_string(), "ui".to_string()],
            },
            CommitType {
                name: "docs".to_string(),
                emoji: Some("📝".to_string()),
                description: "文档变更".to_string(),
                example_scopes: vec!["README".to_string(), "API".to_string(), "guide".to_string()],
            },
            CommitType {
                name: "style".to_string(),
                emoji: Some("💄".to_string()),
                description: "代码格式变更".to_string(),
                example_scopes: vec!["formatting".to_string(), "lint".to_string()],
            },
            CommitType {
                name: "refactor".to_string(),
                emoji: Some("♻️".to_string()),
                description: "代码重构".to_string(),
                example_scopes: vec![
                    "utils".to_string(),
                    "helpers".to_string(),
                    "core".to_string(),
                ],
            },
            CommitType {
                name: "perf".to_string(),
                emoji: Some("⚡️".to_string()),
                description: "性能优化".to_string(),
                example_scopes: vec![
                    "query".to_string(),
                    "cache".to_string(),
                    "render".to_string(),
                ],
            },
            CommitType {
                name: "test".to_string(),
                emoji: Some("✅".to_string()),
                description: "测试相关".to_string(),
                example_scopes: vec![
                    "unit".to_string(),
                    "e2e".to_string(),
                    "integration".to_string(),
                ],
            },
            CommitType {
                name: "build".to_string(),
                emoji: Some("📦️".to_string()),
                description: "构建系统".to_string(),
                example_scopes: vec![
                    "webpack".to_string(),
                    "npm".to_string(),
                    "docker".to_string(),
                ],
            },
            CommitType {
                name: "ci".to_string(),
                emoji: Some("👷".to_string()),
                description: "CI配置".to_string(),
                example_scopes: vec![
                    "travis".to_string(),
                    "jenkins".to_string(),
                    "github".to_string(),
                ],
            },
            CommitType {
                name: "chore".to_string(),
                emoji: Some("🔧".to_string()),
                description: "其他变更".to_string(),
                example_scopes: vec![
                    "scripts".to_string(),
                    "config".to_string(),
                    "deps".to_string(),
                ],
            },
        ]
    }

    fn load_default_templates(&mut self) {
        // 获取默认的提交类型
        let default_commit_types = self.get_default_commit_types();

        // 标准提交消息模板（两段式优化版）
        let standard_template = PromptTemplate {
            id: "standard".to_string(),
            name: "标准提交消息".to_string(),
            description: "生成符合常规规范的提交消息".to_string(),

            // 原有字段（保持向后兼容）
            system_prompt: r#"你是专业的Git提交消息生成助手。请根据代码变更生成简洁、清晰、符合规范的提交消息。

核心要求：
- 第一行为简短摘要（50字符以内）
- 使用动词开头，如 Add, Fix, Update, Remove 等
- 描述做了什么，而不是为什么做
- 不要以句号结尾
- 如果需要，可以添加详细描述（空行后）

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#.to_string(),
            user_prompt_template: r#"请为以下代码变更生成提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请生成一个简洁明了的提交消息。"#.to_string(),

            // 新增：单文件分析阶段
            file_analysis_system_prompt: r#"你是专业的代码变更分析助手。请分析单个文件的具体变更内容，识别变更的类型、目的和影响。

分析要求：
- 识别变更类型（新增、修改、删除、重构等）
- 分析变更的具体内容和目的
- 评估变更的影响范围
- 提取关键的技术细节
- 保持分析简洁但准确

输出格式：
直接输出对该文件变更的简洁分析，不要包含格式标记或额外说明。"#.to_string(),

            file_analysis_user_prompt: r#"请分析以下文件的变更：

文件路径：{staged_files}

代码差异：
{diff}

请分析这个文件的具体变更内容和目的。"#.to_string(),

            // 新增：总结阶段
            summary_system_prompt: r#"你是专业的Git提交消息生成助手。基于各个文件的变更分析，生成统一的提交消息。

核心要求：
- 第一行为简短摘要（50字符以内）
- 使用动词开头，如 Add, Fix, Update, Remove 等
- 描述做了什么，而不是为什么做
- 不要以句号结尾
- 如果需要，可以添加详细描述（空行后）

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#.to_string(),

            summary_user_prompt: r#"基于以下文件变更分析，生成统一的提交消息：

{diff}

请生成一个简洁明了的提交消息。"#.to_string(),
            language: "FOLLOW_GLOBAL".to_string(),
            max_tokens: Some(200),
            temperature: Some(0.3),
            enable_emoji: Some(false),
            enable_body: Some(true),
            enable_merge_commit: Some(false),
            use_recent_commits: Some(false),
            commit_types: Some(default_commit_types.clone()),
            is_custom: Some(false),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            version: Some(self.current_version.clone()),
            template_hash: None, // 简化处理，不计算hash
        };
        self.add_template(standard_template);

        // 简洁提交消息模板（两段式优化版）
        let chinese_template = PromptTemplate {
            id: "chinese".to_string(),
            name: "简洁提交消息".to_string(),
            description: "生成简洁明了的提交消息".to_string(),

            // 原有字段（保持向后兼容）
            system_prompt:
                r#"你是专业的Git提交消息生成助手。请根据代码变更生成简洁、清晰的提交消息。

核心要求：
- 第一行为简短摘要（25字以内）
- 使用动词开头，如 添加, 修复, 更新, 删除, 优化, 重构 等
- 描述做了什么，而不是为什么做
- 语言简洁明了，避免冗余
- 表达自然流畅

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#
                    .to_string(),
            user_prompt_template: r#"请为以下代码变更生成中文提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请生成一个简洁明了的中文提交消息。"#
                .to_string(),

            // 新增：单文件分析阶段
            file_analysis_system_prompt:
                r#"你是专业的代码变更分析助手。请简洁地分析单个文件的变更内容。

分析要求：
- 识别变更类型（添加、修改、删除、重构等）
- 简述变更的核心内容
- 用词简洁，避免冗余
- 表达自然流畅

输出格式：
直接输出对该文件变更的简洁分析，不超过30字。"#
                    .to_string(),

            file_analysis_user_prompt: r#"请简洁分析以下文件的变更：

文件路径：{staged_files}

代码差异：
{diff}

请用简洁的中文描述这个文件的变更内容。"#
                .to_string(),

            // 新增：总结阶段
            summary_system_prompt:
                r#"你是专业的Git提交消息生成助手。基于文件变更分析，生成简洁的中文提交消息。

核心要求：
- 第一行为简短摘要（25字以内）
- 使用动词开头，如 添加, 修复, 更新, 删除, 优化, 重构 等
- 描述做了什么，而不是为什么做
- 语言简洁明了，避免冗余
- 表达自然流畅

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#
                    .to_string(),

            summary_user_prompt: r#"基于以下文件变更分析，生成简洁的中文提交消息：

{diff}

请生成一个简洁明了的中文提交消息。"#
                .to_string(),
            language: "FOLLOW_GLOBAL".to_string(),
            max_tokens: Some(150),
            temperature: Some(0.3),
            enable_emoji: Some(false),
            enable_body: Some(true),
            enable_merge_commit: Some(false),
            use_recent_commits: Some(false),
            commit_types: Some(default_commit_types.clone()),
            is_custom: Some(false),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            version: Some(self.current_version.clone()),
            template_hash: None, // 简化处理，不计算hash
        };
        self.add_template(chinese_template);

        // 详细提交消息模板（两段式优化版）
        let detailed_template = PromptTemplate {
            id: "detailed".to_string(),
            name: "详细提交消息".to_string(),
            description: "生成包含详细描述的提交消息".to_string(),

            // 原有字段（保持向后兼容）
            system_prompt: r#"你是专业的Git提交消息生成助手。请根据代码变更生成详细的提交消息，包括摘要和详细描述。

核心要求：
- 第一行：简短摘要（50字符以内）
- 空行后添加详细描述
- 解释做了什么变更和变更原因
- 如果有破坏性变更，请说明
- 摘要使用动词开头
- 详细描述使用项目符号
- 保持专业和清晰

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#.to_string(),
            user_prompt_template: r#"请为以下代码变更生成详细的提交消息：

分支：{branch_name}
变更的文件：
{staged_files}

代码差异：
{diff}

请生成包含摘要和详细描述的提交消息。"#.to_string(),

            // 新增：单文件分析阶段
            file_analysis_system_prompt: r#"你是专业的代码变更分析助手。请详细分析单个文件的变更内容，为后续生成详细提交消息提供基础。

分析要求：
- 识别变更类型和具体内容
- 分析变更的目的和影响
- 识别关键的技术细节
- 评估是否有破坏性变更
- 提供充分的上下文信息

输出格式：
提供该文件变更的详细分析，包括变更类型、具体内容、目的和影响。"#.to_string(),

            file_analysis_user_prompt: r#"请详细分析以下文件的变更：

文件路径：{staged_files}

代码差异：
{diff}

请提供这个文件变更的详细分析，包括变更类型、具体内容、目的和可能的影响。"#.to_string(),

            // 新增：总结阶段
            summary_system_prompt: r#"你是专业的Git提交消息生成助手。基于详细的文件变更分析，生成包含摘要和详细描述的提交消息。

核心要求：
- 第一行：简短摘要（50字符以内）
- 空行后添加详细描述
- 解释做了什么变更和变更原因
- 如果有破坏性变更，请说明
- 摘要使用动词开头
- 详细描述使用项目符号
- 保持专业和清晰

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#.to_string(),

            summary_user_prompt: r#"基于以下详细的文件变更分析，生成包含摘要和详细描述的提交消息：

{diff}

请生成包含摘要和详细描述的提交消息。"#.to_string(),
            language: "FOLLOW_GLOBAL".to_string(),
            max_tokens: Some(400),
            temperature: Some(0.4),
            enable_emoji: Some(false),
            enable_body: Some(true),
            enable_merge_commit: Some(false),
            use_recent_commits: Some(true),
            commit_types: Some(default_commit_types.clone()),
            is_custom: Some(false),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            version: Some(self.current_version.clone()),
            template_hash: None, // 简化处理，不计算hash
        };
        self.add_template(detailed_template);

        // 约定式提交模板（两段式优化版）
        let conventional_template = PromptTemplate {
            id: "conventional".to_string(),
            name: "约定式提交".to_string(),
            description: "生成符合约定式提交规范的消息".to_string(),

            // 原有字段（保持向后兼容）
            system_prompt: r#"你是专业的Git提交消息生成助手。请根据代码变更生成符合约定式提交规范的消息。

核心要求：
- 格式：<type>[optional scope]: <description>
- 类型：feat(新功能), fix(修复bug), docs(文档变更), style(代码格式变更), refactor(重构), test(测试相关), chore(构建过程或辅助工具的变动)
- 描述使用小写开头
- 不要以句号结尾
- 描述要简洁明了

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#.to_string(),
            user_prompt_template: r#"请为以下代码变更生成约定式提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请分析变更类型并生成符合约定式提交规范的消息。"#.to_string(),

            // 新增：单文件分析阶段
            file_analysis_system_prompt: r#"你是专业的代码变更分析助手。请分析单个文件的变更，识别约定式提交的类型和范围。

分析要求：
- 识别变更类型：feat, fix, docs, style, refactor, test, chore
- 确定变更范围（如果适用）
- 分析变更的具体内容
- 评估变更的影响

输出格式：
输出该文件的变更类型分析和具体内容描述。"#.to_string(),

            file_analysis_user_prompt: r#"请分析以下文件的变更类型：

文件路径：{staged_files}

代码差异：
{diff}

请识别这个文件变更的约定式提交类型和具体内容。"#.to_string(),

            // 新增：总结阶段
            summary_system_prompt: r#"你是专业的Git提交消息生成助手。基于文件变更分析，生成符合约定式提交规范的消息。

核心要求：
- 格式：<type>[optional scope]: <description>
- 类型：feat(新功能), fix(修复bug), docs(文档变更), style(代码格式变更), refactor(重构), test(测试相关), chore(构建过程或辅助工具的变动)
- 描述使用小写开头
- 不要以句号结尾
- 描述要简洁明了

严格禁止：
- 不要包含任何解释、问候或额外文本
- 不要添加格式说明或元数据
- 不要在输出中包含三重反引号或标题格式

直接输出提交消息，无需其他内容。"#.to_string(),

            summary_user_prompt: r#"基于以下文件变更分析，生成符合约定式提交规范的消息：

{diff}

请分析变更类型并生成符合约定式提交规范的消息。"#.to_string(),
            language: "FOLLOW_GLOBAL".to_string(),
            max_tokens: Some(150),
            temperature: Some(0.2),
            enable_emoji: Some(true),
            enable_body: Some(false),
            enable_merge_commit: Some(false),
            use_recent_commits: Some(false),
            commit_types: Some(default_commit_types.clone()),
            is_custom: Some(false),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            version: Some(self.current_version.clone()),
            template_hash: None, // 简化处理，不计算hash
        };
        self.add_template(conventional_template);
    }

    pub fn add_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn get_template(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    pub fn get_all_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }

    /// 生成AI消息，根据语言配置调整提示词（兼容接口，内部使用两段式处理）
    /// 作者：Evilek
    /// 编写日期：2025-07-28
    /// 更新日期：2025-08-08 (重构为两段式处理)
    pub fn generate_messages(
        &self,
        template_id: &str,
        context: &CommitContext,
    ) -> Result<Vec<ChatMessage>> {
        // 为了保持向后兼容，这里使用总结阶段的提示词
        // 实际的两段式处理应该使用 execute_two_phase_commit 方法
        self.generate_summary_messages(template_id, context, &[])
    }

    /// 生成单文件分析阶段的AI消息
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    pub fn generate_file_analysis_messages(
        &self,
        template_id: &str,
        file_path: &str,
        file_diff: &str,
        context: &CommitContext,
    ) -> Result<Vec<ChatMessage>> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

        let mut messages = Vec::new();

        // 使用单文件分析的系统提示词，如果为空则回退到原有的系统提示词
        let system_prompt_base = if !template.file_analysis_system_prompt.is_empty() {
            &template.file_analysis_system_prompt
        } else {
            &template.system_prompt
        };

        // 使用动态系统提示词生成
        let system_prompt = self.generate_dynamic_system_prompt_for_phase(
            system_prompt_base,
            template,
            context,
            "file_analysis",
        );

        // 添加系统消息
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        });

        // 生成用户消息 - 使用单文件分析的用户提示词
        let user_prompt_template = if !template.file_analysis_user_prompt.is_empty() {
            &template.file_analysis_user_prompt
        } else {
            &template.user_prompt_template
        };

        // 创建单文件上下文
        let file_context = CommitContext {
            diff: file_diff.to_string(),
            staged_files: vec![file_path.to_string()],
            branch_name: context.branch_name.clone(),
            commit_type: context.commit_type.clone(),
            max_length: context.max_length,
            language: context.language.clone(),
        };

        let user_content = self.render_template(user_prompt_template, &file_context)?;
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_content,
        });

        Ok(messages)
    }

    /// 生成总结阶段的AI消息
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    pub fn generate_summary_messages(
        &self,
        template_id: &str,
        context: &CommitContext,
        file_summaries: &[&str],
    ) -> Result<Vec<ChatMessage>> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

        let mut messages = Vec::new();

        // 使用总结阶段的系统提示词，如果为空则回退到原有的系统提示词
        let system_prompt_base = if !template.summary_system_prompt.is_empty() {
            &template.summary_system_prompt
        } else {
            &template.system_prompt
        };

        // 使用动态系统提示词生成
        let system_prompt = self.generate_dynamic_system_prompt_for_phase(
            system_prompt_base,
            template,
            context,
            "summary",
        );

        // 添加系统消息
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        });

        // 生成用户消息 - 使用总结阶段的用户提示词
        let user_prompt_template = if !template.summary_user_prompt.is_empty() {
            &template.summary_user_prompt
        } else {
            &template.user_prompt_template
        };

        // 如果有文件摘要，则构建包含摘要的上下文
        let summary_context = if !file_summaries.is_empty() {
            let summaries_text = file_summaries.join("\n\n");
            CommitContext {
                diff: format!(
                    "文件分析摘要:\n{}\n\n原始差异:\n{}",
                    summaries_text, context.diff
                ),
                staged_files: context.staged_files.clone(),
                branch_name: context.branch_name.clone(),
                commit_type: context.commit_type.clone(),
                max_length: context.max_length,
                language: context.language.clone(),
            }
        } else {
            context.clone()
        };

        let user_content = self.render_template(user_prompt_template, &summary_context)?;
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_content,
        });

        Ok(messages)
    }

    /// 执行完整的两段式提交流程（新的核心方法）
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    pub async fn execute_two_phase_commit<F>(
        &self,
        template_id: &str,
        staged_files: &[String],
        context: &CommitContext,
        ai_generate_fn: F,
    ) -> Result<String>
    where
        F: Fn(Vec<ChatMessage>) -> Result<String> + Copy,
    {
        let mut file_summaries = Vec::new();

        // 第一阶段：对每个文件进行单独分析
        for file_path in staged_files {
            // 这里需要获取单个文件的diff，暂时使用整体diff作为占位符
            // 实际使用时需要从外部传入单文件diff获取函数
            let file_diff = context.diff.clone(); // 占位符，实际应该是单文件diff

            let messages = self.generate_file_analysis_messages(
                template_id,
                file_path,
                &file_diff,
                context,
            )?;

            let analysis_result = ai_generate_fn(messages)?;
            file_summaries.push(analysis_result);
        }

        // 第二阶段：基于所有文件分析结果生成最终提交消息
        let summary_messages = self.generate_summary_messages(
            template_id,
            context,
            &file_summaries.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )?;

        let final_commit_message = ai_generate_fn(summary_messages)?;
        Ok(final_commit_message)
    }

    /// 检查模板是否支持两段式处理
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    pub fn supports_two_phase(&self, template_id: &str) -> bool {
        if let Some(template) = self.get_template(template_id) {
            !template.file_analysis_system_prompt.is_empty()
                && !template.summary_system_prompt.is_empty()
        } else {
            false
        }
    }

    /// 获取模板的两段式配置状态
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    pub fn get_two_phase_status(&self, template_id: &str) -> Option<(bool, bool)> {
        if let Some(template) = self.get_template(template_id) {
            Some((
                !template.file_analysis_system_prompt.is_empty(),
                !template.summary_system_prompt.is_empty(),
            ))
        } else {
            None
        }
    }

    fn render_template(&self, template: &str, context: &CommitContext) -> Result<String> {
        let mut rendered = template.to_string();

        // 替换变量
        rendered = rendered.replace("{diff}", &self.truncate_diff(&context.diff, 3000));
        rendered = rendered.replace("{staged_files}", &context.staged_files.join("\n"));

        if let Some(branch) = &context.branch_name {
            rendered = rendered.replace("{branch_name}", branch);
        } else {
            rendered = rendered.replace("{branch_name}", "main");
        }

        if let Some(commit_type) = &context.commit_type {
            rendered = rendered.replace("{commit_type}", commit_type);
        }

        Ok(rendered)
    }

    fn truncate_diff(&self, diff: &str, max_length: usize) -> String {
        if diff.len() <= max_length {
            return diff.to_string();
        }

        let lines: Vec<&str> = diff.lines().collect();
        let mut result = String::new();
        let mut current_length = 0;

        for line in lines {
            if current_length + line.len() + 1 > max_length {
                result.push_str("\n... (diff truncated)");
                break;
            }

            if !result.is_empty() {
                result.push('\n');
                current_length += 1;
            }

            result.push_str(line);
            current_length += line.len();
        }

        result
    }

    pub fn get_template_config(&self, template_id: &str) -> Option<(Option<u32>, Option<f32>)> {
        self.get_template(template_id)
            .map(|t| (t.max_tokens, t.temperature))
    }

    /// 更新模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub fn update_template(&mut self, template: PromptTemplate) -> Result<()> {
        let mut updated_template = template;
        updated_template.updated_at = Some(chrono::Utc::now().to_rfc3339());

        self.templates
            .insert(updated_template.id.clone(), updated_template);
        Ok(())
    }

    /// 删除模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub fn delete_template(&mut self, template_id: &str) -> Result<()> {
        // 不允许删除默认模板
        if let Some(template) = self.get_template(template_id) {
            if template.is_custom == Some(false) {
                return Err(anyhow::anyhow!("Cannot delete default template"));
            }
        }

        self.templates.remove(template_id);
        self.save_to_config()?;
        Ok(())
    }

    /// 重新加载默认模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    #[allow(dead_code)] // 预留的管理功能，通过ai_manager调用
    pub fn reload_default_templates(&mut self) -> Result<()> {
        // 清空现有模板
        self.templates.clear();

        // 重新加载默认模板
        self.load_default_templates();

        // 保存到配置文件
        self.save_to_config()?;

        Ok(())
    }

    /// 创建新的自定义模板
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub fn create_custom_template(&mut self, mut template: PromptTemplate) -> Result<()> {
        // 确保是自定义模板
        template.is_custom = Some(true);
        template.created_at = Some(chrono::Utc::now().to_rfc3339());
        template.updated_at = Some(chrono::Utc::now().to_rfc3339());

        // 如果没有设置提交类型，使用默认的
        if template.commit_types.is_none() {
            template.commit_types = Some(self.get_default_commit_types());
        }

        self.add_template(template);
        Ok(())
    }

    /// 获取自定义模板列表
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub fn get_custom_templates(&self) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|t| t.is_custom == Some(true))
            .collect()
    }

    /// 获取默认模板列表
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    pub fn get_default_templates(&self) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|t| t.is_custom != Some(true))
            .collect()
    }

    /// 根据配置动态生成系统提示词（参考dish-ai-commit）
    /// 作者：Evilek
    /// 编写日期：2025-01-29
    /// 更新日期：2025-08-08 (支持两段式处理)
    pub fn generate_dynamic_system_prompt(
        &self,
        template: &PromptTemplate,
        context: &CommitContext,
    ) -> String {
        // 为了保持向后兼容，使用原有的系统提示词
        self.generate_dynamic_system_prompt_for_phase(
            &template.system_prompt,
            template,
            context,
            "legacy",
        )
    }

    /// 为特定阶段生成动态系统提示词
    /// 作者：Evilek
    /// 编写日期：2025-08-08
    pub fn generate_dynamic_system_prompt_for_phase(
        &self,
        base_system_prompt: &str,
        template: &PromptTemplate,
        context: &CommitContext,
        phase: &str,
    ) -> String {
        let mut system_prompt = base_system_prompt.to_string();

        // 根据阶段添加特定的指导
        match phase {
            "file_analysis" => {
                system_prompt.push_str(
                    "\n\n阶段说明：这是单文件分析阶段，请专注于分析当前文件的具体变更内容和意图。",
                );
            }
            "summary" => {
                system_prompt.push_str(
                    "\n\n阶段说明：这是总结阶段，请基于所有文件的分析结果生成统一的提交消息。",
                );
            }
            _ => {
                // legacy 模式或其他，不添加特殊说明
            }
        }

        // 根据配置添加额外的指导
        if template.enable_emoji == Some(true) {
            system_prompt.push_str("\n\n重要：请在提交类型前添加对应的emoji表情符号。");
        }

        if template.enable_body == Some(false) {
            system_prompt.push_str("\n\n重要：只生成提交消息的标题行，不要包含详细描述。");
        }

        if template.enable_merge_commit == Some(true) {
            system_prompt.push_str("\n\n重要：如果有多个文件变更，请将它们合并为一个提交消息。");
        } else if phase != "file_analysis" {
            // 单文件分析阶段不需要这个指导
            system_prompt
                .push_str("\n\n重要：如果有多个文件变更，请为每个主要变更生成单独的提交消息。");
        }

        // 确定实际使用的语言
        let effective_language = self.resolve_effective_language(template, context);

        // 添加语言声明
        let language_instruction = self.generate_language_instruction(&effective_language);
        system_prompt.push_str(&language_instruction);

        system_prompt
    }

    /// 解析实际使用的语言（处理跟随全局逻辑）
    /// 作者：Evilek
    /// 编写日期：2025-08-05
    fn resolve_effective_language(
        &self,
        template: &PromptTemplate,
        context: &CommitContext,
    ) -> String {
        if template.language == "FOLLOW_GLOBAL" {
            // 跟随全局设置，使用context中的语言
            self.convert_language_code_to_name(&context.language)
        } else {
            // 使用模板独立的语言设置
            template.language.clone()
        }
    }

    /// 将语言代码转换为语言名称
    /// 作者：Evilek
    /// 编写日期：2025-08-05
    fn convert_language_code_to_name(&self, language_code: &str) -> String {
        match language_code {
            "zh-CN" => "Simplified Chinese".to_string(),
            "zh-TW" => "Traditional Chinese".to_string(),
            "en" => "English".to_string(),
            "ja" => "Japanese".to_string(),
            "ko" => "Korean".to_string(),
            "fr" => "French".to_string(),
            "de" => "German".to_string(),
            "es" => "Spanish".to_string(),
            "ru" => "Russian".to_string(),
            "pt" => "Portuguese".to_string(),
            "it" => "Italian".to_string(),
            "nl" => "Dutch".to_string(),
            "sv" => "Swedish".to_string(),
            "cs" => "Czech".to_string(),
            "pl" => "Polish".to_string(),
            "tr" => "Turkish".to_string(),
            "vi" => "Vietnamese".to_string(),
            "th" => "Thai".to_string(),
            "id" => "Indonesian".to_string(),
            _ => "English".to_string(), // 默认英文
        }
    }

    /// 生成语言指令
    /// 作者：Evilek
    /// 编写日期：2025-08-05
    fn generate_language_instruction(&self, language: &str) -> String {
        match language {
            "Simplified Chinese" => "\n\n重要：请使用简体中文生成提交消息，确保语言自然流畅。".to_string(),
            "Traditional Chinese" => "\n\n重要：请使用繁体中文生成提交消息，确保语言自然流畅。".to_string(),
            "English" => "\n\nImportant: Please generate commit messages in English, ensure natural and fluent language.".to_string(),
            "Japanese" => "\n\n重要：日本語でコミットメッセージを生成してください。自然で流暢な言語を確保してください。".to_string(),
            "Korean" => "\n\n중요: 한국어로 커밋 메시지를 생성해주세요. 자연스럽고 유창한 언어를 보장해주세요.".to_string(),
            "French" => "\n\nImportant: Veuillez générer des messages de commit en français, en vous assurant que le langage soit naturel et fluide.".to_string(),
            "German" => "\n\nWichtig: Bitte generieren Sie Commit-Nachrichten auf Deutsch und stellen Sie sicher, dass die Sprache natürlich und fließend ist.".to_string(),
            "Spanish" => "\n\nImportante: Por favor genere mensajes de commit en español, asegurando que el lenguaje sea natural y fluido.".to_string(),
            "Russian" => "\n\nВажно: Пожалуйста, генерируйте сообщения коммитов на русском языке, обеспечивая естественность и беглость языка.".to_string(),
            "Portuguese" => "\n\nImportante: Por favor, gere mensagens de commit em português, garantindo que a linguagem seja natural e fluida.".to_string(),
            "Italian" => "\n\nImportante: Si prega di generare messaggi di commit in italiano, assicurandosi che il linguaggio sia naturale e fluido.".to_string(),
            "Dutch" => "\n\nBelangrijk: Genereer commit-berichten in het Nederlands en zorg ervoor dat de taal natuurlijk en vloeiend is.".to_string(),
            "Swedish" => "\n\nViktigt: Vänligen generera commit-meddelanden på svenska och se till att språket är naturligt och flytande.".to_string(),
            "Czech" => "\n\nDůležité: Prosím generujte commit zprávy v češtině a zajistěte, aby byl jazyk přirozený a plynulý.".to_string(),
            "Polish" => "\n\nWażne: Proszę generować wiadomości commit w języku polskim, zapewniając naturalność i płynność języka.".to_string(),
            "Turkish" => "\n\nÖnemli: Lütfen commit mesajlarını Türkçe olarak oluşturun ve dilin doğal ve akıcı olmasını sağlayın.".to_string(),
            "Vietnamese" => "\n\nQuan trọng: Vui lòng tạo thông điệp commit bằng tiếng Việt, đảm bảo ngôn ngữ tự nhiên và trôi chảy.".to_string(),
            "Thai" => "\n\nสำคัญ: โปรดสร้างข้อความ commit เป็นภาษาไทย โดยให้แน่ใจว่าภาษาเป็นธรรมชาติและลื่นไหล".to_string(),
            "Indonesian" => "\n\nPenting: Harap buat pesan commit dalam bahasa Indonesia, pastikan bahasa yang digunakan alami dan lancar.".to_string(),
            _ => "\n\nImportant: Please generate commit messages in English, ensure natural and fluent language.".to_string(),
        }
    }
}
