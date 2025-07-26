use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

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
    
    fn load_default_templates(&mut self) {
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
        });
        
        // 中文提交消息模板
        self.add_template(PromptTemplate {
            id: "chinese".to_string(),
            name: "中文提交消息".to_string(),
            description: "生成中文的提交消息".to_string(),
            system_prompt: r#"你是一个专业的Git提交消息生成助手。请根据代码变更生成简洁、清晰的中文提交消息。

规则：
1. 使用中文编写提交消息
2. 第一行为简短摘要（25字以内）
3. 使用动词开头，如 添加, 修复, 更新, 删除 等
4. 描述做了什么，而不是为什么做
5. 语言简洁明了，避免冗余"#.to_string(),
            user_prompt_template: r#"请为以下代码变更生成中文提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请生成一个简洁明了的中文提交消息。"#.to_string(),
            language: "zh".to_string(),
            max_tokens: Some(150),
            temperature: Some(0.3),
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
        });
        
        // 约定式提交模板
        self.add_template(PromptTemplate {
            id: "conventional".to_string(),
            name: "约定式提交".to_string(),
            description: "生成符合约定式提交规范的消息".to_string(),
            system_prompt: r#"你是一个专业的Git提交消息生成助手。请根据代码变更生成符合约定式提交规范的消息。

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
4. 描述要简洁明了"#.to_string(),
            user_prompt_template: r#"请为以下代码变更生成约定式提交消息：

变更的文件：
{staged_files}

代码差异：
{diff}

请分析变更类型并生成符合约定式提交规范的消息。"#.to_string(),
            language: "en".to_string(),
            max_tokens: Some(150),
            temperature: Some(0.2),
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
    
    pub fn generate_messages(&self, template_id: &str, context: &CommitContext) -> Result<Vec<ChatMessage>> {
        let template = self.get_template(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_id))?;
        
        let mut messages = Vec::new();
        
        // 添加系统消息
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: template.system_prompt.clone(),
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
}
