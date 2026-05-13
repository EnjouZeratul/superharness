//! # Built-in Tools
//!
//! 内置工具集：提供 40+ 常用工具的实现。

pub mod adapter;
pub mod code;
pub mod file_ops;
pub mod memory_tools;
pub mod network;
pub mod search;
pub mod shell;
pub mod workflow_tools;

// Re-export adapter for Layer 2 integration
pub use adapter::{register_builtin_tools, ToolAdapter};

use crate::types::{Layer3Result, ToolCategory, ToolMeta};
use async_trait::async_trait;
use std::collections::HashMap;

/// 内置工具 trait
///
/// 所有内置工具必须实现此 trait。
#[async_trait]
pub trait BuiltinTool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 参数 JSON Schema
    fn parameters_schema(&self) -> serde_json::Value;

    /// 工具分类
    fn category(&self) -> ToolCategory;

    /// 是否需要用户确认
    fn requires_confirmation(&self) -> bool {
        false
    }

    /// 是否为危险操作
    fn is_dangerous(&self) -> bool {
        false
    }

    /// 执行工具
    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String>;

    /// 获取元数据
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters_schema(),
            requires_confirmation: self.requires_confirmation(),
            is_dangerous: self.is_dangerous(),
            category: self.category(),
        }
    }
}

/// 内置工具注册表
///
/// 管理所有内置工具的注册和查找。
pub struct BuiltinToolRegistry {
    tools: HashMap<String, Box<dyn BuiltinTool>>,
}

impl BuiltinToolRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 创建并注册所有默认工具
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        // 注册将在各模块实现后添加
        registry
    }

    /// 注册工具
    pub fn register(&mut self, tool: Box<dyn BuiltinTool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&dyn BuiltinTool> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    /// 列出所有工具
    pub fn list(&self) -> Vec<&dyn BuiltinTool> {
        self.tools.values().map(|b| b.as_ref()).collect()
    }

    /// 列出所有工具元数据
    pub fn list_meta(&self) -> Vec<ToolMeta> {
        self.tools.values().map(|t| t.meta()).collect()
    }

    /// 按分类列出工具
    pub fn list_by_category(&self, category: ToolCategory) -> Vec<&dyn BuiltinTool> {
        self.tools
            .values()
            .filter(|t| t.category() == category)
            .map(|b| b.as_ref())
            .collect()
    }
}

impl Default for BuiltinToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具分类快捷常量
pub const FILE_OPS_TOOLS: &[&str] = &[
    "read_file",
    "write_file",
    "edit_file",
    "create_file",
    "delete_file",
    "list_directory",
    "copy_file",
    "move_file",
];

pub const SEARCH_TOOLS: &[&str] = &["grep", "glob", "find_in_files", "search_content"];

pub const SHELL_TOOLS: &[&str] = &["bash", "run_command", "shell_exec"];

pub const CODE_TOOLS: &[&str] = &[
    "go_to_definition",
    "find_references",
    "get_hover",
    "list_symbols",
];

pub const MEMORY_TOOLS: &[&str] = &["save_memory", "load_memory", "query_memory", "clear_memory"];

pub const WORKFLOW_TOOLS: &[&str] = &["create_checkpoint", "restore_checkpoint", "create_subtask"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = BuiltinToolRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_tool_categories() {
        assert!(!FILE_OPS_TOOLS.is_empty());
        assert!(!SEARCH_TOOLS.is_empty());
    }
}
