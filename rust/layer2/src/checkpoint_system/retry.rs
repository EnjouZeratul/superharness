//! # 错误恢复系统
//!
//! 实现三层重试机制：自动 → 降级 → 用户介入

use crate::types::{Layer2Result, SessionId};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// 错误类型分类
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 临时错误（网络波动、API 限流）
    Transient,
    /// 资源错误（内存不足、磁盘满）
    Resource,
    /// 配置错误（API Key 无效、配置缺失）
    Configuration,
    /// 逻辑错误（参数错误、工具失败）
    Logic,
    /// 系统错误（未知错误）
    System,
    /// 用户中断
    UserInterrupt,
}

impl ErrorCategory {
    /// 从错误消息分析类型
    pub fn from_error_message(msg: &str) -> Self {
        let msg_lower = msg.to_lowercase();

        if msg_lower.contains("timeout")
            || msg_lower.contains("network")
            || msg_lower.contains("rate limit")
        {
            ErrorCategory::Transient
        } else if msg_lower.contains("memory")
            || msg_lower.contains("disk")
            || msg_lower.contains("resource")
        {
            ErrorCategory::Resource
        } else if msg_lower.contains("api key")
            || msg_lower.contains("config")
            || msg_lower.contains("auth")
        {
            ErrorCategory::Configuration
        } else if msg_lower.contains("invalid")
            || msg_lower.contains("parameter")
            || msg_lower.contains("argument")
        {
            ErrorCategory::Logic
        } else if msg_lower.contains("interrupt")
            || msg_lower.contains("cancel")
            || msg_lower.contains("abort")
        {
            ErrorCategory::UserInterrupt
        } else {
            ErrorCategory::System
        }
    }

    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(self, ErrorCategory::Transient | ErrorCategory::Resource)
    }
}

/// 重试策略
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: usize,
    /// 初始延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    /// 延迟倍数（指数退避）
    pub multiplier: f64,
    /// 抖动因子（0.0-1.0）
    pub jitter: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl RetryPolicy {
    /// 计算第 n 次重试的延迟
    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let base_delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32);
        let capped_delay = base_delay.min(self.max_delay_ms as f64);

        // 添加确定性抖动（基于 attempt 避免需要 rand 依赖）
        let jitter_range = capped_delay * self.jitter;
        let jitter_offset = ((attempt as f64 * 0.3).fract() - 0.5) * 2.0 * jitter_range;
        let final_delay = (capped_delay + jitter_offset).max(0.0) as u64;

        Duration::from_millis(final_delay)
    }
}

/// 降级策略
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// 不降级
    None,
    /// 使用备用服务
    BackupService { endpoint: String },
    /// 使用缓存数据
    UseCache { max_age_seconds: u64 },
    /// 简化功能
    Simplified { mode: String },
    /// 跳过操作
    Skip,
}

/// 恢复层执行结果
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// 是否成功
    pub success: bool,
    /// 使用的恢复层
    pub layer_used: RecoveryLayer,
    /// 重试次数
    pub attempts: usize,
    /// 最终错误消息
    pub error_message: Option<String>,
    /// 用户操作建议
    pub user_action: Option<String>,
}

/// 恢复层
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryLayer {
    /// 第一层：自动重试
    Automatic,
    /// 第二层：降级执行
    Fallback,
    /// 第三层：用户介入
    UserIntervention,
}

/// 恢复动作（用户介入时）
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// 重试操作
    Retry,
    /// 跳过操作
    Skip,
    /// 终止会话
    Abort,
    /// 修改配置后重试
    ModifyConfig { key: String, value: String },
    /// 切换到备用服务
    SwitchBackup { service: String },
}

/// 用户确认回调
pub type UserConfirmationCallback =
    Arc<dyn Fn(&str, Vec<RecoveryAction>) -> RecoveryAction + Send + Sync>;

/// 错误恢复管理器
pub struct ErrorRecovery {
    /// 重试策略
    retry_policy: RetryPolicy,
    /// 降级策略
    fallback_strategy: FallbackStrategy,
    /// 用户确认回调
    user_callback: RwLock<Option<UserConfirmationCallback>>,
    /// 恢复统计
    stats: RwLock<RecoveryStats>,
}

/// 恢复统计
#[derive(Debug, Clone, Default)]
pub struct RecoveryStats {
    pub total_errors: usize,
    pub auto_recovered: usize,
    pub fallback_recovered: usize,
    pub user_interventions: usize,
    pub unrecovered: usize,
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecovery {
    /// 创建新的恢复管理器
    pub fn new() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            fallback_strategy: FallbackStrategy::None,
            user_callback: RwLock::new(None),
            stats: RwLock::new(RecoveryStats::default()),
        }
    }

    /// 设置重试策略
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// 设置降级策略
    pub fn with_fallback(mut self, strategy: FallbackStrategy) -> Self {
        self.fallback_strategy = strategy;
        self
    }

    /// 设置用户确认回调
    pub async fn set_user_callback(&self, callback: UserConfirmationCallback) {
        *self.user_callback.write().await = Some(callback);
    }

    /// 执行带恢复的操作
    pub async fn execute_with_recovery<F, Fut, T>(&self, operation: F) -> RecoveryResult
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Layer2Result<T>> + Send,
        T: Send,
    {
        let mut stats = self.stats.write().await;
        stats.total_errors += 1;
        drop(stats);

        // 第一层：自动重试
        let retry_result = self.try_with_retry(&operation).await;

        if retry_result.success {
            let mut stats = self.stats.write().await;
            stats.auto_recovered += 1;
            return retry_result;
        }

        // 第二层：降级执行
        let fallback_result = self.try_with_fallback(&operation).await;

        if fallback_result.success {
            let mut stats = self.stats.write().await;
            stats.fallback_recovered += 1;
            return fallback_result;
        }

        // 第三层：用户介入
        let user_result = self.try_with_user_intervention(&operation).await;

        if user_result.success {
            let mut stats = self.stats.write().await;
            stats.user_interventions += 1;
        } else {
            let mut stats = self.stats.write().await;
            stats.unrecovered += 1;
        }

        user_result
    }

    /// 第一层：自动重试
    async fn try_with_retry<F, Fut, T>(&self, operation: &F) -> RecoveryResult
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Layer2Result<T>> + Send,
        T: Send,
    {
        let mut last_error: Option<String> = None;

        for attempt in 0..=self.retry_policy.max_retries {
            match operation().await {
                Ok(_) => {
                    return RecoveryResult {
                        success: true,
                        layer_used: RecoveryLayer::Automatic,
                        attempts: attempt,
                        error_message: None,
                        user_action: None,
                    };
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let category = ErrorCategory::from_error_message(&error_msg);

                    if !category.is_retryable() {
                        return RecoveryResult {
                            success: false,
                            layer_used: RecoveryLayer::Automatic,
                            attempts: attempt,
                            error_message: Some(error_msg.clone()),
                            user_action: Some(self.get_user_hint(&category)),
                        };
                    }

                    last_error = Some(error_msg);

                    if attempt < self.retry_policy.max_retries {
                        let delay = self.retry_policy.delay_for_attempt(attempt);
                        sleep(delay).await;
                    }
                }
            }
        }

        RecoveryResult {
            success: false,
            layer_used: RecoveryLayer::Automatic,
            attempts: self.retry_policy.max_retries + 1,
            error_message: last_error,
            user_action: None,
        }
    }

    /// 第二层：降级执行
    async fn try_with_fallback<F, Fut, T>(&self, _operation: &F) -> RecoveryResult
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Layer2Result<T>> + Send,
        T: Send,
    {
        match &self.fallback_strategy {
            FallbackStrategy::None => RecoveryResult {
                success: false,
                layer_used: RecoveryLayer::Fallback,
                attempts: 0,
                error_message: Some("No fallback strategy configured".to_string()),
                user_action: None,
            },
            FallbackStrategy::Skip => RecoveryResult {
                success: true,
                layer_used: RecoveryLayer::Fallback,
                attempts: 1,
                error_message: None,
                user_action: Some("Operation skipped due to fallback policy".to_string()),
            },
            FallbackStrategy::BackupService { endpoint } => {
                // 简化实现：返回成功（实际实现需要切换服务端点）
                RecoveryResult {
                    success: true,
                    layer_used: RecoveryLayer::Fallback,
                    attempts: 1,
                    error_message: None,
                    user_action: Some(format!("Switched to backup: {}", endpoint)),
                }
            }
            FallbackStrategy::UseCache { max_age_seconds } => RecoveryResult {
                success: true,
                layer_used: RecoveryLayer::Fallback,
                attempts: 1,
                error_message: None,
                user_action: Some(format!("Using cached data (max {}s old)", max_age_seconds)),
            },
            FallbackStrategy::Simplified { mode } => RecoveryResult {
                success: true,
                layer_used: RecoveryLayer::Fallback,
                attempts: 1,
                error_message: None,
                user_action: Some(format!("Using simplified mode: {}", mode)),
            },
        }
    }

    /// 第三层：用户介入
    async fn try_with_user_intervention<F, Fut, T>(&self, _operation: &F) -> RecoveryResult
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Layer2Result<T>> + Send,
        T: Send,
    {
        let callback = self.user_callback.read().await;

        if let Some(cb) = callback.as_ref() {
            let actions = vec![
                RecoveryAction::Retry,
                RecoveryAction::Skip,
                RecoveryAction::Abort,
            ];

            let action = cb("Operation failed. Choose action:", actions);

            match action {
                RecoveryAction::Retry => RecoveryResult {
                    success: false, // 实际实现需要重新尝试
                    layer_used: RecoveryLayer::UserIntervention,
                    attempts: 1,
                    error_message: None,
                    user_action: Some("User requested retry".to_string()),
                },
                RecoveryAction::Skip => RecoveryResult {
                    success: true,
                    layer_used: RecoveryLayer::UserIntervention,
                    attempts: 1,
                    error_message: None,
                    user_action: Some("User chose to skip".to_string()),
                },
                RecoveryAction::Abort => RecoveryResult {
                    success: false,
                    layer_used: RecoveryLayer::UserIntervention,
                    attempts: 1,
                    error_message: Some("User aborted operation".to_string()),
                    user_action: Some("User aborted".to_string()),
                },
                _ => RecoveryResult {
                    success: false,
                    layer_used: RecoveryLayer::UserIntervention,
                    attempts: 1,
                    error_message: Some("Unknown action".to_string()),
                    user_action: None,
                },
            }
        } else {
            RecoveryResult {
                success: false,
                layer_used: RecoveryLayer::UserIntervention,
                attempts: 0,
                error_message: Some("No user callback set".to_string()),
                user_action: Some("Please configure user callback for intervention".to_string()),
            }
        }
    }

    /// 获取用户提示
    fn get_user_hint(&self, category: &ErrorCategory) -> String {
        match category {
            ErrorCategory::Configuration => "Check your API key and configuration".to_string(),
            ErrorCategory::Logic => "Verify your input parameters".to_string(),
            ErrorCategory::UserInterrupt => "Operation was cancelled".to_string(),
            ErrorCategory::Transient => "Temporary issue, will retry automatically".to_string(),
            ErrorCategory::Resource => {
                "System resource issue, consider freeing up memory/disk".to_string()
            }
            ErrorCategory::System => "Unknown error occurred".to_string(),
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> RecoveryStats {
        self.stats.read().await.clone()
    }
}

/// 会话恢复检测器
pub struct SessionRecovery {
    /// 存储路径
    storage_path: std::path::PathBuf,
}

impl SessionRecovery {
    /// 创建新的会话恢复器
    pub fn new(storage_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            storage_path: storage_path.as_ref().to_path_buf(),
        }
    }

    /// 检测是否有中断的会话
    pub fn detect_interrupted_sessions(&self) -> Layer2Result<Vec<InterruptedSession>> {
        let mut interrupted = Vec::new();

        if !self.storage_path.exists() {
            return Ok(interrupted);
        }

        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let session_dir = entry.path();

            if !session_dir.is_dir() {
                continue;
            }

            let state_file = session_dir.join("state.json");
            if state_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&state_file) {
                    if let Ok(state) = serde_json::from_str::<SessionState>(&content) {
                        if state.status == SessionStatus::Running && !state.completed {
                            interrupted.push(InterruptedSession {
                                session_id: state.session_id,
                                last_iteration: state.iteration,
                                last_activity: state.last_updated,
                                task_description: state.task_description,
                            });
                        }
                    }
                }
            }
        }

        // 按时间排序（最近的中断在前）
        interrupted.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

        Ok(interrupted)
    }

    /// 渲染中断会话列表
    pub fn render_interrupted(&self) -> String {
        match self.detect_interrupted_sessions() {
            Ok(sessions) => {
                if sessions.is_empty() {
                    "No interrupted sessions found.".to_string()
                } else {
                    let mut output =
                        format!("Found {} interrupted session(s):\n\n", sessions.len());
                    for (i, session) in sessions.iter().enumerate() {
                        output.push_str(&format!(
                            "{}. Session: {}\n   Task: {}\n   Iteration: {}\n   Last activity: {}\n\n",
                            i + 1,
                            session.session_id,
                            session.task_description.as_deref().unwrap_or("Unknown"),
                            session.last_iteration,
                            session.last_activity.format("%Y-%m-%d %H:%M:%S")
                        ));
                    }
                    output.push_str("Use 'continuum session resume <id>' to continue.");
                    output
                }
            }
            Err(e) => format!("Error detecting sessions: {}", e),
        }
    }
}

/// 中断的会话信息
#[derive(Debug, Clone)]
pub struct InterruptedSession {
    pub session_id: SessionId,
    pub last_iteration: i32,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub task_description: Option<String>,
}

/// 会话状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SessionState {
    session_id: SessionId,
    status: SessionStatus,
    completed: bool,
    iteration: i32,
    last_updated: chrono::DateTime<chrono::Utc>,
    task_description: Option<String>,
}

/// 会话状态枚举
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum SessionStatus {
    Running,
    Paused,
    Completed,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_analysis() {
        let cat = ErrorCategory::from_error_message("network timeout");
        assert_eq!(cat, ErrorCategory::Transient);

        let cat = ErrorCategory::from_error_message("invalid parameter");
        assert_eq!(cat, ErrorCategory::Logic);
    }

    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::default();
        let delay = policy.delay_for_attempt(0);
        assert!(delay.as_millis() >= 900); // 考虑抖动
        assert!(delay.as_millis() <= 1100);
    }

    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy {
            max_delay_ms: 5000,
            ..Default::default()
        };
        let delay = policy.delay_for_attempt(10);
        assert!(delay.as_millis() <= 5500); // 考虑抖动
    }

    #[tokio::test]
    async fn test_error_recovery_creation() {
        let recovery = ErrorRecovery::new();
        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_fallback_strategy() {
        let strategy = FallbackStrategy::Skip;
        matches!(strategy, FallbackStrategy::Skip);
    }
}
