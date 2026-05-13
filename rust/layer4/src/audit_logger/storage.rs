//! 审计日志存储后端
//!
//! 支持多种存储方式。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use super::entry::{AuditEntry, AuditFilter, ExportFormat};
use anyhow::{anyhow, Result};

/// 审计存储 trait
#[async_trait]
pub trait AuditStorage: Send + Sync {
    /// 保存审计条目
    async fn save(&self, entry: &AuditEntry) -> Result<()>;

    /// 查询审计条目
    async fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>>;

    /// 导出审计日志
    async fn export(&self, format: ExportFormat, filter: &AuditFilter) -> Result<Vec<u8>>;

    /// 清理过期日志
    async fn cleanup(&self, before: DateTime<Utc>) -> Result<usize>;

    /// 获取条目数量
    async fn count(&self) -> Result<usize>;
}

/// 内存存储 (用于测试和小规模使用)
pub struct MemoryStorage {
    entries: Arc<RwLock<VecDeque<AuditEntry>>>,
    max_entries: usize,
}

impl MemoryStorage {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// 获取所有条目
    pub fn get_all(&self) -> Vec<AuditEntry> {
        self.entries.read().iter().cloned().collect()
    }
}

#[async_trait]
impl AuditStorage for MemoryStorage {
    async fn save(&self, entry: &AuditEntry) -> Result<()> {
        let mut entries = self.entries.write();

        // 如果超过容量，删除最旧的条目
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }

        entries.push_back(entry.clone());
        Ok(())
    }

    async fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>> {
        let entries = self.entries.read();

        let mut results: Vec<AuditEntry> = entries
            .iter()
            .filter(|e| e.matches_filter(filter))
            .cloned()
            .collect();

        // 应用限制
        if let Some(limit) = filter.limit {
            results = results.into_iter().take(limit).collect();
        }

        Ok(results)
    }

    async fn export(&self, format: ExportFormat, filter: &AuditFilter) -> Result<Vec<u8>> {
        let entries = self.query(filter).await?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&entries)?;
                Ok(json.into_bytes())
            }
            ExportFormat::Csv => {
                let mut csv = String::from("id,timestamp,user_id,action,resource_type,result\n");
                for entry in entries {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        entry.id,
                        entry.timestamp.to_rfc3339(),
                        entry.user_id,
                        entry.action.as_str(),
                        entry.resource_type,
                        if entry.result.is_success() { "success" } else { "failure" }
                    ));
                }
                Ok(csv.into_bytes())
            }
            ExportFormat::Syslog => {
                let mut syslog = String::new();
                for entry in entries {
                    syslog.push_str(&format!(
                        "{} AUDIT: user={} action={} resource={} result={}\n",
                        entry.timestamp.to_rfc3339(),
                        entry.user_id,
                        entry.action.as_str(),
                        entry.resource_type,
                        if entry.result.is_success() { "SUCCESS" } else { "FAILURE" }
                    ));
                }
                Ok(syslog.into_bytes())
            }
        }
    }

    async fn cleanup(&self, before: DateTime<Utc>) -> Result<usize> {
        let mut entries = self.entries.write();
        let original_len = entries.len();

        entries.retain(|e| e.timestamp >= before);

        Ok(original_len - entries.len())
    }

    async fn count(&self) -> Result<usize> {
        Ok(self.entries.read().len())
    }
}

/// 文件存储
pub struct FileStorage {
    base_path: PathBuf,
    max_file_size: usize,
}

impl FileStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }

    fn get_log_path(&self, date: DateTime<Utc>) -> PathBuf {
        self.base_path
            .join(format!("audit-{}.jsonl", date.format("%Y-%m-%d")))
    }
}

#[async_trait]
impl AuditStorage for FileStorage {
    async fn save(&self, entry: &AuditEntry) -> Result<()> {
        let path = self.get_log_path(entry.timestamp);

        // 确保目录存在
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 写入条目 (JSON Lines 格式)
        let json = serde_json::to_string(entry)?;
        let line = format!("{}\n", json);

        tokio::fs::write(&path, line).await?;
        Ok(())
    }

    async fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditEntry>> {
        // 简化实现：读取所有文件并过滤
        let mut results = Vec::new();

        // 列出所有日志文件
        let mut entries = tokio::fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "jsonl") {
                let content = tokio::fs::read_to_string(&path).await?;

                for line in content.lines() {
                    if let Ok(audit_entry) = serde_json::from_str::<AuditEntry>(line) {
                        if audit_entry.matches_filter(filter) {
                            results.push(audit_entry);
                        }
                    }
                }
            }
        }

        // 应用限制
        if let Some(limit) = filter.limit {
            results = results.into_iter().take(limit).collect();
        }

        Ok(results)
    }

    async fn export(&self, format: ExportFormat, filter: &AuditFilter) -> Result<Vec<u8>> {
        let entries = self.query(filter).await?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&entries)?;
                Ok(json.into_bytes())
            }
            ExportFormat::Csv => {
                let mut csv = String::from("id,timestamp,user_id,action,resource_type,result\n");
                for entry in entries {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        entry.id,
                        entry.timestamp.to_rfc3339(),
                        entry.user_id,
                        entry.action.as_str(),
                        entry.resource_type,
                        if entry.result.is_success() { "success" } else { "failure" }
                    ));
                }
                Ok(csv.into_bytes())
            }
            ExportFormat::Syslog => {
                let mut syslog = String::new();
                for entry in entries {
                    syslog.push_str(&format!(
                        "{} AUDIT: user={} action={} resource={} result={}\n",
                        entry.timestamp.to_rfc3339(),
                        entry.user_id,
                        entry.action.as_str(),
                        entry.resource_type,
                        if entry.result.is_success() { "SUCCESS" } else { "FAILURE" }
                    ));
                }
                Ok(syslog.into_bytes())
            }
        }
    }

    async fn cleanup(&self, before: DateTime<Utc>) -> Result<usize> {
        let mut removed_count = 0;

        let mut entries = tokio::fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // 检查文件名中的日期
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if let Some(date_str) = filename.strip_prefix("audit-").and_then(|s| s.strip_suffix(".jsonl")) {
                if let Ok(file_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    let file_datetime = file_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                    if file_datetime < before {
                        tokio::fs::remove_file(&path).await?;
                        removed_count += 1;
                    }
                }
            }
        }

        Ok(removed_count)
    }

    async fn count(&self) -> Result<usize> {
        let entries = self.query(&AuditFilter::default()).await?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::entry::AuditAction;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryStorage::new(100);

        let entry = AuditEntry::new("user1", AuditAction::Read, "doc");
        storage.save(&entry).await.unwrap();

        let entries = storage.query(&AuditFilter::default()).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_storage_limit() {
        let storage = MemoryStorage::new(2);

        storage.save(&AuditEntry::new("user1", AuditAction::Read, "doc")).await.unwrap();
        storage.save(&AuditEntry::new("user2", AuditAction::Read, "doc")).await.unwrap();
        storage.save(&AuditEntry::new("user3", AuditAction::Read, "doc")).await.unwrap();

        let count = storage.count().await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_export_json() {
        let storage = MemoryStorage::new(100);
        storage.save(&AuditEntry::new("user1", AuditAction::Login, "session")).await.unwrap();

        let data = storage.export(ExportFormat::Json, &AuditFilter::default()).await.unwrap();
        let json_str = String::from_utf8(data).unwrap();
        assert!(json_str.contains("user1"));
    }

    #[tokio::test]
    async fn test_file_storage() {
        let dir = tempdir().unwrap();
        let storage = FileStorage::new(dir.path().to_path_buf());

        let entry = AuditEntry::new("user1", AuditAction::Read, "doc");
        storage.save(&entry).await.unwrap();

        let entries = storage.query(&AuditFilter::default()).await.unwrap();
        assert!(!entries.is_empty());
    }
}