//! # Session Manager
//!
//! 会话管理模块，提供并发安全的会话存储和生命周期管理。
//!
//! ## 模块结构
//! - `manager`: ConcurrentSessionManager 实现
//! - `session`: Session 和 ExecutionContext 定义
//! - `lock`: ReadWriteLock 实现
//! - `context`: 执行上下文

mod context;
mod lock;
mod manager;
mod session;

pub use context::ExecutionContext;
pub use lock::ReadWriteLock;
pub use manager::ConcurrentSessionManager;
pub use session::{Session, SessionConfig};

use async_trait::async_trait;

use crate::types::{AgentState, Layer2Result, Message, SessionId, SessionMeta};

/// 会话管理器接口
///
/// 定义会话生命周期的核心操作。
#[async_trait]
pub trait SessionManagerTrait: Send + Sync {
    /// 创建新会话
    ///
    /// # Arguments
    /// * `config` - 会话配置
    ///
    /// # Returns
    /// 新创建的会话 ID
    async fn create(&self, config: SessionConfig) -> Layer2Result<SessionId>;

    /// 获取会话
    ///
    /// # Arguments
    /// * `id` - 会话 ID
    ///
    /// # Returns
    /// 会话对象（只读）
    async fn get(&self, id: &SessionId) -> Layer2Result<Option<Session>>;

    /// 获取或创建会话
    ///
    /// 如果会话存在则返回，否则创建新会话。
    ///
    /// # Arguments
    /// * `id` - 会话 ID（可选）
    /// * `config` - 创建新会话时的配置
    async fn get_or_create(
        &self,
        id: Option<&SessionId>,
        config: SessionConfig,
    ) -> Layer2Result<SessionId>;

    /// 保存会话状态
    ///
    /// # Arguments
    /// * `session` - 会话对象
    async fn save(&self, session: &Session) -> Layer2Result<()>;

    /// 删除会话
    ///
    /// # Arguments
    /// * `id` - 会话 ID
    async fn delete(&self, id: &SessionId) -> Layer2Result<bool>;

    /// 列出所有会话
    ///
    /// # Returns
    /// 会话元数据列表
    async fn list(&self) -> Layer2Result<Vec<SessionMeta>>;

    /// 更新会话状态（带写锁）
    ///
    /// # Arguments
    /// * `id` - 会话 ID
    /// * `update_fn` - 更新函数
    async fn update<F>(&self, id: &SessionId, update_fn: F) -> Layer2Result<bool>
    where
        F: FnOnce(&mut Session) + Send;

    /// 读取会话状态（带读锁）
    ///
    /// # Arguments
    /// * `id` - 会话 ID
    /// * `read_fn` - 读取函数
    async fn read<F, T>(&self, id: &SessionId, read_fn: F) -> Layer2Result<Option<T>>
    where
        F: FnOnce(&Session) -> T + Send,
        T: Send;

    /// 获取会话状态
    async fn get_state(&self, id: &SessionId) -> Layer2Result<Option<AgentState>>;

    /// 设置会话状态
    async fn set_state(&self, id: &SessionId, state: AgentState) -> Layer2Result<bool>;

    /// 添加消息到会话
    async fn add_message(&self, id: &SessionId, message: Message) -> Layer2Result<bool>;

    /// 获取会话消息列表
    async fn get_messages(&self, id: &SessionId) -> Layer2Result<Option<Vec<Message>>>;

    /// 获取统计信息
    fn stats(&self) -> SessionStats;
}

/// 会话统计信息
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub max_sessions: usize,
    pub active_sessions: usize,
}

/// 简化版状态锁接口（当读写分离不是瓶颈时使用）
pub trait StateLockTrait: Send + Sync {
    /// 获取读锁
    fn read_lock<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T;

    /// 获取写锁
    fn write_lock<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_stats_default() {
        let stats = SessionStats::default();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.active_sessions, 0);
    }
}
