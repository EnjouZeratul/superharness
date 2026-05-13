//! # Crash Recovery
//!
//! 崩溃恢复工具，检测不正常关闭并恢复会话。

use std::path::Path;

use crate::types::{Layer2Error, Layer2Result, SessionId};

/// 崩溃恢复管理器
pub struct CrashRecovery {
    storage_path: std::path::PathBuf,
}

impl CrashRecovery {
    /// 创建新的崩溃恢复管理器
    pub fn new(storage_path: impl AsRef<Path>) -> Self {
        Self {
            storage_path: storage_path.as_ref().to_path_buf(),
        }
    }

    /// 检测不正常关闭
    ///
    /// 返回崩溃信息（如果有）
    pub fn detect_unclean_shutdown(&self) -> Layer2Result<Option<CrashInfo>> {
        // 检查是否有活跃会话没有正常终止
        if !self.storage_path.exists() {
            return Ok(None);
        }

        for session_dir in std::fs::read_dir(&self.storage_path)? {
            let session_dir = session_dir?;
            if !session_dir.path().is_dir() {
                continue;
            }

            let session_meta = session_dir.path().join("session_meta.json");
            if !session_meta.exists() {
                continue;
            }

            match std::fs::read_to_string(&session_meta) {
                Ok(content) => {
                    if let Ok(meta) = serde_json::from_str::<SessionMeta>(&content) {
                        if meta.is_active && meta.termination_reason.is_none() {
                            return Ok(Some(CrashInfo {
                                session_id: Some(meta.session_id.clone()),
                                last_activity: meta.last_updated,
                                last_iteration: meta.last_iteration,
                            }));
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    /// 恢复会话
    ///
    /// 从最新的有效检查点恢复
    pub fn recover_session(&self, session_id: &SessionId) -> Layer2Result<Option<RecoveryResult>> {
        let session_dir = self
            .storage_path
            .join(session_id.to_string())
            .join("checkpoints");

        if !session_dir.exists() {
            return Ok(None);
        }

        // 查找最新的有效检查点
        let checkpoints = self.list_valid_checkpoints(&session_dir)?;

        for checkpoint_path in checkpoints {
            match std::fs::read_to_string(&checkpoint_path) {
                Ok(content) => {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        // 验证校验和
                        if super::ChecksumUtils::verify_checksum(&data).0 {
                            return Ok(Some(RecoveryResult {
                                checkpoint_path: Some(checkpoint_path),
                                data: Some(data),
                                recovered_from_backup: false,
                            }));
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        // 尝试从备份恢复
        self.recover_from_backup(&session_dir)
    }

    /// 列出有效的检查点
    fn list_valid_checkpoints(&self, dir: &Path) -> Layer2Result<Vec<std::path::PathBuf>> {
        let mut checkpoints = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Some(filename) = path.file_name() {
                    if filename.to_string_lossy().starts_with("cp_") {
                        checkpoints.push(path);
                    }
                }
            }
        }

        // 按修改时间排序（最新的在前）
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

        Ok(checkpoints)
    }

    /// 从备份恢复
    fn recover_from_backup(&self, dir: &Path) -> Layer2Result<Option<RecoveryResult>> {
        let backup_suffix = ".backup";

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "backup").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                        if super::ChecksumUtils::verify_checksum(&data).0 {
                            return Ok(Some(RecoveryResult {
                                checkpoint_path: Some(path),
                                data: Some(data),
                                recovered_from_backup: true,
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// 标记会话为活跃
    pub fn mark_session_active(&self, session_id: &SessionId) -> Layer2Result<()> {
        let meta_path = self
            .storage_path
            .join(session_id.to_string())
            .join("session_meta.json");

        if meta_path.exists() {
            let content = std::fs::read_to_string(&meta_path)?;
            let mut meta: SessionMeta = serde_json::from_str(&content)?;
            meta.is_active = true;
            meta.termination_reason = None;

            let json = serde_json::to_string_pretty(&meta)?;
            std::fs::write(&meta_path, json)?;
        }

        Ok(())
    }

    /// 标记会话为终止
    pub fn mark_session_terminated(
        &self,
        session_id: &SessionId,
        reason: &str,
    ) -> Layer2Result<()> {
        let meta_path = self
            .storage_path
            .join(session_id.to_string())
            .join("session_meta.json");

        if meta_path.exists() {
            let content = std::fs::read_to_string(&meta_path)?;
            let mut meta: SessionMeta = serde_json::from_str(&content)?;
            meta.is_active = false;
            meta.termination_reason = Some(reason.to_string());

            let json = serde_json::to_string_pretty(&meta)?;
            std::fs::write(&meta_path, json)?;
        }

        Ok(())
    }
}

/// 崩溃信息
#[derive(Debug, Clone)]
pub struct CrashInfo {
    pub session_id: Option<SessionId>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub last_iteration: i32,
}

/// 恢复结果
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub checkpoint_path: Option<std::path::PathBuf>,
    pub data: Option<serde_json::Value>,
    pub recovered_from_backup: bool,
}

/// 会话元数据（用于崩溃检测）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SessionMeta {
    session_id: SessionId,
    is_active: bool,
    termination_reason: Option<String>,
    last_updated: chrono::DateTime<chrono::Utc>,
    last_iteration: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_crash_recovery_creation() {
        let temp_dir = TempDir::new().unwrap();
        let recovery = CrashRecovery::new(temp_dir.path());

        let result = recovery.detect_unclean_shutdown().unwrap();
        assert!(result.is_none());
    }
}
