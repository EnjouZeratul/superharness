//! # Workflow Engine
//!
//! DAG 工作流引擎，支持节点依赖和并行执行。

mod dag;
mod executor;
mod node;

pub use dag::Dag;
pub use executor::WorkflowExecutor;
pub use node::{Node, NodeStatus};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::types::{Layer2Result, TaskId};

/// 工作流输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    pub task: String,
    pub context: serde_json::Value,
}

/// 工作流输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOutput {
    pub task_id: TaskId,
    pub results: Vec<NodeResult>,
    pub status: WorkflowStatus,
    pub duration_ms: u64,
}

/// 节点执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    pub node_id: String,
    pub status: NodeStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// 工作流状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 工作流引擎接口
#[async_trait]
pub trait WorkflowEngineTrait: Send + Sync {
    /// 添加节点
    fn add_node(&mut self, node: Node) -> Layer2Result<()>;

    /// 添加边（依赖关系）
    fn add_edge(&mut self, from: &str, to: &str) -> Layer2Result<()>;

    /// 执行工作流
    async fn execute(&self, input: WorkflowInput) -> Layer2Result<WorkflowOutput>;

    /// 取消工作流
    async fn cancel(&self, task_id: &TaskId) -> Layer2Result<bool>;

    /// 获取工作流状态
    fn status(&self, task_id: &TaskId) -> Layer2Result<WorkflowStatus>;

    /// 验证 DAG 结构
    fn validate(&self) -> Layer2Result<Vec<String>>;

    /// 获取节点数量
    fn node_count(&self) -> usize;

    /// 获取边数量
    fn edge_count(&self) -> usize;
}

/// 节点执行器接口
#[async_trait]
pub trait NodeExecutor: Send + Sync {
    /// 执行节点
    async fn execute(&self, node: &Node, input: &WorkflowInput) -> Layer2Result<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_input_creation() {
        let input = WorkflowInput {
            task: "test".to_string(),
            context: serde_json::Value::Null,
        };
        assert_eq!(input.task, "test");
    }
}
