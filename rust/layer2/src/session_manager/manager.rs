//! # Concurrent Session Manager
//!
//! 并发安全的会话管理器实现。

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::session_manager::{
    ExecutionContext, ReadWriteLock, Session, SessionConfig, SessionManagerTrait, SessionStats,
};
use crate::types::{AgentState, Layer2Error, Layer2Result, Message, SessionId, SessionMeta};

/// 会话锁包装
struct SessionLock {
    session: Session,
    lock: ReadWriteLock,
}

/// 并发安全会话管理器
///
/// 使用读写分离锁，读操作可并发，写操作互斥。
pub struct ConcurrentSessionManager {
    sessions: RwLock<HashMap<SessionId, SessionLock>>,
    max_sessions: usize,
}

impl ConcurrentSessionManager {
    /// 创建新的会话管理器
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            max_sessions,
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(100)
    }

    /// 获取会话锁
    fn get_session_lock(&self, id: &SessionId) -> Option<SessionLock> {
        let guard = self.sessions.read();
        guard.get(id).map(|s| SessionLock {
            session: s.session.clone(),
            lock: ReadWriteLock::new(), // 每次返回新的锁实例
        })
    }
}

impl Default for ConcurrentSessionManager {
    fn default() -> Self {
        Self::default_config()
    }
}

#[async_trait]
impl SessionManagerTrait for ConcurrentSessionManager {
    async fn create(&self, config: SessionConfig) -> Layer2Result<SessionId> {
        let mut sessions = self.sessions.write();

        if sessions.len() >= self.max_sessions {
            return Err(Layer2Error::SessionNotFound(SessionId::new())) // TODO: 适当的错误
                .map_err(|e| anyhow::anyhow!("Max sessions limit reached: {}", self.max_sessions));
        }

        let session = Session::new(&config);
        let session_id = session.session_id.clone();

        sessions.insert(
            session_id.clone(),
            SessionLock {
                session,
                lock: ReadWriteLock::new(),
            },
        );

        Ok(session_id)
    }

    async fn get(&self, id: &SessionId) -> Layer2Result<Option<Session>> {
        let sessions = self.sessions.read();
        Ok(sessions.get(id).map(|s| s.session.clone()))
    }

    async fn get_or_create(
        &self,
        id: Option<&SessionId>,
        config: SessionConfig,
    ) -> Layer2Result<SessionId> {
        let mut sessions = self.sessions.write();

        // 如果指定了 ID 且存在，直接返回
        if let Some(session_id) = id {
            if sessions.contains_key(session_id) {
                return Ok(session_id.clone());
            }
        }

        // 检查限制
        if sessions.len() >= self.max_sessions {
            return Err(anyhow::anyhow!("Max sessions limit reached"));
        }

        // 创建新会话
        let session = Session::new(&config);
        let session_id = session.session_id.clone();

        // 如果指定了 ID，使用指定的 ID
        let final_id = id.cloned().unwrap_or_else(|| session_id.clone());

        sessions.insert(
            final_id.clone(),
            SessionLock {
                session,
                lock: ReadWriteLock::new(),
            },
        );

        Ok(final_id)
    }

    async fn save(&self, session: &Session) -> Layer2Result<()> {
        let mut sessions = self.sessions.write();

        if let Some(session_lock) = sessions.get_mut(&session.session_id) {
            session_lock.session = session.clone();
            session_lock.session.touch();
        }

        Ok(())
    }

    async fn delete(&self, id: &SessionId) -> Layer2Result<bool> {
        let mut sessions = self.sessions.write();
        Ok(sessions.remove(id).is_some())
    }

    async fn list(&self) -> Layer2Result<Vec<SessionMeta>> {
        let sessions = self.sessions.read();
        Ok(sessions
            .values()
            .map(|s| SessionMeta {
                session_id: s.session.session_id.clone(),
                agent_id: s.session.agent_id.clone(),
                state: s.session.state,
                created_at: s.session.created_at,
                last_updated: s.session.last_updated,
                message_count: s.session.messages.len(),
                checkpoint_count: s.session.checkpoint_count,
            })
            .collect())
    }

    async fn update<F>(&self, id: &SessionId, update_fn: F) -> Layer2Result<bool>
    where
        F: FnOnce(&mut Session) + Send,
    {
        let mut sessions = self.sessions.write();

        if let Some(session_lock) = sessions.get_mut(id) {
            session_lock.lock.write(|| {
                update_fn(&mut session_lock.session);
                session_lock.session.touch();
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn read<F, T>(&self, id: &SessionId, read_fn: F) -> Layer2Result<Option<T>>
    where
        F: FnOnce(&Session) -> T + Send,
        T: Send,
    {
        let sessions = self.sessions.read();

        if let Some(session_lock) = sessions.get(id) {
            // 使用读锁
            let result = session_lock.lock.read(|| read_fn(&session_lock.session));
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn get_state(&self, id: &SessionId) -> Layer2Result<Option<AgentState>> {
        self.read(id, |s| s.state).await
    }

    async fn set_state(&self, id: &SessionId, state: AgentState) -> Layer2Result<bool> {
        self.update(id, |s| s.state = state).await
    }

    async fn add_message(&self, id: &SessionId, message: Message) -> Layer2Result<bool> {
        self.update(id, |s| {
            s.messages.push(message);
            s.iteration += 1;
        })
        .await
    }

    async fn get_messages(&self, id: &SessionId) -> Layer2Result<Option<Vec<Message>>> {
        self.read(id, |s| s.messages.clone()).await
    }

    fn stats(&self) -> SessionStats {
        let sessions = self.sessions.read();
        SessionStats {
            total_sessions: sessions.len(),
            max_sessions: self.max_sessions,
            active_sessions: sessions
                .values()
                .filter(|s| matches!(s.session.state, AgentState::Running))
                .count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let manager = ConcurrentSessionManager::default_config();
        let config = SessionConfig::default();

        let session_id = manager.create(config).await.unwrap();
        assert!(!session_id.0.is_empty());
    }

    #[tokio::test]
    async fn test_get_session() {
        let manager = ConcurrentSessionManager::default_config();
        let config = SessionConfig::default();

        let session_id = manager.create(config).await.unwrap();
        let session = manager.get(&session_id).await.unwrap();

        assert!(session.is_some());
        assert_eq!(session.unwrap().session_id, session_id);
    }

    #[tokio::test]
    async fn test_update_session() {
        let manager = ConcurrentSessionManager::default_config();
        let config = SessionConfig::default();

        let session_id = manager.create(config).await.unwrap();

        manager
            .update(&session_id, |s| {
                s.add_user_message("Hello");
            })
            .await
            .unwrap();

        let messages = manager.get_messages(&session_id).await.unwrap().unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let manager = ConcurrentSessionManager::default_config();
        let config = SessionConfig::default();

        let session_id = manager.create(config).await.unwrap();
        let deleted = manager.delete(&session_id).await.unwrap();

        assert!(deleted);

        let session = manager.get(&session_id).await.unwrap();
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn test_session_stats() {
        let manager = ConcurrentSessionManager::new(10);

        let config = SessionConfig::default();
        manager.create(config).await.unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.max_sessions, 10);
    }
}
