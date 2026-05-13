//! # Sandbox Runtime
//!
//! 沙箱运行时：安全隔离的执行环境。

use crate::types::{Layer3Result, ToolRequest, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// 沙箱运行时 trait
///
/// 提供安全隔离的代码执行环境。
#[async_trait]
pub trait SandboxRuntime: Send + Sync {
    /// 创建沙箱
    async fn create(&self, config: SandboxConfig) -> Layer3Result<SandboxId>;

    /// 销毁沙箱
    async fn destroy(&self, id: &SandboxId) -> Layer3Result<bool>;

    /// 在沙箱中执行代码
    async fn execute(
        &self,
        id: &SandboxId,
        code: &str,
        language: &str,
    ) -> Layer3Result<ExecutionResult>;

    /// 在沙箱中执行工具
    async fn execute_tool(
        &self,
        id: &SandboxId,
        request: ToolRequest,
    ) -> Layer3Result<ToolResponse>;

    /// 获取沙箱状态
    async fn status(&self, id: &SandboxId) -> Layer3Result<SandboxStatus>;

    /// 获取沙箱信息
    async fn info(&self, id: &SandboxId) -> Layer3Result<Option<SandboxInfo>>;

    /// 列出所有活跃沙箱
    async fn list(&self) -> Layer3Result<Vec<SandboxInfo>>;

    /// 重置沙箱
    async fn reset(&self, id: &SandboxId) -> Layer3Result<bool>;
}

/// 沙箱 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SandboxId(pub String);

impl std::fmt::Display for SandboxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 沙箱配置
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// 基础镜像/环境
    pub base_image: String,
    /// 资源限制
    pub limits: SandboxLimits,
    /// 允许的网络访问
    pub network: NetworkPolicy,
    /// 允许的文件系统访问
    pub filesystem: FsPolicy,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 工作目录
    pub working_dir: PathBuf,
    /// 最大执行时间（秒）
    pub timeout_secs: u64,
    /// 是否允许交互
    pub interactive: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            base_image: "default".to_string(),
            limits: SandboxLimits::default(),
            network: NetworkPolicy::Disabled,
            filesystem: FsPolicy::ReadOnly,
            env_vars: HashMap::new(),
            working_dir: PathBuf::from("/sandbox"),
            timeout_secs: 30,
            interactive: false,
        }
    }
}

/// 沙箱资源限制
#[derive(Debug, Clone, Default)]
pub struct SandboxLimits {
    /// 最大内存（字节）
    pub max_memory: Option<u64>,
    /// 最大 CPU（百分比）
    pub max_cpu_percent: Option<u32>,
    /// 最大文件大小（字节）
    pub max_file_size: Option<u64>,
    /// 最大进程数
    pub max_processes: Option<u32>,
}

/// 网络策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkPolicy {
    /// 禁止网络
    Disabled,
    /// 仅允许出站
    OutboundOnly,
    /// 允许特定端口
    RestrictedPorts(Vec<u16>),
    /// 完全开放（危险）
    Full,
}

/// 文件系统策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsPolicy {
    /// 只读
    ReadOnly,
    /// 仅允许特定目录
    RestrictedDirs(Vec<PathBuf>),
    /// 临时可写（执行后清空）
    TempWritable,
    /// 完全可写（危险）
    FullWritable,
}

/// 沙箱状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxStatus {
    Creating,
    Ready,
    Running,
    Paused,
    Error,
    Destroyed,
}

/// 沙箱信息
#[derive(Debug, Clone)]
pub struct SandboxInfo {
    /// 沙箱 ID
    pub id: SandboxId,
    /// 状态
    pub status: SandboxStatus,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 当前内存使用
    pub memory_used: u64,
    /// 当前 CPU 使用
    pub cpu_used: f32,
    /// 执行次数
    pub executions: u32,
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: i32,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    /// 是否超时
    pub timed_out: bool,
    /// 是否被杀
    pub killed: bool,
}

impl ExecutionResult {
    /// 创建成功结果
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 0,
            timed_out: false,
            killed: false,
        }
    }

    /// 创建失败结果
    pub fn failure(stderr: String, exit_code: i32) -> Self {
        Self {
            stdout: String::new(),
            stderr,
            exit_code,
            duration_ms: 0,
            timed_out: false,
            killed: false,
        }
    }

    /// 创建超时结果
    pub fn timeout(stdout: String, stderr: String) -> Self {
        Self {
            stdout,
            stderr,
            exit_code: -1,
            duration_ms: 0,
            timed_out: true,
            killed: false,
        }
    }

    /// 是否成功
    pub fn is_success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out && !self.killed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.network, NetworkPolicy::Disabled);
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult::success("hello".to_string());
        assert!(result.is_success());
    }

    #[test]
    fn test_execution_result_timeout() {
        let result = ExecutionResult::timeout("out".to_string(), "err".to_string());
        assert!(result.timed_out);
        assert!(!result.is_success());
    }
}
