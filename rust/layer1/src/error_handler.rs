//! 错误处理模块
//!
//! 统一的错误类型和结果类型定义。

use thiserror::Error;

/// SuperHarness 统一错误类型
#[derive(Debug, Error)]
pub enum ShError {
    #[error("Layer 0 error: {0}")]
    Layer0(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("LLM API error: {0}")]
    LlmApi(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Timeout error after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Rate limited")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// 从 anyhow::Error 转换
impl From<anyhow::Error> for ShError {
    fn from(e: anyhow::Error) -> Self {
        ShError::Internal(e.to_string())
    }
}

/// SuperHarness 统一结果类型
pub type ShResult<T> = std::result::Result<T, ShError>;

/// 错误处理器（用于集中处理错误）
pub struct ErrorHandler {
    /// 是否启用日志
    log_errors: bool,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self { log_errors: true }
    }

    /// 处理错误，返回用户友好的消息
    pub fn handle(&self, error: &ShError) -> String {
        if self.log_errors {
            tracing::error!("Error: {:?}", error);
        }
        error.to_string()
    }

    /// 将错误转换为用户消息
    pub fn to_user_message(&self, error: &ShError) -> String {
        match error {
            ShError::Timeout { seconds } => format!("操作超时，请重试（已等待 {} 秒）", seconds),
            ShError::RateLimited => "请求过于频繁，请稍后再试".to_string(),
            ShError::NotFound { resource } => format!("找不到资源: {}", resource),
            _ => format!("发生错误: {}", error),
        }
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}
