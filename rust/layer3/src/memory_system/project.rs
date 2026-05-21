//! # Project Memory
//!
//! 项目记忆：项目级别的知识库，持久化到文件系统。

use crate::memory_system::{DecayPolicy, MemoryStore, TimeBasedDecay};
use crate::types::{Layer3Result, MemoryEntry, MemoryQuery, MemoryTier};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

/// Project Memory 实现
///
/// 持久化到项目目录下的 `.continuum/memory/`。
pub struct ProjectMemory {
    /// 项目根目录
    project_root: PathBuf,
    /// 内存文件路径
    memory_path: PathBuf,
    /// 内存缓存
    cache: Arc<RwLock<Vec<MemoryEntry>>>,
    /// 衰减策略
    decay_policy: Box<dyn DecayPolicy>,
}

impl ProjectMemory {
    pub fn new(project_root: PathBuf) -> Self {
        let memory_path = project_root.join(".continuum").join("memory");
        Self {
            project_root,
            memory_path,
            cache: Arc::new(RwLock::new(Vec::new())),
            decay_policy: Box::new(TimeBasedDecay::default()),
        }
    }

    /// 确保内存目录存在
    async fn ensure_dir(&self) -> Layer3Result<()> {
        tokio::fs::create_dir_all(&self.memory_path).await?;
        Ok(())
    }
}

#[async_trait]
impl MemoryStore for ProjectMemory {
    fn tier(&self) -> MemoryTier {
        MemoryTier::Project
    }

    async fn store(&self, entry: MemoryEntry) -> Layer3Result<String> {
        self.ensure_dir().await?;
        let file_path = self.memory_path.join(format!("{}.json", entry.id));
        let content = serde_json::to_string(&entry)?;
        tokio::fs::write(&file_path, content).await?;

        let mut cache = self.cache.write();
        cache.push(entry.clone());

        Ok(entry.id)
    }

    async fn get(&self, id: &str) -> Layer3Result<Option<MemoryEntry>> {
        // 先查缓存
        {
            let cache = self.cache.read();
            if let Some(entry) = cache.iter().find(|e| e.id == id) {
                return Ok(Some(entry.clone()));
            }
        }

        // 从文件读取
        let file_path = self.memory_path.join(format!("{}.json", id));
        if file_path.exists() {
            let content = tokio::fs::read_to_string(&file_path).await?;
            let entry: MemoryEntry = serde_json::from_str(&content)?;
            return Ok(Some(entry));
        }

        Ok(None)
    }

    async fn delete(&self, id: &str) -> Layer3Result<bool> {
        let file_path = self.memory_path.join(format!("{}.json", id));
        if file_path.exists() {
            tokio::fs::remove_file(&file_path).await?;
            self.cache.write().retain(|e| e.id != id);
            return Ok(true);
        }
        Ok(false)
    }

    async fn query(&self, query: &MemoryQuery) -> Layer3Result<Vec<MemoryEntry>> {
        let cache = self.cache.read();
        let results: Vec<MemoryEntry> = cache
            .iter()
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

        // 删除文件
        if self.memory_path.exists() {
            let mut entries = tokio::fs::read_dir(&self.memory_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                tokio::fs::remove_file(entry.path()).await?;
            }
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
    fn test_project_memory_tier() {
        let memory = ProjectMemory::new(PathBuf::from("/tmp/test"));
        assert_eq!(memory.tier(), MemoryTier::Project);
    }
}
