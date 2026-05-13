//! 审计日志模块
//!
//! 符合 SOC2 合规标准的审计日志系统。
//!
//! ## 功能
//! - 完整的审计日志记录
//! - 敏感数据脱敏
//! - 多种存储后端
//! - 灵活的查询和导出
//!
//! ## 用法
//! ```rust,ignore
//! use sh_layer4::audit_logger::{AuditLogger, AuditConfig, AuditAction, AuditEntry};
//!
//! let logger = AuditLogger::new(AuditConfig::default());
//!
//! // 快速记录
//! logger.log_success("user1", AuditAction::Read, "document", Some("doc123")).await?;
//!
//! // 构建器模式
//! logger.builder("user1", AuditAction::Login, "session")
//!     .with_ip("192.168.1.1")
//!     .with_duration(100)
//!     .log()
//!     .await?;
//! ```

pub mod entry;
pub mod logger;
pub mod storage;

// 主要导出
pub use entry::{AuditAction, AuditEntry, AuditFilter, AuditResult, ExportFormat};
pub use logger::{AuditConfig, AuditLogger};
pub use storage::{AuditStorage, FileStorage, MemoryStorage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // 测试主要类型可以导出
        let _config = AuditConfig::default();
        let _entry = AuditEntry::new("user1", AuditAction::Read, "document");
    }
}
