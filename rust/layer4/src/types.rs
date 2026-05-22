//! # Layer 4 Core Types
//!
//! Layer 4 使用的核心类型定义。

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Layer 4 统一 Result 类型
pub type Layer4Result<T> = anyhow::Result<T>;

/// Layer 4 错误类型
#[derive(Debug, Error)]
pub enum Layer4Error {
    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Worktree error: {0}")]
    Worktree(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// 集成层配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub plugin_dir: String,
    pub worktrees_dir: String,
    pub channel_timeout_ms: u64,
    pub max_plugins: usize,
    pub max_worktrees: usize,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            plugin_dir: "~/.continuum/plugins".to_string(),
            worktrees_dir: ".claude/worktrees".to_string(),
            channel_timeout_ms: 30000,
            max_plugins: 50,
            max_worktrees: 10,
        }
    }
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MessagePriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

/// 插件权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub can_read_files: bool,
    pub can_write_files: bool,
    pub can_execute_commands: bool,
    pub can_access_network: bool,
    pub allowed_paths: Vec<String>,
    pub allowed_commands: Vec<String>,
}

impl Default for PluginPermission {
    fn default() -> Self {
        Self {
            can_read_files: true,
            can_write_files: false,
            can_execute_commands: false,
            can_access_network: false,
            allowed_paths: Vec::new(),
            allowed_commands: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_config_default() {
        let config = IntegrationConfig::default();
        assert_eq!(config.max_plugins, 50);
        assert_eq!(config.max_worktrees, 10);
    }

    #[test]
    fn test_plugin_permission_default() {
        let perm = PluginPermission::default();
        assert!(perm.can_read_files);
        assert!(!perm.can_write_files);
    }

    #[test]
    fn test_message_priority_default() {
        let priority = MessagePriority::default();
        assert_eq!(priority, MessagePriority::Normal);
    }
}
