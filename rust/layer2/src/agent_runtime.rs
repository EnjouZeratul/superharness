//! # Agent Runtime
//!
//! Agent 执行运行时接口定义。

use async_trait::async_trait;

use crate::types::{AgentId, AgentState, Layer2Result, Message, SessionId, ToolCall, ToolResult};

/// Agent 执行结果
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub session_id: SessionId,
    pub final_state: AgentState,
    pub messages: Vec<Message>,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
    pub iterations: i32,
    pub tokens_used: i64,
}

/// Agent 配置
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub agent_id: AgentId,
    pub model: String,
    pub temperature: f32,
    pub max_iterations: i32,
    pub system_prompt: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: AgentId::new(),
            model: "gpt-4o".to_string(),
            temperature: 0.7,
            max_iterations: 100,
            system_prompt: None,
        }
    }
}

/// Agent 运行时接口
///
/// 定义 Agent 执行的核心生命周期操作。
#[async_trait]
pub trait AgentRuntimeTrait: Send + Sync {
    /// 启动 Agent 执行
    ///
    /// # Arguments
    /// * `task` - 用户任务描述
    /// * `config` - Agent 配置
    ///
    /// # Returns
    /// 执行结果，包含最终状态和输出
    async fn run(&self, task: &str, config: AgentConfig) -> Layer2Result<AgentResult>;

    /// 启动 Agent 并返回会话 ID（用于流式执行）
    ///
    /// # Arguments
    /// * `task` - 用户任务描述
    /// * `config` - Agent 配置
    ///
    /// # Returns
    /// 会话 ID，用于后续操作
    async fn start(&self, task: &str, config: AgentConfig) -> Layer2Result<SessionId>;

    /// 暂停正在执行的 Agent
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    async fn pause(&self, session_id: &SessionId) -> Layer2Result<()>;

    /// 恢复暂停的 Agent
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    async fn resume(&self, session_id: &SessionId) -> Layer2Result<()>;

    /// 停止 Agent 执行
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    async fn stop(&self, session_id: &SessionId) -> Layer2Result<()>;

    /// 获取 Agent 当前状态
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    fn status(&self, session_id: &SessionId) -> Layer2Result<AgentState>;

    /// 向 Agent 发送消息
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    /// * `message` - 消息内容
    async fn send_message(&self, session_id: &SessionId, message: &str) -> Layer2Result<()>;

    /// 获取 Agent 的工具调用结果
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    /// * `tool_call_id` - 工具调用 ID
    async fn submit_tool_result(
        &self,
        session_id: &SessionId,
        tool_call_id: &str,
        result: ToolResult,
    ) -> Layer2Result<()>;
}

/// Agent 执行循环回调接口
///
/// 用于在执行过程中注入自定义逻辑。
#[async_trait]
pub trait AgentLoopCallback: Send + Sync {
    /// 在每次迭代前调用
    async fn before_iteration(&self, session_id: &SessionId, iteration: i32) -> Layer2Result<bool>;

    /// 在每次迭代后调用
    async fn after_iteration(
        &self,
        session_id: &SessionId,
        iteration: i32,
        result: &IterationResult,
    ) -> Layer2Result<()>;

    /// 在工具调用前调用
    async fn before_tool_call(
        &self,
        session_id: &SessionId,
        tool_call: &ToolCall,
    ) -> Layer2Result<bool>;

    /// 在工具调用后调用
    async fn after_tool_call(
        &self,
        session_id: &SessionId,
        tool_call: &ToolCall,
        result: &ToolResult,
    ) -> Layer2Result<()>;
}

/// 单次迭代结果
#[derive(Debug, Clone)]
pub struct IterationResult {
    pub iteration: i32,
    pub state: AgentState,
    pub message: Option<Message>,
    pub tool_calls: Vec<ToolCall>,
    pub should_continue: bool,
}

/// 默认 Agent Runtime 实现（占位）
pub struct AgentRuntime {
    // 内部状态将在实现时添加
    _marker: std::marker::PhantomData<()>,
}

impl AgentRuntime {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for AgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentRuntimeTrait for AgentRuntime {
    async fn run(&self, _task: &str, _config: AgentConfig) -> Layer2Result<AgentResult> {
        // TODO: 实现完整执行逻辑
        Err(anyhow::anyhow!("AgentRuntime::run not implemented"))
    }

    async fn start(&self, _task: &str, _config: AgentConfig) -> Layer2Result<SessionId> {
        // TODO: 实现启动逻辑
        Err(anyhow::anyhow!("AgentRuntime::start not implemented"))
    }

    async fn pause(&self, _session_id: &SessionId) -> Layer2Result<()> {
        // TODO: 实现暂停逻辑
        Err(anyhow::anyhow!("AgentRuntime::pause not implemented"))
    }

    async fn resume(&self, _session_id: &SessionId) -> Layer2Result<()> {
        // TODO: 实现恢复逻辑
        Err(anyhow::anyhow!("AgentRuntime::resume not implemented"))
    }

    async fn stop(&self, _session_id: &SessionId) -> Layer2Result<()> {
        // TODO: 实现停止逻辑
        Err(anyhow::anyhow!("AgentRuntime::stop not implemented"))
    }

    fn status(&self, _session_id: &SessionId) -> Layer2Result<AgentState> {
        // TODO: 实现状态查询逻辑
        Err(anyhow::anyhow!("AgentRuntime::status not implemented"))
    }

    async fn send_message(&self, _session_id: &SessionId, _message: &str) -> Layer2Result<()> {
        // TODO: 实现消息发送逻辑
        Err(anyhow::anyhow!(
            "AgentRuntime::send_message not implemented"
        ))
    }

    async fn submit_tool_result(
        &self,
        _session_id: &SessionId,
        _tool_call_id: &str,
        _result: ToolResult,
    ) -> Layer2Result<()> {
        // TODO: 实现工具结果提交逻辑
        Err(anyhow::anyhow!(
            "AgentRuntime::submit_tool_result not implemented"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.max_iterations, 100);
    }

    #[test]
    fn test_agent_runtime_creation() {
        let runtime = AgentRuntime::new();
        // 验证创建成功
        let _ = runtime;
    }
}
