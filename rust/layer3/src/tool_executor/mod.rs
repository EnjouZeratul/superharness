//! # Tool Executor
//!
//! 工具执行器：负责执行工具调用并返回结果。

pub mod executor;

use crate::types::{Layer3Result, ToolMeta, ToolRequest, ToolResponse};
use async_trait::async_trait;

// Re-export executor implementation
pub use executor::{DefaultToolExecutor, ExecutionRecord, JsonSchemaValidator};

/// 工具执行器 trait
///
/// 定义工具执行的核心接口。实现者负责：
/// 1. 接收工具调用请求
/// 2. 验证参数
/// 3. 执行工具逻辑
/// 4. 返回执行结果
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// 执行单个工具调用
    ///
    /// # Arguments
    /// * `request` - 工具调用请求
    ///
    /// # Returns
    /// * `ToolResponse` - 执行结果
    async fn execute(&self, request: ToolRequest) -> Layer3Result<ToolResponse>;

    /// 执行多个工具调用（并行）
    ///
    /// # Arguments
    /// * `requests` - 多个工具调用请求
    ///
    /// # Returns
    /// * `Vec<ToolResponse>` - 所有执行结果
    async fn execute_batch(&self, requests: Vec<ToolRequest>) -> Layer3Result<Vec<ToolResponse>>;

    /// 检查工具是否可用
    ///
    /// # Arguments
    /// * `name` - 工具名称
    fn is_available(&self, name: &str) -> bool;

    /// 获取工具元数据
    ///
    /// # Arguments
    /// * `name` - 工具名称
    fn get_meta(&self, name: &str) -> Option<ToolMeta>;

    /// 获取所有已注册工具的元数据
    fn list_tools(&self) -> Vec<ToolMeta>;
}

/// 工具验证 trait
///
/// 定义参数验证接口。
pub trait ToolValidator: Send + Sync {
    /// 验证工具调用参数
    ///
    /// # Arguments
    /// * `request` - 工具调用请求
    ///
    /// # Returns
    /// * `bool` - 参数是否有效
    fn validate(&self, request: &ToolRequest) -> bool;

    /// 获取验证失败原因
    ///
    /// # Arguments
    /// * `request` - 工具调用请求
    fn validate_with_reason(&self, request: &ToolRequest) -> Result<(), String>;
}

/// 工具执行上下文
///
/// 提供工具执行时需要的上下文信息。
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 当前会话 ID
    pub session_id: String,
    /// 当前工作目录
    pub working_dir: std::path::PathBuf,
    /// 用户 ID（用于权限检查）
    pub user_id: Option<String>,
    /// 环境变量
    pub env_vars: std::collections::HashMap<String, String>,
    /// 最大执行时间（秒）
    pub timeout_secs: u64,
    /// 是否允许危险操作
    pub allow_dangerous: bool,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            working_dir: std::path::PathBuf::from("."),
            user_id: None,
            env_vars: std::collections::HashMap::new(),
            timeout_secs: 30,
            allow_dangerous: false,
        }
    }
}

/// 带上下文的工具执行器
///
/// 扩展 ToolExecutor，支持上下文传递。
#[async_trait]
pub trait ContextualExecutor: ToolExecutor {
    /// 带上下文执行工具
    ///
    /// # Arguments
    /// * `request` - 工具调用请求
    /// * `context` - 执行上下文
    async fn execute_with_context(
        &self,
        request: ToolRequest,
        context: ExecutionContext,
    ) -> Layer3Result<ToolResponse>;

    /// 带上下文批量执行
    async fn execute_batch_with_context(
        &self,
        requests: Vec<ToolRequest>,
        context: ExecutionContext,
    ) -> Layer3Result<Vec<ToolResponse>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_default() {
        let ctx = ExecutionContext::default();
        assert_eq!(ctx.timeout_secs, 30);
        assert!(!ctx.allow_dangerous);
    }
}
