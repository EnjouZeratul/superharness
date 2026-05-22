//! # Checkpoint Writer
//!
//! 检查点写入器实现。

use async_trait::async_trait;
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::types::{CheckpointId, CheckpointMeta, Layer2Result, SessionId};

use super::{AtomicFileWriter, CheckpointData, CheckpointSystemTrait, ChecksumUtils};

/// 检查点写入器
pub struct CheckpointWriter {
    storage_path: PathBuf,
    max_backups: usize,
    atomic_writer: AtomicFileWriter,
}

impl CheckpointWriter {
    /// 创建新的检查点写入器
    pub fn new(storage_path: impl AsRef<Path>) -> Self {
        Self {
            storage_path: storage_path.as_ref().to_path_buf(),
            max_backups: 3,
            atomic_writer: AtomicFileWriter::new(),
        }
    }

    /// 配置最大备份数
    pub fn with_max_backups(mut self, max: usize) -> Self {
        self.max_backups = max;
        self
    }

    /// 获取会话目录
    fn session_dir(&self, session_id: &SessionId) -> PathBuf {
        self.storage_path
            .join(session_id.to_string())
            .join("checkpoints")
    }

    /// 备份现有检查点
    fn backup_checkpoint(&self, filepath: &Path) {
        if !filepath.exists() {
            return;
        }

        let ext = filepath.extension().and_then(|e| e.to_str()).unwrap_or("");

        let backup_path = filepath.with_extension(format!("{}.backup", ext));
        let _ = std::fs::copy(filepath, backup_path);
    }

    /// 清理旧备份
    fn prune_backups(&self, session_dir: &Path) {
        let mut backups: Vec<_> = std::fs::read_dir(session_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "backup")
                    .unwrap_or(false)
            })
            .collect();

        // 按修改时间排序（最新在前）
        backups.sort_by(|a, b| {
            let a_time = a
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            let b_time = b
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        // 删除超过限制的备份
        for old_backup in backups.into_iter().skip(self.max_backups) {
            let _ = std::fs::remove_file(old_backup.path());
        }
    }

    /// 更新 latest 引用
    fn update_latest(&self, session_dir: &Path, checkpoint_path: &Path) -> Layer2Result<()> {
        let latest_path = session_dir.join("latest.json");

        #[cfg(windows)]
        {
            // Windows: 使用复制（符号链接需要管理员权限）
            std::fs::copy(checkpoint_path, &latest_path)?;
        }

        #[cfg(not(windows))]
        {
            // Unix: 使用原子符号链接
            let temp_link = session_dir.join(format!(".tmp_latest_{}", uuid::Uuid::new_v4()));
            std::os::unix::fs::symlink(checkpoint_path.file_name().unwrap(), &temp_link)?;
            std::fs::rename(&temp_link, &latest_path)?;
        }

        Ok(())
    }
}

#[async_trait]
impl CheckpointSystemTrait for CheckpointWriter {
    async fn save(&self, data: &CheckpointData) -> Layer2Result<CheckpointId> {
        let session_dir = self.session_dir(&data.session_id);
        std::fs::create_dir_all(&session_dir)?;

        // 生成文件名
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let checkpoint_id = data.checkpoint_id.clone();
        let filename = format!("cp_{}_{}.json", timestamp, checkpoint_id);
        let filepath = session_dir.join(&filename);

        // 序列化数据
        let mut json_data = serde_json::to_value(data)?;
        json_data = ChecksumUtils::add_checksum(json_data);
        let json_content = serde_json::to_string_pretty(&json_data)?;

        // 备份现有 latest
        let latest_path = session_dir.join("latest.json");
        self.backup_checkpoint(&latest_path);

        // 原子写入
        self.atomic_writer.write_atomic(&filepath, &json_content)?;

        // 更新 latest 引用
        let _ = self.update_latest(&session_dir, &filepath);

        // 清理旧备份
        self.prune_backups(&session_dir);

        Ok(checkpoint_id)
    }

    async fn load(
        &self,
        session_id: &SessionId,
        checkpoint_id: Option<&CheckpointId>,
    ) -> Layer2Result<Option<CheckpointData>> {
        let session_dir = self.session_dir(session_id);

        if !session_dir.exists() {
            return Ok(None);
        }

        // 确定要加载的检查点
        let filepath = if let Some(id) = checkpoint_id {
            // 查找指定的检查点
            let pattern = format!("cp_*_{}.json", id);
            let matches: Vec<_> =
                glob::glob(session_dir.join(&pattern).to_string_lossy().as_ref())?
                    .filter_map(|e| e.ok())
                    .collect();

            if matches.is_empty() {
                return Ok(None);
            }
            matches[0].clone()
        } else {
            // 加载最新的检查点
            let latest_path = session_dir.join("latest.json");
            if latest_path.exists() {
                latest_path
            } else {
                // 查找最新的检查点
                let mut checkpoints: Vec<_> = std::fs::read_dir(&session_dir)?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name().to_string_lossy().starts_with("cp_")
                            && e.path()
                                .extension()
                                .map(|ext| ext == "json")
                                .unwrap_or(false)
                    })
                    .collect();

                if checkpoints.is_empty() {
                    return Ok(None);
                }

                checkpoints.sort_by(|a, b| {
                    let a_time = a
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    let b_time = b
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    b_time.cmp(&a_time)
                });

                checkpoints[0].path()
            }
        };

        if !filepath.exists() {
            return Ok(None);
        }

        // 读取并验证
        let content = std::fs::read_to_string(&filepath)?;
        let data: serde_json::Value = serde_json::from_str(&content)?;

        let (valid, _) = ChecksumUtils::verify_checksum(&data);
        if !valid {
            return Err(anyhow::anyhow!("Checkpoint checksum verification failed"));
        }

        let checkpoint: CheckpointData = serde_json::from_value(data)?;
        Ok(Some(checkpoint))
    }

    async fn list(&self, session_id: &SessionId) -> Layer2Result<Vec<CheckpointMeta>> {
        let session_dir = self.session_dir(session_id);

        if !session_dir.exists() {
            return Ok(Vec::new());
        }

        let mut metas = Vec::new();

        for entry in std::fs::read_dir(&session_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("cp_"))
                .unwrap_or(false)
            {
                continue;
            }

            if path.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }

            // 读取并验证
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let (valid, _) = ChecksumUtils::verify_checksum(&data);

                    if valid {
                        if let Ok(meta) = serde_json::from_value::<CheckpointMeta>(data) {
                            metas.push(meta);
                        }
                    }
                }
            }
        }

        // 按创建时间排序（最新在前）
        metas.sort_by_key(|b| std::cmp::Reverse(b.created_at));

        Ok(metas)
    }

    async fn delete(
        &self,
        session_id: &SessionId,
        checkpoint_id: &CheckpointId,
    ) -> Layer2Result<bool> {
        let session_dir = self.session_dir(session_id);

        if !session_dir.exists() {
            return Ok(false);
        }

        let pattern = format!("cp_*_{}.json", checkpoint_id);

        if let Some(path) = glob::glob(session_dir.join(&pattern).to_string_lossy().as_ref())?.flatten().next() {
            std::fs::remove_file(&path)?;
            return Ok(true);
        }

        Ok(false)
    }

    fn verify(&self, path: &Path) -> Layer2Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(path)?;
        let data: serde_json::Value = serde_json::from_str(&content)?;

        let (valid, _) = ChecksumUtils::verify_checksum(&data);
        Ok(valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_writer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let writer = CheckpointWriter::new(temp_dir.path());

        assert!(writer.storage_path.exists() || writer.storage_path.parent().is_some());
    }

    #[tokio::test]
    async fn test_save_and_load_checkpoint() {
        let temp_dir = TempDir::new().unwrap();
        let writer = CheckpointWriter::new(temp_dir.path());

        let data = CheckpointData {
            checkpoint_id: CheckpointId::new(),
            session_id: SessionId::new(),
            created_at: Utc::now(),
            trigger: "manual".to_string(),
            iteration: 1,
            messages: vec![serde_json::json!({"role": "user", "content": "test"})],
            tool_calls_pending: Vec::new(),
            tool_results: serde_json::Value::Null,
            tokens_used: 100,
            cost_estimate: 0.01,
            resume_hint: None,
        };

        let session_id = data.session_id.clone();
        let saved_id = writer.save(&data).await.unwrap();

        let loaded = writer.load(&session_id, None).await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.checkpoint_id, saved_id);
        assert_eq!(loaded.iteration, 1);
    }
}
