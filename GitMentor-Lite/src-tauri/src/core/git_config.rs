use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/**
 * Git配置管理
 * 作者：Evilek
 * 编写日期：2025-08-07
 */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GitExecutionMode {
    /// 自动检测（默认）
    Auto,
    /// 强制使用系统Git
    SystemGit,
    /// 强制使用内置Git
    BundledGit,
    /// 强制使用Git2库API
    Git2Api,
}

impl Default for GitExecutionMode {
    fn default() -> Self {
        GitExecutionMode::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Git执行方式
    pub execution_mode: GitExecutionMode,
    /// 自定义Git路径（当使用SystemGit时）
    pub custom_git_path: Option<String>,
    /// 启用详细日志
    pub enable_verbose_logging: bool,
    /// Git命令超时时间（秒）
    pub command_timeout: u64,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            execution_mode: GitExecutionMode::Auto,
            custom_git_path: None,
            enable_verbose_logging: false,
            command_timeout: 30,
        }
    }
}

pub struct GitConfigManager {
    config: GitConfig,
    config_path: PathBuf,
}

impl GitConfigManager {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            GitConfig::default()
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    pub fn get_config(&self) -> &GitConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: GitConfig) -> Result<()> {
        self.config = config;
        self.save()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// 获取Git执行方式的显示名称
    pub fn get_execution_mode_display_name(mode: &GitExecutionMode) -> &'static str {
        match mode {
            GitExecutionMode::Auto => "自动检测",
            GitExecutionMode::SystemGit => "系统Git",
            GitExecutionMode::BundledGit => "内置Git",
            GitExecutionMode::Git2Api => "Git2库API",
        }
    }

    /// 获取所有可用的执行方式
    pub fn get_available_modes() -> Vec<(GitExecutionMode, &'static str)> {
        vec![
            (GitExecutionMode::Auto, "自动检测（推荐）"),
            (GitExecutionMode::SystemGit, "系统Git命令"),
            (GitExecutionMode::BundledGit, "内置Git可执行文件"),
            (GitExecutionMode::Git2Api, "Git2库API（功能受限）"),
        ]
    }
}
