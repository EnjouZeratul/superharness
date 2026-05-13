//! # Checkpoint System
//!
//! 检查点持久化系统，支持原子写入和崩溃恢复。
//!
//! ## 模块结构
//! - `writer`: CheckpointWriter 实现
//! - `atomic`: AtomicFileWriter 原子写入
//! - `recovery`: CrashRecovery 崩溃恢复
//! - `checksum`: ChecksumUtils 校验和

mod writer;
mod atomic;
mod recovery;
mod checksum;

pub use writer::CheckpointWriter;
pub use atomic::AtomicFileWriter;
pub use recovery::CrashRecovery;
pub use checksum::ChecksumUtils;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::types::{CheckpointId, CheckpointMeta, Layer2Result, SessionId};

/// 检查点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointData {
    pub checkpoint_id: CheckpointId,
    pub session_id: SessionId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub trigger: String,
    pub iteration: i32,
    pub messages: Vec<serde_json::Value>,
    pub tool_calls_pending: Vec<serde_json::Value>,
    pub tool_results: serde_json::Value,
    pub tokens_used: i64,
    pub cost_estimate: f64,
    pub resume_hint: Option<String>,
}

/// 检查点系统接口
#[async_trait]
pub trait CheckpointSystemTrait: Send + Sync {
    /// 保存检查点
    async fn save(&self, data: &CheckpointData) -> Layer2Result<CheckpointId>;

    /// 加载检查点
    async fn load(&self, session_id: &SessionId, checkpoint_id: Option<&CheckpointId>) -> Layer2Result<Option<CheckpointData>>;

    /// 列出会话的所有检查点
    async fn list(&self, session_id: &SessionId) -> Layer2Result<Vec<CheckpointMeta>>;

    /// 删除检查点
    async fn delete(&self, session_id: &SessionId, checkpoint_id: &CheckpointId) -> Layer2Result<bool>;

    /// 验证检查点完整性
    fn verify(&self, path: &Path) -> Layer2Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_data_creation() {
        let data = CheckpointData {
            checkpoint_id: CheckpointId::new(),
            session_id: SessionId::new(),
            created_at: chrono::Utc::now(),
            trigger: "manual".to_string(),
            iteration: 0,
            messages: Vec::new(),
            tool_calls_pending: Vec::new(),
            tool_results: serde_json::Value::Null,
            tokens_used: 0,
            cost_estimate: 0.0,
            resume_hint: None,
        };

        assert_eq!(data.trigger, "manual");
    }
}
