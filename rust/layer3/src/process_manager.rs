//! # Process Manager
//!
//! 进程管理器：管理后台进程的生命周期。

use crate::types::{Layer3Result, ProcessInfo, ProcessState};
use async_trait::async_trait;

/// 进程管理器 trait
///
/// 定义进程管理核心接口。
#[async_trait]
pub trait ProcessManager: Send + Sync {
    /// 启动新进程
    ///
    /// # Arguments
    /// * `command` - 命令行
    /// * `args` - 参数
    /// * `working_dir` - 工作目录
    ///
    /// # Returns
    /// * `u32` - 进程 PID
    async fn start(
        &self,
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
    ) -> Layer3Result<u32>;

    /// 停止进程
    ///
    /// # Arguments
    /// * `pid` - 进程 PID
    /// * `force` - 是否强制终止
    async fn stop(&self, pid: u32, force: bool) -> Layer3Result<bool>;

    /// 获取进程信息
    async fn get_info(&self, pid: u32) -> Layer3Result<Option<ProcessInfo>>;

    /// 获取进程状态
    async fn get_state(&self, pid: u32) -> Layer3Result<ProcessState>;

    /// 检查进程是否存活
    async fn is_alive(&self, pid: u32) -> Layer3Result<bool>;

    /// 等待进程结束
    async fn wait(&self, pid: u32) -> Layer3Result<i32>;

    /// 列出所有管理的进程
    async fn list(&self) -> Layer3Result<Vec<ProcessInfo>>;

    /// 列出指定状态的进程
    async fn list_by_state(&self, state: ProcessState) -> Layer3Result<Vec<ProcessInfo>>;

    /// 发送信号到进程
    async fn signal(&self, pid: u32, signal: ProcessSignal) -> Layer3Result<bool>;
}

/// 进程信号
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSignal {
    /// 终止信号 (SIGTERM)
    Terminate,
    /// 强制终止 (SIGKILL)
    Kill,
    /// 暂停 (SIGSTOP)
    Stop,
    /// 继续 (SIGCONT)
    Continue,
    /// 中断 (SIGINT)
    Interrupt,
}

/// 进程输出捕获 trait
#[async_trait]
pub trait OutputCapture: Send + Sync {
    /// 获取进程 stdout
    async fn get_stdout(&self, pid: u32) -> Layer3Result<String>;

    /// 获取进程 stderr
    async fn get_stderr(&self, pid: u32) -> Layer3Result<String>;

    /// 流式读取输出
    async fn stream_output(&self, pid: u32) -> Layer3Result<ProcessOutputStream>;
}

/// 进程输出流
#[derive(Debug)]
pub struct ProcessOutputStream {
    /// stdout 行
    pub stdout_lines: Vec<String>,
    /// stderr 行
    pub stderr_lines: Vec<String>,
    /// 是否结束
    pub finished: bool,
}

/// 进程监控 trait
#[async_trait]
pub trait ProcessMonitor: Send + Sync {
    /// 获取 CPU 使用率
    async fn cpu_usage(&self, pid: u32) -> Layer3Result<f32>;

    /// 获取内存使用
    async fn memory_usage(&self, pid: u32) -> Layer3Result<u64>;

    /// 获取运行时间
    async fn runtime(&self, pid: u32) -> Layer3Result<u64>;

    /// 设置资源限制
    async fn set_limits(&self, pid: u32, limits: ProcessLimits) -> Layer3Result<bool>;
}

/// 进程资源限制
#[derive(Debug, Clone, Default)]
pub struct ProcessLimits {
    /// 最大 CPU 时间（秒）
    pub max_cpu_secs: Option<u64>,
    /// 最大内存（字节）
    pub max_memory_bytes: Option<u64>,
    /// 最大运行时间（秒）
    pub max_runtime_secs: Option<u64>,
    /// 最大输出大小（字节）
    pub max_output_bytes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_limits_default() {
        let limits = ProcessLimits::default();
        assert!(limits.max_cpu_secs.is_none());
        assert!(limits.max_memory_bytes.is_none());
    }

    #[test]
    fn test_process_signal() {
        assert_eq!(ProcessSignal::Terminate, ProcessSignal::Terminate);
    }
}
