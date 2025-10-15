use anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 模板版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVersion {
    /// 版本ID
    pub id: String,
    /// 版本号
    pub version: String,
    /// 版本名称/标签
    pub name: String,
    /// 版本描述
    pub description: String,
    /// 模板内容
    pub content: String,
    /// 创建时间
    pub created_at: String,
    /// 是否为系统内置版本
    pub is_builtin: bool,
    /// 父版本ID（用于跟踪版本历史）
    pub parent_id: Option<String>,
}

/// 模板配置（支持版本管理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfigWithVersions {
    /// 模板ID
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 模板类型
    pub template_type: String,
    /// 所有版本历史
    pub versions: Vec<TemplateVersion>,
    /// 当前使用的版本ID
    pub current_version_id: String,
    /// 创建时间
    pub created_at: String,
    /// 最后更新时间
    pub updated_at: String,
    /// 是否为用户自定义模板
    pub is_custom: bool,
    /// 原始系统模板ID（如果是基于系统模板修改的）
    pub original_template_id: Option<String>,
    /// 系统版本号（用于系统模板更新检测）
    pub system_version: Option<String>,
    /// 模板类别（用于区分不同类型的模板）
    pub template_category: Option<String>,
    /// 模板配置（如提交类型、表情符号等）
    pub template_config: Option<serde_json::Value>,
}

/// 模板更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateUpdateRequest {
    /// 模板ID
    pub template_id: String,
    /// 新的模板内容
    pub content: String,
    /// 版本名称
    pub version_name: String,
    /// 版本描述
    pub version_description: String,
    /// 是否创建新版本
    pub create_new_version: bool,
}

/// 模板版本切换请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVersionSwitchRequest {
    /// 模板ID
    pub template_id: String,
    /// 目标版本ID
    pub version_id: String,
}

/// 模板系统更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSystemUpdate {
    /// 系统模板ID
    pub system_template_id: String,
    /// 新版本号
    pub new_version: String,
    /// 更新内容描述
    pub update_description: String,
    /// 更新时间
    pub update_time: String,
    /// 是否需要用户确认
    pub requires_confirmation: bool,
}

impl TemplateVersion {
    /// 创建新的模板版本
    pub fn new(
        content: String,
        name: String,
        description: String,
        is_builtin: bool,
        parent_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            version: if is_builtin { "1.0.0" } else { "1.0.0" }.to_string(),
            name,
            description,
            content,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_builtin,
            parent_id,
        }
    }

    /// 创建系统内置版本
    pub fn builtin(content: String, name: String, description: String) -> Self {
        Self::new(content, name, description, true, None)
    }

    /// 创建用户自定义版本
    pub fn custom(
        content: String,
        name: String,
        description: String,
        parent_id: Option<String>,
    ) -> Self {
        Self::new(content, name, description, false, parent_id)
    }
}

impl TemplateConfigWithVersions {
    /// 创建新的模板配置
    pub fn new(
        name: String,
        description: String,
        template_type: String,
        initial_version: TemplateVersion,
    ) -> Self {
        let current_version_id = initial_version.id.clone();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            template_type,
            versions: vec![initial_version],
            current_version_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_custom: false,
            original_template_id: None,
            system_version: None,
            template_category: None,
            template_config: None,
        }
    }

    /// 创建用户自定义模板
    pub fn custom(
        name: String,
        description: String,
        template_type: String,
        initial_version: TemplateVersion,
        original_template_id: Option<String>,
    ) -> Self {
        let current_version_id = initial_version.id.clone();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            template_type,
            versions: vec![initial_version],
            current_version_id,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            is_custom: true,
            original_template_id,
            system_version: None,
            template_category: None,
            template_config: None,
        }
    }

    /// 获取当前版本
    pub fn get_current_version(&self) -> Option<&TemplateVersion> {
        self.versions
            .iter()
            .find(|v| v.id == self.current_version_id)
    }

    /// 获取当前版本的可变引用
    pub fn get_current_version_mut(&mut self) -> Option<&mut TemplateVersion> {
        self.versions
            .iter_mut()
            .find(|v| v.id == self.current_version_id)
    }

    /// 获取当前内容
    pub fn get_current_content(&self) -> Option<&str> {
        self.get_current_version().map(|v| v.content.as_str())
    }

    /// 添加新版本
    pub fn add_version(&mut self, version: TemplateVersion) -> anyhow::Result<()> {
        // 验证版本内容不为空
        if version.content.trim().is_empty() {
            return Err(anyhow::anyhow!("模板内容不能为空"));
        }

        self.versions.push(version);
        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// 切换到指定版本
    pub fn switch_to_version(&mut self, version_id: &str) -> anyhow::Result<()> {
        if self.versions.iter().any(|v| v.id == version_id) {
            self.current_version_id = version_id.to_string();
            self.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(anyhow::anyhow!("指定的版本不存在"))
        }
    }

    /// 获取版本历史
    pub fn get_version_history(&self) -> Vec<&TemplateVersion> {
        let mut versions: Vec<&TemplateVersion> = self.versions.iter().collect();
        // 按创建时间倒序排列
        versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        versions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_version_creation() {
        let version = TemplateVersion::builtin(
            "test content".to_string(),
            "v1.0".to_string(),
            "Initial version".to_string(),
        );

        assert!(!version.id.is_empty());
        assert_eq!(version.version, "1.0.0");
        assert_eq!(version.content, "test content");
        assert!(version.is_builtin);
    }

    #[test]
    fn test_template_config_creation() {
        let version = TemplateVersion::builtin(
            "test content".to_string(),
            "v1.0".to_string(),
            "Initial version".to_string(),
        );

        let config = TemplateConfigWithVersions::new(
            "Test Template".to_string(),
            "Test description".to_string(),
            "daily_summary".to_string(),
            version,
        );

        assert_eq!(config.name, "Test Template");
        assert_eq!(config.versions.len(), 1);
        assert_eq!(config.current_version_id, config.versions[0].id);
    }

    #[test]
    fn test_version_switching() {
        let version1 = TemplateVersion::builtin(
            "content1".to_string(),
            "v1.0".to_string(),
            "First version".to_string(),
        );

        let mut config = TemplateConfigWithVersions::new(
            "Test Template".to_string(),
            "Test description".to_string(),
            "daily_summary".to_string(),
            version1,
        );

        let version2 = TemplateVersion::builtin(
            "content2".to_string(),
            "v2.0".to_string(),
            "Second version".to_string(),
        );

        config.add_version(version2).unwrap();
        let version2_id = config.versions[1].id.clone();

        assert!(config.switch_to_version(&version2_id).is_ok());
        assert_eq!(config.current_version_id, version2_id);
        assert_eq!(config.get_current_content(), Some("content2"));
    }
}
