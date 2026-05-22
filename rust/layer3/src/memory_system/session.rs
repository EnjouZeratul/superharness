//! # Session Memory
//!
//! 会话记忆：单次会话内的持久化存储。

use crate::memory_system::{DecayPolicy, MemoryStore, TimeBasedDecay};
use crate::types::{Layer3Result, MemoryEntry, MemoryQuery, MemoryTier};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Session Memory 实现
///
/// 使用 HashMap 存储会话期间的记忆。
#[allow(dead_code)]
pub struct SessionMemory {
    /// 存储
    storage: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    /// 会话 ID
    #[allow(dead_code)]
    session_id: String,
    /// 衰减策略
    #[allow(dead_code)]
    decay_policy: Box<dyn DecayPolicy>,
}

impl SessionMemory {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            session_id: session_id.into(),
            decay_policy: Box::new(TimeBasedDecay::default()),
        }
    }
}

impl Default for SessionMemory {
    fn default() -> Self {
        Self::new("default")
    }
}

#[async_trait]
impl MemoryStore for SessionMemory {
    fn tier(&self) -> MemoryTier {
        MemoryTier::Session
    }

    async fn store(&self, entry: MemoryEntry) -> Layer3Result<String> {
        let mut storage = self.storage.write();
        let id = entry.id.clone();
        storage.insert(id.clone(), entry);
        Ok(id)
    }

    async fn get(&self, id: &str) -> Layer3Result<Option<MemoryEntry>> {
        Ok(self.storage.read().get(id).cloned())
    }

    async fn delete(&self, id: &str) -> Layer3Result<bool> {
        Ok(self.storage.write().remove(id).is_some())
    }

    async fn query(&self, query: &MemoryQuery) -> Layer3Result<Vec<MemoryEntry>> {
        let storage = self.storage.read();
        let results: Vec<MemoryEntry> = storage
            .values()
            .filter(|e| {
                if let Some(tier) = query.tier {
                    if e.tier != tier {
                        return false;
                    }
                }
                e.content.contains(&query.query)
            })
            .take(query.limit.unwrap_or(10))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn list(&self, limit: Option<usize>) -> Layer3Result<Vec<MemoryEntry>> {
        let storage = self.storage.read();
        Ok(storage
            .values()
            .take(limit.unwrap_or(usize::MAX))
            .cloned()
            .collect())
    }

    async fn clear(&self) -> Layer3Result<usize> {
        let mut storage = self.storage.write();
        let count = storage.len();
        storage.clear();
        Ok(count)
    }

    async fn count(&self) -> Layer3Result<usize> {
        Ok(self.storage.read().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_memory() {
        let memory = SessionMemory::new("test-session");
        assert_eq!(memory.tier(), MemoryTier::Session);
    }
}
