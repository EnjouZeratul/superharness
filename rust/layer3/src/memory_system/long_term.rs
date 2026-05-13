//! # Long-term Memory
//!
//! 长期记忆：跨项目的通用知识，使用向量存储。

use crate::memory_system::{DecayPolicy, MemoryStore, TimeBasedDecay};
use crate::retriever_engine::RetrieverEngine;
use crate::types::{Layer3Result, MemoryEntry, MemoryQuery, MemoryTier};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;

/// Long-term Memory 实现
///
/// 使用向量数据库存储，支持语义检索。
pub struct LongTermMemory {
    /// 检索引擎
    retriever: Option<Arc<dyn RetrieverEngine>>,
    /// 本地缓存
    cache: Arc<RwLock<Vec<MemoryEntry>>>,
    /// 衰减策略
    decay_policy: Box<dyn DecayPolicy>,
}

impl LongTermMemory {
    pub fn new(retriever: Option<Arc<dyn RetrieverEngine>>) -> Self {
        Self {
            retriever,
            cache: Arc::new(RwLock::new(Vec::new())),
            decay_policy: Box::new(TimeBasedDecay::default()),
        }
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl MemoryStore for LongTermMemory {
    fn tier(&self) -> MemoryTier {
        MemoryTier::LongTerm
    }

    async fn store(&self, entry: MemoryEntry) -> Layer3Result<String> {
        let id = entry.id.clone();

        // 存储到检索引擎（如果有）
        if let Some(retriever) = &self.retriever {
            use crate::retriever_engine::Document;
            let doc = Document::new(&entry.content).with_source(&entry.id);
            retriever.index(vec![doc]).await?;
        }

        // 缓存
        self.cache.write().push(entry);

        Ok(id)
    }

    async fn get(&self, id: &str) -> Layer3Result<Option<MemoryEntry>> {
        let cache = self.cache.read();
        Ok(cache.iter().find(|e| e.id == id).cloned())
    }

    async fn delete(&self, id: &str) -> Layer3Result<bool> {
        // 从检索引擎删除
        if let Some(retriever) = &self.retriever {
            retriever.delete(&[id.to_string()]).await?;
        }

        // 从缓存删除
        let mut cache = self.cache.write();
        let len_before = cache.len();
        cache.retain(|e| e.id != id);
        Ok(cache.len() < len_before)
    }

    async fn query(&self, query: &MemoryQuery) -> Layer3Result<Vec<MemoryEntry>> {
        // 使用向量检索
        if let Some(retriever) = &self.retriever {
            let results = retriever
                .retrieve(&query.query, query.limit.unwrap_or(10))
                .await?;
            let entries: Vec<MemoryEntry> = results
                .into_iter()
                .map(|r| MemoryEntry {
                    id: r.doc_id,
                    tier: MemoryTier::LongTerm,
                    content: r.content,
                    metadata: r.metadata.into_iter().collect(),
                    created_at: chrono::Utc::now(),
                    last_accessed: chrono::Utc::now(),
                    access_count: 0,
                    importance: r.score,
                })
                .collect();
            return Ok(entries);
        }

        // 回退到缓存搜索
        let cache = self.cache.read();
        let results: Vec<MemoryEntry> = cache
            .iter()
            .filter(|e| e.content.contains(&query.query))
            .take(query.limit.unwrap_or(10))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn list(&self, limit: Option<usize>) -> Layer3Result<Vec<MemoryEntry>> {
        let cache = self.cache.read();
        Ok(cache
            .iter()
            .take(limit.unwrap_or(usize::MAX))
            .cloned()
            .collect())
    }

    async fn clear(&self) -> Layer3Result<usize> {
        let count = self.cache.read().len();
        self.cache.write().clear();

        // 清空检索引擎
        if let Some(retriever) = &self.retriever {
            retriever.clear().await?;
        }

        Ok(count)
    }

    async fn count(&self) -> Layer3Result<usize> {
        Ok(self.cache.read().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_term_memory_tier() {
        let memory = LongTermMemory::default();
        assert_eq!(memory.tier(), MemoryTier::LongTerm);
    }
}
