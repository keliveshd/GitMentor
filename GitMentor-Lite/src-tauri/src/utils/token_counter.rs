use crate::core::ai_provider::AIRequest;

/**
 * Token计算工具
 * 作者：Evilek
 * 编写日期：2025-08-04
 */

pub struct TokenCounter;

impl TokenCounter {
    /// 估算文本的token数量
    /// 使用简化的估算方法：英文约4字符/token，中文约1.5字符/token
    pub fn estimate_tokens(text: &str) -> u32 {
        let char_count = text.chars().count();
        let chinese_chars = text.chars().filter(|c| Self::is_chinese_char(*c)).count();
        let other_chars = char_count - chinese_chars;

        // 中文字符按1.5字符/token计算，其他字符按4字符/token计算
        let chinese_tokens = (chinese_chars as f32 / 1.5).ceil() as u32;
        let other_tokens = (other_chars as f32 / 4.0).ceil() as u32;

        chinese_tokens + other_tokens
    }

    /// 估算AI请求的总token数量
    pub fn estimate_request_tokens(request: &AIRequest) -> u32 {
        let message_tokens: u32 = request
            .messages
            .iter()
            .map(|msg| Self::estimate_tokens(&msg.content))
            .sum();

        // 添加系统开销（约10%）
        (message_tokens as f32 * 1.1).ceil() as u32
    }

    /// 估算单个文件diff的token数量
    #[allow(dead_code)]
    pub fn estimate_file_diff_tokens(file_path: &str, diff_content: &str) -> u32 {
        let path_tokens = Self::estimate_tokens(file_path);
        let diff_tokens = Self::estimate_tokens(diff_content);

        // 文件路径 + diff内容 + 格式化开销
        path_tokens + diff_tokens + 50
    }

    /// 检查是否为中文字符
    fn is_chinese_char(c: char) -> bool {
        matches!(c, '\u{4e00}'..='\u{9fff}' | '\u{3400}'..='\u{4dbf}' | '\u{20000}'..='\u{2a6df}')
    }

    /// 检查token数量是否超出模型限制
    pub fn is_over_limit(estimated_tokens: u32, model_max_tokens: Option<u32>) -> bool {
        if let Some(max_tokens) = model_max_tokens {
            // 保留20%的余量用于响应生成
            let safe_limit = (max_tokens as f32 * 0.8) as u32;
            estimated_tokens > safe_limit
        } else {
            // 如果没有明确限制，使用通用的4k限制
            estimated_tokens > 3200
        }
    }

    /// 将大的diff内容分割为多个文件块
    #[allow(dead_code)]
    pub fn split_files_by_token_limit(
        files_with_diffs: Vec<(String, String)>,
        max_tokens_per_chunk: u32,
    ) -> Vec<Vec<(String, String)>> {
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_tokens = 0u32;

        for (file_path, diff_content) in files_with_diffs {
            let file_tokens = Self::estimate_file_diff_tokens(&file_path, &diff_content);

            // 如果单个文件就超过限制，单独成块
            if file_tokens > max_tokens_per_chunk {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = Vec::new();
                    current_tokens = 0;
                }
                chunks.push(vec![(file_path, diff_content)]);
                continue;
            }

            // 如果添加这个文件会超过限制，先保存当前块
            if current_tokens + file_tokens > max_tokens_per_chunk && !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = Vec::new();
                current_tokens = 0;
            }

            current_chunk.push((file_path, diff_content));
            current_tokens += file_tokens;
        }

        // 添加最后一个块
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        // 测试英文文本
        let english_text = "Hello world, this is a test message.";
        let tokens = TokenCounter::estimate_tokens(english_text);
        assert!(tokens > 0);

        // 测试中文文本
        let chinese_text = "你好世界，这是一个测试消息。";
        let chinese_tokens = TokenCounter::estimate_tokens(chinese_text);
        assert!(chinese_tokens > 0);

        // 中文token数量应该相对较高
        assert!(chinese_tokens > tokens / 2);
    }

    #[test]
    fn test_is_over_limit() {
        assert!(TokenCounter::is_over_limit(5000, Some(4096)));
        assert!(!TokenCounter::is_over_limit(2000, Some(4096)));
        assert!(TokenCounter::is_over_limit(4000, None)); // 使用默认限制
    }

    #[test]
    fn test_split_files_by_token_limit() {
        let files = vec![
            ("file1.txt".to_string(), "small diff".to_string()),
            ("file2.txt".to_string(), "another small diff".to_string()),
            ("file3.txt".to_string(), "yet another diff".to_string()),
        ];

        let chunks = TokenCounter::split_files_by_token_limit(files, 100);
        assert!(!chunks.is_empty());
    }
}
