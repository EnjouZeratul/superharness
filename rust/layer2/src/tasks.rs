//! # Task Manager
//!
//! 任务队列管理，支持优先级和依赖关系。

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::types::{AgentState, Layer2Error, Layer2Result, TaskId};

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub dependencies: Vec<TaskId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub timeout: Option<Duration>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Task {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: TaskId::new(),
            name: name.into(),
            description: description.into(),
            status: TaskStatus::Pending,
            priority: TaskPriority::Normal,
            dependencies: Vec::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            timeout: None,
            retry_count: 0,
            max_retries: 3,
            metadata: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_dependency(mut self, task_id: TaskId) -> Self {
        self.dependencies.push(task_id);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// 检查是否可以执行（所有依赖已完成）
    pub fn can_execute(&self, completed: &HashMap<TaskId, TaskStatus>) -> bool {
        self.dependencies
            .iter()
            .all(|dep_id| completed.get(dep_id) == Some(&TaskStatus::Completed))
    }

    /// 获取执行时长
    pub fn duration(&self) -> Option<Duration> {
        self.started_at.and_then(|start| {
            self.completed_at
                .map(|end| Duration::from_secs((end - start).num_seconds() as u64))
        })
    }
}

impl Eq for Task {}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 优先级高的排在前面
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// 任务管理器接口
#[async_trait]
pub trait TaskManagerTrait: Send + Sync {
    /// 添加任务
    fn add(&self, task: Task) -> Layer2Result<TaskId>;

    /// 获取任务
    fn get(&self, id: &TaskId) -> Option<Task>;

    /// 更新任务状态
    async fn update_status(&self, id: &TaskId, status: TaskStatus) -> Layer2Result<bool>;

    /// 取消任务
    async fn cancel(&self, id: &TaskId) -> Layer2Result<bool>;

    /// 获取下一个可执行任务
    fn next(&self) -> Option<Task>;

    /// 获取任务数量
    fn count(&self) -> usize;

    /// 获取特定状态的任务数量
    fn count_by_status(&self, status: TaskStatus) -> usize;

    /// 清理已完成任务
    fn cleanup_completed(&self) -> usize;
}

/// 任务管理器实现
pub struct TaskManager {
    tasks: RwLock<HashMap<TaskId, Task>>,
    queue: RwLock<BinaryHeap<Task>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            queue: RwLock::new(BinaryHeap::new()),
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskManagerTrait for TaskManager {
    fn add(&self, task: Task) -> Layer2Result<TaskId> {
        let id = task.id.clone();

        self.queue.write().push(task.clone());
        self.tasks.write().insert(id.clone(), task);

        Ok(id)
    }

    fn get(&self, id: &TaskId) -> Option<Task> {
        self.tasks.read().get(id).cloned()
    }

    async fn update_status(&self, id: &TaskId, status: TaskStatus) -> Layer2Result<bool> {
        let mut tasks = self.tasks.write();

        if let Some(task) = tasks.get_mut(id) {
            task.status = status;

            if status == TaskStatus::Running {
                task.started_at = Some(chrono::Utc::now());
            } else if matches!(status, TaskStatus::Completed | TaskStatus::Failed) {
                task.completed_at = Some(chrono::Utc::now());
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn cancel(&self, id: &TaskId) -> Layer2Result<bool> {
        self.update_status(id, TaskStatus::Cancelled).await
    }

    fn next(&self) -> Option<Task> {
        let tasks = self.tasks.read();
        let completed: HashMap<TaskId, TaskStatus> = tasks
            .iter()
            .filter(|(_, t)| t.status == TaskStatus::Completed)
            .map(|(id, t)| (id.clone(), t.status))
            .collect();

        self.queue
            .write()
            .pop()
            .filter(|t| t.can_execute(&completed))
    }

    fn count(&self) -> usize {
        self.tasks.read().len()
    }

    fn count_by_status(&self, status: TaskStatus) -> usize {
        self.tasks
            .read()
            .values()
            .filter(|t| t.status == status)
            .count()
    }

    fn cleanup_completed(&self) -> usize {
        let mut tasks = self.tasks.write();
        let completed: Vec<TaskId> = tasks
            .iter()
            .filter(|(_, t)| t.status == TaskStatus::Completed)
            .map(|(id, _)| id.clone())
            .collect();

        let count = completed.len();
        for id in completed {
            tasks.remove(&id);
        }

        // 重建队列
        let mut queue = self.queue.write();
        *queue = tasks.values().cloned().collect();

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("test", "Test task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_task_priority() {
        let task = Task::new("test", "Test").with_priority(TaskPriority::High);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_task_manager() {
        let manager = TaskManager::new();
        assert_eq!(manager.count(), 0);

        let task = Task::new("test", "Test task");
        manager.add(task).unwrap();

        assert_eq!(manager.count(), 1);
    }
}
