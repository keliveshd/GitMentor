use regex::Regex;

/**
 * AI响应内容清理工具
 * 用于从AI响应中提取真正的提交消息，过滤掉思考过程和元信息
 * 作者：Evilek
 * 编写日期：2025-01-19
 */

pub struct ResponseCleaner;

impl ResponseCleaner {
    /// 清理AI响应内容，提取真正的提交消息
    ///
    /// # Arguments
    /// * `content` - AI响应的原始内容
    ///
    /// # Returns
    /// * `String` - 清理后的提交消息
    pub fn clean_commit_message(content: &str) -> String {
        println!("🔍 [ResponseCleaner] 开始清理响应内容...");

        let mut cleaned = content.to_string();

        // 1. 移除思考标记中的内容
        let step0 = Self::remove_thinking_content(&cleaned);
        if cleaned.len() != step0.len() {
            println!("🔍 [Step 0] 移除思考内容后长度: {} -> {}", cleaned.len(), step0.len());
        }
        cleaned = step0;

        // 2. 移除开头的分析过程
        let step1 = Self::remove_analysis_prefix(&cleaned);
        println!("🔍 [Step 1] 移除分析过程后长度: {} -> {}", cleaned.len(), step1.len());
        cleaned = step1;

        // 3. 移除重复的提交消息（AI可能会重复生成）
        let step2 = Self::remove_duplicates(&cleaned);
        println!("🔍 [Step 2] 移除重复后长度: {} -> {}", cleaned.len(), step2.len());
        cleaned = step2;

        // 4. 提取最后的完整提交消息
        let step3 = Self::extract_final_message(&cleaned);
        println!("🔍 [Step 3] 提取最终消息后长度: {} -> {}", cleaned.len(), step3.len());
        if step3.is_empty() {
            println!("🔍 [WARNING] 提取最终消息后内容为空！");
            // 如果提取后内容为空，使用原始内容的前几行作为备选
            let fallback = content.lines().take(10).collect::<Vec<_>>().join("\n");
            println!("🔍 [FALLBACK] 使用原始内容前10行作为备选，长度: {}", fallback.len());
            return fallback.trim().to_string();
        }
        cleaned = step3;

        // 5. 清理多余的空白行
        let step4 = Self::clean_whitespace(&cleaned);
        println!("🔍 [Step 4] 清理空白后长度: {} -> {}", cleaned.len(), step4.len());
        cleaned = step4;

        let final_result = cleaned.trim().to_string();
        println!("🔍 [ResponseCleaner] 清理完成，最终长度: {}", final_result.len());
        final_result
    }

    /// 移除思考标记中的内容
    fn remove_thinking_content(content: &str) -> String {
        // 使用正则表达式匹配并移除</think>...</think>标签及其中的内容
        let think_regex = Regex::new(r"(?s)</think>.*?</think>").unwrap();
        let result = think_regex.replace_all(content, "").to_string();

        // 如果找到了思考内容，打印日志
        if result.len() != content.len() {
            println!("🔍 [remove_thinking_content] 已移除思考标记内容");
        }

        result
    }

    /// 移除开头的分析过程
    fn remove_analysis_prefix(content: &str) -> String {
        // 常见的前缀模式
        let patterns = vec![
            r"(?s)^让我分析一下.*?(\n\n|$)",
            r"(?s)^根据描述.*?(\n\n|$)",
            r"(?s)^从描述中.*?(\n\n|$)",
            r"(?s)^我需要分析.*?(\n\n|$)",
            r"(?s)^首先.*?(\n\n|$)",
            r"(?s)^现在.*?(\n\n|$)",
            r"(?s)^根据要求.*?(\n\n|$)",
        ];

        let mut result = content.to_string();

        for pattern in patterns {
            let re = Regex::new(pattern).unwrap();
            result = re.replace(&result, "").to_string();
        }

        result
    }

    /// 移除重复的提交消息
    fn remove_duplicates(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for line in lines {
            let trimmed = line.trim();
            // 跳过太短的行（可能是标题或分隔符）
            if trimmed.len() < 5 {
                result.push(line.to_string());
                continue;
            }

            // 如果是重复的行，跳过
            if seen.contains(trimmed) {
                continue;
            }

            seen.insert(trimmed.to_string());
            result.push(line.to_string());
        }

        result.join("\n")
    }

    /// 提取最后的完整提交消息
    fn extract_final_message(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut in_message = false;

        println!("🔍 [extract_final_message] 开始处理 {} 行内容", lines.len());

        // 从后往前找，找到最后一个有效的提交消息
        for (i, line) in lines.iter().rev().enumerate() {
            let trimmed = line.trim();

            println!("🔍 [extract_final_message] 处理第 {} 行: '{}'", i, trimmed);

            // 空行处理
            if trimmed.is_empty() {
                if in_message {
                    result.insert(0, line.to_string());
                    println!("🔍 [extract_final_message] 添加空行到结果");
                }
                continue;
            }

            // 检查是否是提交消息的开头（通常以动词开头）
            if Self::is_commit_message_start(trimmed) {
                if !in_message {
                    in_message = true;
                    println!("🔍 [extract_final_message] 找到消息开头，开始收集");
                }
                result.insert(0, line.to_string());
                println!("🔍 [extract_final_message] 添加到结果");
            } else if in_message {
                // 如果已经在消息中，继续添加
                result.insert(0, line.to_string());
                println!("🔍 [extract_final_message] 在消息中，继续添加");
            } else {
                println!("🔍 [extract_final_message] 跳过此行");
            }
        }

        let final_result = result.join("\n");
        println!("🔍 [extract_final_message] 最终结果包含 {} 行", final_result.lines().count());
        final_result
    }

    /// 检查是否是提交消息的开头
    fn is_commit_message_start(text: &str) -> bool {
        // 常见的提交消息开头动词
        let start_verbs = vec![
            "添加", "新增", "修复", "更新", "改进", "优化", "重构",
            "删除", "移除", "调整", "修改", "创建", "实现",
            "add", "fix", "update", "improve", "optimize", "refactor",
            "delete", "remove", "adjust", "modify", "create", "implement",
        ];

        println!("🔍 [is_commit_message_start] 检查文本: '{}'", text);

        for verb in start_verbs {
            if text.starts_with(verb) {
                println!("🔍 [is_commit_message_start] 匹配到动词: {}", verb);
                return true;
            }
        }

        println!("🔍 [is_commit_message_start] 未匹配到任何动词");
        false
    }

    /// 清理多余的空白行
    fn clean_whitespace(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();
        let mut empty_count = 0;

        for line in lines {
            if line.trim().is_empty() {
                empty_count += 1;
                // 最多保留2个连续空行
                if empty_count <= 2 {
                    result.push(line.to_string());
                }
            } else {
                empty_count = 0;
                result.push(line.to_string());
            }
        }

        // 移除开头和结尾的空行
        let result = result.join("\n");
        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_commit_message() {
        let sample = r#"让我分析一下提供的文件变更信息：

从描述中，我看到有几个文件变更：

1. 新增了智能自动切换提交分析模板功能的规划文档 (`auto-template-switching.md`)
2. 新增了Claude本地权限配置文件 (`settings.local.json`)

根据要求，我需要为每个主要变更生成单独的提交消息...

新增智能自动切换提交分析模板功能的规划文档，详细描述了技术实现方案、任务分解和阶段目标

新增Claude权限配置

添加Claude本地权限配置文件，允许使用mcp__cunzhi__ji和mcp__cunzhi__zhi两个MCP工具"#;

        let cleaned = ResponseCleaner::clean_commit_message(sample);
        println!("Cleaned: {}", cleaned);

        // 应该只包含实际的提交消息
        assert!(cleaned.contains("新增智能自动切换提交分析模板功能的规划文档"));
        assert!(cleaned.contains("新增Claude权限配置"));
        assert!(!cleaned.contains("让我分析一下"));
    }
}