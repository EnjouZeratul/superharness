//! # Node Definition
//!
//! 工作流节点定义。

use serde::{Deserialize, Serialize};

/// 节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NodeStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// 工作流节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 节点 ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 节点类型
    #[serde(default = "default_node_type")]
    pub node_type: String,
    /// 节点配置
    #[serde(default)]
    pub config: serde_json::Value,
    /// 依赖的节点 ID
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// 超时时间（毫秒）
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// 重试次数
    #[serde(default)]
    pub retry_count: u32,
}

fn default_node_type() -> String {
    "task".to_string()
}

fn default_timeout() -> u64 {
    30000
}

impl Node {
    /// 创建新节点
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: default_node_type(),
            config: serde_json::Value::Null,
            dependencies: Vec::new(),
            timeout_ms: default_timeout(),
            retry_count: 0,
        }
    }

    /// 设置节点类型
    pub fn with_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_type = node_type.into();
        self
    }

    /// 设置配置
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }

    /// 添加依赖
    pub fn depends_on(mut self, node_id: &str) -> Self {
        self.dependencies.push(node_id.to_string());
        self
    }

    /// 设置超时
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// 设置重试次数
    pub fn with_retry(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("test", "Test Node");
        assert_eq!(node.id, "test");
        assert_eq!(node.name, "Test Node");
    }

    #[test]
    fn test_node_builder() {
        let node = Node::new("test", "Test Node")
            .with_type("custom")
            .with_timeout(60000)
            .depends_on("dep1");

        assert_eq!(node.node_type, "custom");
        assert_eq!(node.timeout_ms, 60000);
        assert_eq!(node.dependencies.len(), 1);
    }
}
