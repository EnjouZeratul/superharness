//! 审计日志条目定义
//!
//! 符合 SOC2 合规标准的审计日志格式。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// 唯一 ID
    pub id: Uuid,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 用户 ID
    pub user_id: String,
    /// 会话 ID (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 操作类型
    pub action: AuditAction,
    /// 资源类型
    pub resource_type: String,
    /// 资源 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// 操作结果
    pub result: AuditResult,
    /// 详细信息 (已脱敏)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// IP 地址 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    /// 请求 ID (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// 持续时间 (毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// 审计操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    /// 登录
    Login,
    /// 登出
    Logout,
    /// 创建资源
    Create,
    /// 读取资源
    Read,
    /// 更新资源
    Update,
    /// 删除资源
    Delete,
    /// 执行操作
    Execute,
    /// 访问资源
    Access,
    /// 配置更改
    ConfigChange,
    /// 权限更改
    PermissionChange,
    /// 工具调用
    ToolCall,
    /// LLM 请求
    LlmRequest,
    /// 会话操作
    SessionOperation,
    /// 其他
    Other(String),
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            AuditAction::Login => "login",
            AuditAction::Logout => "logout",
            AuditAction::Create => "create",
            AuditAction::Read => "read",
            AuditAction::Update => "update",
            AuditAction::Delete => "delete",
            AuditAction::Execute => "execute",
            AuditAction::Access => "access",
            AuditAction::ConfigChange => "config_change",
            AuditAction::PermissionChange => "permission_change",
            AuditAction::ToolCall => "tool_call",
            AuditAction::LlmRequest => "llm_request",
            AuditAction::SessionOperation => "session_operation",
            AuditAction::Other(s) => s.as_str(),
        }
    }
}

/// 审计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    /// 成功
    Success,
    /// 失败
    Failure {
        /// 错误码
        error_code: String,
        /// 错误消息
        error_message: String,
    },
    /// 拒绝访问
    Denied {
        /// 拒绝原因
        reason: String,
    },
}

impl AuditResult {
    pub fn is_success(&self) -> bool {
        matches!(self, AuditResult::Success)
    }

    pub fn success() -> Self {
        AuditResult::Success
    }

    pub fn failure(code: &str, message: &str) -> Self {
        AuditResult::Failure {
            error_code: code.to_string(),
            error_message: message.to_string(),
        }
    }

    pub fn denied(reason: &str) -> Self {
        AuditResult::Denied {
            reason: reason.to_string(),
        }
    }
}

/// 审计日志过滤器
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuditFilter {
    /// 用户 ID (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// 操作类型 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<AuditAction>,
    /// 资源类型 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    /// 结果 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<AuditResult>,
    /// 开始时间 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    /// 限制数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

impl AuditEntry {
    /// 创建新的审计条目
    pub fn new(user_id: &str, action: AuditAction, resource_type: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: user_id.to_string(),
            session_id: None,
            action,
            resource_type: resource_type.to_string(),
            resource_id: None,
            result: AuditResult::Success,
            details: None,
            ip_address: None,
            request_id: None,
            duration_ms: None,
        }
    }

    /// 设置会话 ID
    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// 设置资源 ID
    pub fn with_resource_id(mut self, resource_id: &str) -> Self {
        self.resource_id = Some(resource_id.to_string());
        self
    }

    /// 设置结果
    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    /// 设置详细信息
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// 设置 IP 地址
    pub fn with_ip(mut self, ip_address: &str) -> Self {
        self.ip_address = Some(ip_address.to_string());
        self
    }

    /// 设置请求 ID
    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    /// 设置持续时间
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// 匹配过滤器
    pub fn matches_filter(&self, filter: &AuditFilter) -> bool {
        if let Some(user_id) = &filter.user_id {
            if self.user_id != *user_id {
                return false;
            }
        }

        if let Some(action) = &filter.action {
            if self.action.as_str() != action.as_str() {
                return false;
            }
        }

        if let Some(resource_type) = &filter.resource_type {
            if self.resource_type != *resource_type {
                return false;
            }
        }

        if let Some(start_time) = &filter.start_time {
            if self.timestamp < *start_time {
                return false;
            }
        }

        if let Some(end_time) = &filter.end_time {
            if self.timestamp > *end_time {
                return false;
            }
        }

        true
    }
}

/// 导出格式
#[derive(Debug, Clone)]
pub enum ExportFormat {
    /// JSON
    Json,
    /// CSV
    Csv,
    /// Syslog 格式
    Syslog,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new("user123", AuditAction::Read, "document");
        assert_eq!(entry.user_id, "user123");
        assert!(entry.result.is_success());
    }

    #[test]
    fn test_audit_entry_with_details() {
        let entry = AuditEntry::new("user123", AuditAction::Execute, "tool")
            .with_details(serde_json::json!({ "tool_name": "test" }));

        assert!(entry.details.is_some());
    }

    #[test]
    fn test_audit_result() {
        let success = AuditResult::success();
        assert!(success.is_success());

        let failure = AuditResult::failure("E001", "Test error");
        assert!(!failure.is_success());
    }

    #[test]
    fn test_audit_filter_matching() {
        let entry = AuditEntry::new("user123", AuditAction::Read, "document");

        let filter = AuditFilter {
            user_id: Some("user123".to_string()),
            ..Default::default()
        };
        assert!(entry.matches_filter(&filter));

        let filter2 = AuditFilter {
            user_id: Some("other_user".to_string()),
            ..Default::default()
        };
        assert!(!entry.matches_filter(&filter2));
    }

    #[test]
    fn test_serialize_audit_entry() {
        let entry = AuditEntry::new("user123", AuditAction::Login, "session");
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("user123"));
        assert!(json.contains("Login")); // Serde uses PascalCase for enum variants
    }
}
