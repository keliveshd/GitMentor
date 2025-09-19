use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/**
 * AI响应缓存管理器
 * 用于缓存相同文件变更的AI响应，减少token消耗
 * 作者：Evilek
 * 编写日期：2025-01-19
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub timestamp: u64,
    pub model: String,
    pub template_id: String,
}

#[derive(Debug)]
pub struct AIResponseCache {
    cache_dir: PathBuf,
    cache: HashMap<String, CacheEntry>,
    max_age_seconds: u64,
}

impl AIResponseCache {
    /// 创建新的缓存实例
    pub fn new(cache_dir: PathBuf) -> Self {
        let mut cache = Self {
            cache_dir,
            cache: HashMap::new(),
            max_age_seconds: 3600 * 24, // 默认缓存24小时
        };

        // 加载现有缓存
        if let Err(e) = cache.load_cache() {
            eprintln!("⚠️ [Cache] 加载缓存失败: {}", e);
        }

        cache
    }

    /// 设置缓存过期时间（秒）
    pub fn set_max_age(&mut self, seconds: u64) {
        self.max_age_seconds = seconds;
    }

    /// 生成缓存键
    fn generate_cache_key(
        &self,
        template_id: &str,
        model: &str,
        file_changes: &[(String, String)], // (文件路径, 差异内容)
        repository_path: Option<&str>,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 哈希模板ID
        template_id.hash(&mut hasher);

        // 哈希模型名称
        model.hash(&mut hasher);

        // 哈希仓库路径
        if let Some(repo_path) = repository_path {
            repo_path.hash(&mut hasher);
        }

        // 哈希文件变更
        for (file_path, diff) in file_changes {
            file_path.hash(&mut hasher);
            // 只哈希差异的前1000个字符，避免过长的哈希计算
            let diff_preview = if diff.len() > 1000 {
                &diff[..1000]
            } else {
                diff
            };
            diff_preview.hash(&mut hasher);
        }

        format!("cache_{:x}", hasher.finish())
    }

    /// 获取缓存响应
    pub fn get(
        &self,
        template_id: &str,
        model: &str,
        file_changes: &[(String, String)],
        repository_path: Option<&str>,
    ) -> Option<CacheEntry> {
        let key = self.generate_cache_key(template_id, model, file_changes, repository_path);

        if let Some(entry) = self.cache.get(&key) {
            // 检查是否过期
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now - entry.timestamp <= self.max_age_seconds {
                eprintln!("✅ [Cache] 命中缓存: {}", template_id);
                Some(entry.clone())
            } else {
                eprintln!("⏰ [Cache] 缓存已过期: {}", template_id);
                None
            }
        } else {
            None
        }
    }

    /// 设置缓存响应
    pub fn set(
        &mut self,
        template_id: &str,
        model: &str,
        file_changes: &[(String, String)],
        repository_path: Option<&str>,
        content: String,
        reasoning_content: Option<String>,
    ) -> Result<()> {
        let key = self.generate_cache_key(template_id, model, file_changes, repository_path);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = CacheEntry {
            content,
            reasoning_content,
            timestamp: now,
            model: model.to_string(),
            template_id: template_id.to_string(),
        };

        self.cache.insert(key, entry.clone());

        // 异步保存到文件
        let cache_file = self.cache_dir.join("ai_response_cache.json");
        let cache_data = serde_json::to_string(&self.cache)?;

        tokio::spawn(async move {
            if let Err(e) = fs::write(&cache_file, cache_data) {
                eprintln!("❌ [Cache] 保存缓存失败: {}", e);
            }
        });

        eprintln!("💾 [Cache] 缓存响应: {}", template_id);
        Ok(())
    }

    /// 加载缓存文件
    fn load_cache(&mut self) -> Result<()> {
        let cache_file = self.cache_dir.join("ai_response_cache.json");

        if !cache_file.exists() {
            return Ok(());
        }

        let cache_data = fs::read_to_string(&cache_file)?;
        let loaded_cache: HashMap<String, CacheEntry> = serde_json::from_str(&cache_data)?;

        // 清理过期缓存
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.cache = loaded_cache
            .into_iter()
            .filter(|(_, entry)| now - entry.timestamp <= self.max_age_seconds)
            .collect();

        eprintln!("✅ [Cache] 加载了 {} 个缓存项", self.cache.len());
        Ok(())
    }

    /// 清理所有缓存
    pub fn clear_all(&mut self) -> Result<()> {
        self.cache.clear();

        let cache_file = self.cache_dir.join("ai_response_cache.json");
        if cache_file.exists() {
            fs::remove_file(&cache_file)?;
        }

        eprintln!("🧹 [Cache] 已清理所有缓存");
        Ok(())
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), self.cache.len() as u64);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let valid_entries = self.cache
            .values()
            .filter(|entry| now - entry.timestamp <= self.max_age_seconds)
            .count() as u64;

        stats.insert("valid_entries".to_string(), valid_entries);
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_operations() {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let mut cache = AIResponseCache::new(cache_dir.clone());

        // 测试缓存设置和获取
        let file_changes = vec![
            ("src/main.rs".to_string(), "println!(\"Hello\");".to_string()),
        ];

        cache.set(
            "test_template",
            "gpt-4",
            &file_changes,
            Some("/test/repo"),
            "Test response".to_string(),
            None,
        ).unwrap();

        let cached = cache.get(
            "test_template",
            "gpt-4",
            &file_changes,
            Some("/test/repo"),
        );

        assert!(cached.is_some());
        assert_eq!(cached.unwrap().content, "Test response");

        // 测试清理缓存
        cache.clear_all().unwrap();
        assert_eq!(cache.cache.len(), 0);
    }
}