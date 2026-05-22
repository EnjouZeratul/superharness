//! # Audit CLI 集成
//!
//! 封装 audit_logger 为 CLI 易用接口。

use anyhow::Result;
use sh_core::layer4::{
    AuditAction, AuditConfig, AuditEntry, AuditFilter, AuditLogger, ExportFormat,
};
use std::sync::Arc;

/// Audit CLI 服务
///
/// 提供简化的审计日志操作接口。
pub struct AuditService {
    logger: Arc<AuditLogger>,
}

impl AuditService {
    /// 创建新的审计服务
    pub fn new() -> Self {
        Self {
            logger: Arc::new(AuditLogger::new(AuditConfig::default())),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(config: AuditConfig) -> Self {
        Self {
            logger: Arc::new(AuditLogger::new(config)),
        }
    }

    /// 记录成功操作
    pub async fn log_success(
        &self,
        user_id: &str,
        action: AuditAction,
        resource_type: &str,
        resource_id: Option<&str>,
    ) -> Result<()> {
        self.logger
            .log_success(user_id, action, resource_type, resource_id)
            .await
    }

    /// 记录失败操作
    pub async fn log_failure(
        &self,
        user_id: &str,
        action: AuditAction,
        resource_type: &str,
        resource_id: Option<&str>,
        error_code: &str,
        error_message: &str,
    ) -> Result<()> {
        self.logger
            .log_failure(
                user_id,
                action,
                resource_type,
                resource_id,
                error_code,
                error_message,
            )
            .await
    }

    /// 记录拒绝访问
    pub async fn log_denied(
        &self,
        user_id: &str,
        action: AuditAction,
        resource_type: &str,
        resource_id: Option<&str>,
        reason: &str,
    ) -> Result<()> {
        self.logger
            .log_denied(user_id, action, resource_type, resource_id, reason)
            .await
    }

    /// 使用构建器记录
    pub fn builder(&self, user_id: &str, action: AuditAction, resource_type: &str) -> AuditBuilder {
        AuditBuilder {
            logger: self.logger.clone(),
            user_id: user_id.to_string(),
            action,
            resource_type: resource_type.to_string(),
            resource_id: None,
            session_id: None,
            ip_address: None,
            details: None,
        }
    }

    /// 查询审计日志
    pub async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditLogEntry>> {
        let entries = self.logger.query(filter).await?;
        Ok(entries
            .into_iter()
            .map(AuditLogEntry::from_entry)
            .collect())
    }

    /// 查询用户操作
    pub async fn query_by_user(&self, user_id: &str, limit: usize) -> Result<Vec<AuditLogEntry>> {
        let filter = AuditFilter {
            user_id: Some(user_id.to_string()),
            limit: Some(limit),
            ..Default::default()
        };
        self.query(filter).await
    }

    /// 查询特定操作类型
    pub async fn query_by_action(
        &self,
        action: AuditAction,
        limit: usize,
    ) -> Result<Vec<AuditLogEntry>> {
        let filter = AuditFilter {
            action: Some(action),
            limit: Some(limit),
            ..Default::default()
        };
        self.query(filter).await
    }

    /// 导出审计日志
    pub async fn export(&self, format: ExportFormat, filter: AuditFilter) -> Result<Vec<u8>> {
        self.logger.export(format, filter).await
    }

    /// 导出为 JSON
    pub async fn export_json(&self) -> Result<String> {
        let data = self
            .export(ExportFormat::Json, AuditFilter::default())
            .await?;
        Ok(String::from_utf8(data)?)
    }

    /// 导出为 CSV
    pub async fn export_csv(&self) -> Result<String> {
        let data = self
            .export(ExportFormat::Csv, AuditFilter::default())
            .await?;
        Ok(String::from_utf8(data)?)
    }

    /// 清理过期日志
    pub async fn cleanup(&self, days: u32) -> Result<usize> {
        let before = chrono::Utc::now() - chrono::Duration::days(days as i64);
        self.logger.cleanup(before).await
    }

    /// 获取日志数量
    pub async fn count(&self) -> Result<usize> {
        self.logger.count().await
    }

    /// 获取配置
    pub fn config(&self) -> &AuditConfig {
        self.logger.config()
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}

/// 审计日志构建器
pub struct AuditBuilder {
    logger: Arc<AuditLogger>,
    user_id: String,
    action: AuditAction,
    resource_type: String,
    resource_id: Option<String>,
    session_id: Option<String>,
    ip_address: Option<String>,
    details: Option<serde_json::Value>,
}

impl AuditBuilder {
    pub fn with_resource_id(mut self, resource_id: &str) -> Self {
        self.resource_id = Some(resource_id.to_string());
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_ip(mut self, ip_address: &str) -> Self {
        self.ip_address = Some(ip_address.to_string());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub async fn log_success(self) -> Result<()> {
        let mut entry = AuditEntry::new(&self.user_id, self.action, &self.resource_type);

        if let Some(id) = self.resource_id {
            entry = entry.with_resource_id(&id);
        }
        if let Some(id) = self.session_id {
            entry = entry.with_session(&id);
        }
        if let Some(ip) = self.ip_address {
            entry = entry.with_ip(&ip);
        }
        if let Some(details) = self.details {
            entry = entry.with_details(details);
        }

        self.logger.log(entry).await
    }
}

/// 审计日志条目（CLI 显示格式）
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: String,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: String,
}

impl AuditLogEntry {
    fn from_entry(entry: AuditEntry) -> Self {
        Self {
            id: entry.id.to_string(),
            timestamp: entry.timestamp.to_rfc3339(),
            user_id: entry.user_id,
            action: entry.action.as_str().to_string(),
            resource_type: entry.resource_type,
            resource_id: entry.resource_id,
            result: if entry.result.is_success() {
                "success".to_string()
            } else {
                "failure".to_string()
            },
        }
    }

    /// 格式化为单行显示
    pub fn to_line(&self) -> String {
        format!(
            "[{}] {} {} {} {} {}",
            self.timestamp,
            self.user_id,
            self.action,
            self.resource_type,
            self.resource_id.as_deref().unwrap_or("-"),
            self.result
        )
    }
}

/// 常用操作类型快捷方式
pub mod actions {
    use super::AuditAction;

    pub fn login() -> AuditAction {
        AuditAction::Login
    }
    pub fn logout() -> AuditAction {
        AuditAction::Logout
    }
    pub fn read() -> AuditAction {
        AuditAction::Read
    }
    pub fn write() -> AuditAction {
        AuditAction::Create
    }
    pub fn delete() -> AuditAction {
        AuditAction::Delete
    }
    pub fn execute() -> AuditAction {
        AuditAction::Execute
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_service_creation() {
        let service = AuditService::new();
        assert!(service.config().enabled);
    }

    #[tokio::test]
    async fn test_log_success() {
        let service = AuditService::new();
        service
            .log_success("user1", AuditAction::Read, "doc", Some("doc123"))
            .await
            .unwrap();

        let count = service.count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_query_by_user() {
        let service = AuditService::new();
        service
            .log_success("user1", AuditAction::Read, "doc", None)
            .await
            .unwrap();
        service
            .log_success("user2", AuditAction::Read, "doc", None)
            .await
            .unwrap();

        let entries = service.query_by_user("user1", 10).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_audit_entry_to_line() {
        let entry = AuditLogEntry {
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            user_id: "user1".to_string(),
            action: "read".to_string(),
            resource_type: "doc".to_string(),
            resource_id: Some("doc123".to_string()),
            result: "success".to_string(),
        };

        let line = entry.to_line();
        assert!(line.contains("user1"));
        assert!(line.contains("read"));
    }
}
