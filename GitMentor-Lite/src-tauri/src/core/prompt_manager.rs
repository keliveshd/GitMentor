use anyhow::Result;
use chrono;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub system_prompt: String,
    pub user_prompt_template: String,
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

pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // 加载默认模板
        manager.load_default_templates();
        manager
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

        // 标准提交消息模板
        self.add_template(PromptTemplate {
            id: "standard".to_string(),
            name: "标准提交消息".to_string(),
            description: "生成符合常规规范的提交消息".to_string(),
            system_prompt: r#"你是一个专业的Git提交消息生成助手。请根据代码变更生成简洁、清晰、符合规范的提交消息。

规则：
1. 使用英文编写提交消息
2. 第一行为简短摘要（50字符以内）
3. 使用动词开头，如 Add, Fix, Update, Remove 等
4. 描述做了什么，而不是为什么做
5. 不要以句号结尾
6. 如果需要，可以添加详细描述（空行后）"#.to_string(),
            user_prompt_template: r#"请为以下代码变更生成提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请生成一个简洁明了的提交消息。"#.to_string(),
            language: "en".to_string(),
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
        });

        // 中文提交消息模板
        self.add_template(PromptTemplate {
            id: "chinese".to_string(),
            name: "中文提交消息".to_string(),
            description: "生成中文的提交消息".to_string(),
            system_prompt:
                r#"你是一个专业的Git提交消息生成助手。请根据代码变更生成简洁、清晰的中文提交消息。

规则：
1. 必须使用中文编写提交消息
2. 第一行为简短摘要（25字以内）
3. 使用动词开头，如 添加, 修复, 更新, 删除, 优化, 重构 等
4. 描述做了什么，而不是为什么做
5. 语言简洁明了，避免冗余
6. 符合中文表达习惯，自然流畅"#
                    .to_string(),
            user_prompt_template: r#"请为以下代码变更生成中文提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请生成一个简洁明了的中文提交消息。"#
                .to_string(),
            language: "zh".to_string(),
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
        });

        // 详细提交消息模板
        self.add_template(PromptTemplate {
            id: "detailed".to_string(),
            name: "详细提交消息".to_string(),
            description: "生成包含详细描述的提交消息".to_string(),
            system_prompt: r#"你是一个专业的Git提交消息生成助手。请根据代码变更生成详细的提交消息，包括摘要和详细描述。

格式：
第一行：简短摘要（50字符以内）
空行
详细描述：
- 解释做了什么变更
- 说明变更的原因
- 如果有破坏性变更，请说明

规则：
1. 使用英文编写
2. 摘要使用动词开头
3. 详细描述使用项目符号
4. 保持专业和清晰"#.to_string(),
            user_prompt_template: r#"请为以下代码变更生成详细的提交消息：

分支：{branch_name}
变更的文件：
{staged_files}

代码差异：
{diff}

请生成包含摘要和详细描述的提交消息。"#.to_string(),
            language: "en".to_string(),
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
        });

        // 约定式提交模板
        self.add_template(PromptTemplate {
            id: "conventional".to_string(),
            name: "约定式提交".to_string(),
            description: "生成符合约定式提交规范的消息".to_string(),
            system_prompt:
                r#"你是一个专业的Git提交消息生成助手。请根据代码变更生成符合约定式提交规范的消息。

格式：<type>[optional scope]: <description>

类型（type）：
- feat: 新功能
- fix: 修复bug
- docs: 文档变更
- style: 代码格式变更
- refactor: 重构
- test: 测试相关
- chore: 构建过程或辅助工具的变动

规则：
1. 使用英文编写
2. 描述使用小写开头
3. 不要以句号结尾
4. 描述要简洁明了"#
                    .to_string(),
            user_prompt_template: r#"请为以下代码变更生成约定式提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请分析变更类型并生成符合约定式提交规范的消息。"#
                .to_string(),
            language: "en".to_string(),
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
        });
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

    /// 生成AI消息，根据语言配置调整提示词
    /// 作者：Evilek
    /// 编写日期：2025-07-28
    /// 更新日期：2025-01-29 (使用动态系统提示词)
    pub fn generate_messages(
        &self,
        template_id: &str,
        context: &CommitContext,
    ) -> Result<Vec<ChatMessage>> {
        let template = self
            .get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;

        let mut messages = Vec::new();

        // 使用动态系统提示词生成
        let system_prompt = self.generate_dynamic_system_prompt(template, context);

        // 添加系统消息
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        });

        // 生成用户消息
        let user_content = self.render_template(&template.user_prompt_template, context)?;
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_content,
        });

        Ok(messages)
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
    /// 更新日期：2025-08-05 (支持跟随全局和完整语言声明)
    pub fn generate_dynamic_system_prompt(
        &self,
        template: &PromptTemplate,
        context: &CommitContext,
    ) -> String {
        let mut system_prompt = template.system_prompt.clone();

        // 根据配置添加额外的指导
        if template.enable_emoji == Some(true) {
            system_prompt.push_str("\n\n重要：请在提交类型前添加对应的emoji表情符号。");
        }

        if template.enable_body == Some(false) {
            system_prompt.push_str("\n\n重要：只生成提交消息的标题行，不要包含详细描述。");
        }

        if template.enable_merge_commit == Some(true) {
            system_prompt.push_str("\n\n重要：如果有多个文件变更，请将它们合并为一个提交消息。");
        } else {
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
