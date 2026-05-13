//! # Workflow Executor
//!
//! 工作流执行器实现。

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::types::{Layer2Result, TaskId};

use super::{
    Dag, Node, NodeExecutor, NodeResult, NodeStatus, WorkflowEngineTrait, WorkflowInput,
    WorkflowOutput, WorkflowStatus,
};

/// 工作流执行器
pub struct WorkflowExecutor {
    dag: RwLock<Dag>,
    task_status: RwLock<HashMap<TaskId, WorkflowStatus>>,
    node_executors: RwLock<HashMap<String, Arc<dyn NodeExecutor>>>,
}

impl WorkflowExecutor {
    pub fn new() -> Self {
        Self {
            dag: RwLock::new(Dag::new()),
            task_status: RwLock::new(HashMap::new()),
            node_executors: RwLock::new(HashMap::new()),
        }
    }

    /// 注册节点执行器
    pub fn register_executor(&self, node_type: &str, executor: Arc<dyn NodeExecutor>) {
        self.node_executors
            .write()
            .insert(node_type.to_string(), executor);
    }

    /// 获取节点和执行器信息（不持有锁）
    fn get_node_info(
        &self,
        node_id: &str,
    ) -> Option<(Node, Option<Arc<dyn NodeExecutor>>, String)> {
        let dag = self.dag.read();
        let node = dag.get_node(node_id)?;
        let node_type = node.node_type.clone();
        let node_clone = node.clone();

        drop(dag);

        let executors = self.node_executors.read();
        let executor = executors.get(&node_type).cloned();
        drop(executors);

        Some((node_clone, executor, node_type))
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowEngineTrait for WorkflowExecutor {
    fn add_node(&mut self, node: Node) -> Layer2Result<()> {
        self.dag.write().add_node(node)
    }

    fn add_edge(&mut self, from: &str, to: &str) -> Layer2Result<()> {
        self.dag.write().add_edge(from, to)
    }

    async fn execute(&self, input: WorkflowInput) -> Layer2Result<WorkflowOutput> {
        let task_id = TaskId::new();
        let start = Instant::now();

        // 设置状态为运行中
        self.task_status
            .write()
            .insert(task_id.clone(), WorkflowStatus::Running);

        // 获取排序后的节点列表（释放锁后再执行）
        let sorted_nodes = {
            let dag = self.dag.read();
            dag.topological_sort()?
        };

        let mut results = Vec::new();

        for node_id in sorted_nodes {
            // 获取节点信息（不持有锁）
            if let Some((node, executor, node_type)) = self.get_node_info(&node_id) {
                let node_start = Instant::now();

                let (status, output, error) = if let Some(exec) = executor {
                    match exec.execute(&node, &input).await {
                        Ok(out) => (NodeStatus::Completed, Some(out), None),
                        Err(e) => (NodeStatus::Failed, None, Some(e.to_string())),
                    }
                } else {
                    (
                        NodeStatus::Skipped,
                        None,
                        Some(format!("No executor for node type: {}", node_type)),
                    )
                };

                results.push(NodeResult {
                    node_id: node_id.clone(),
                    status,
                    output,
                    error,
                    duration_ms: node_start.elapsed().as_millis() as u64,
                });
            }
        }

        let final_status = if results.iter().all(|r| r.status == NodeStatus::Completed) {
            WorkflowStatus::Completed
        } else if results.iter().any(|r| r.status == NodeStatus::Failed) {
            WorkflowStatus::Failed
        } else {
            WorkflowStatus::Completed
        };

        self.task_status
            .write()
            .insert(task_id.clone(), final_status);

        Ok(WorkflowOutput {
            task_id,
            results,
            status: final_status,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn cancel(&self, task_id: &TaskId) -> Layer2Result<bool> {
        let mut status = self.task_status.write();
        if let Some(s) = status.get_mut(task_id) {
            *s = WorkflowStatus::Cancelled;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn status(&self, task_id: &TaskId) -> Layer2Result<WorkflowStatus> {
        let status = self.task_status.read();
        status
            .get(task_id)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))
    }

    fn validate(&self) -> Layer2Result<Vec<String>> {
        let dag = self.dag.read();
        let mut errors = Vec::new();

        if dag.has_cycle() {
            errors.push("DAG contains cycle".to_string());
        }

        Ok(errors)
    }

    fn node_count(&self) -> usize {
        self.dag.read().node_count()
    }

    fn edge_count(&self) -> usize {
        self.dag.read().edge_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = WorkflowExecutor::new();
        assert_eq!(executor.node_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut executor = WorkflowExecutor::new();
        let node = Node::new("test", "Test");
        executor.add_node(node).unwrap();
        assert_eq!(executor.node_count(), 1);
    }
}
