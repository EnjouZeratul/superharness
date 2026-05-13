//! # Session Definition
//!
//! 会话结构定义。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{AgentId, AgentState, Message, SessionId, ToolCall, ToolResult};

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub model: String,
    pub temperature: f32,
    pub max_iterations: i32,
    pub system_prompt: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            temperature: 0.7,
            max_iterations: 100,
            system_prompt: None,
        }
    }
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// 会话 ID
    pub session_id: SessionId,
    /// Agent ID
    pub agent_id: AgentId,
    /// 当前状态
    pub state: AgentState,
    /// 当前迭代次数
    pub iteration: i32,
    /// 最大迭代次数
    pub max_iterations: i32,
    /// 消息历史
    pub messages: Vec<Message>,
    /// 已注册工具
    pub tools_registered: Vec<String>,
    /// 待处理的工具调用
    pub tool_calls_pending: Vec<ToolCall>,
    /// 工具结果缓存
    pub tool_results_cache: Vec<ToolResult>,
    /// 模型名称
    pub model: String,
    /// 温度参数
    pub temperature: f32,
    /// 系统提示词
    pub system_prompt: String,
    /// Token 使用统计
    pub tokens_total: i64,
    pub tokens_prompt: i64,
    pub tokens_completion: i64,
    /// 成本估算
    pub cost_estimate: f64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 检查点计数
    pub checkpoint_count: i32,
}

impl Session {
    /// 创建新会话
    pub fn new(config: &SessionConfig) -> Self {
        let now = Utc::now();
        Self {
            session_id: SessionId::new(),
            agent_id: AgentId::new(),
            state: AgentState::Idle,
            iteration: 0,
            max_iterations: config.max_iterations,
            messages: Vec::new(),
            tools_registered: Vec::new(),
            tool_calls_pending: Vec::new(),
            tool_results_cache: Vec::new(),
            model: config.model.clone(),
            temperature: config.temperature,
            system_prompt: config.system_prompt.clone().unwrap_or_default(),
            tokens_total: 0,
            tokens_prompt: 0,
            tokens_completion: 0,
            cost_estimate: 0.0,
            created_at: now,
            last_updated: now,
            checkpoint_count: 0,
        }
    }

    /// 添加用户消息
    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(Message::user(content));
        self.iteration += 1;
        self.touch();
    }

    /// 添加助手消息
    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(Message::assistant(content));
        self.touch();
    }

    /// 添加系统消息
    pub fn add_system_message(&mut self, content: &str) {
        self.messages.push(Message::system(content));
        self.touch();
    }

    /// 更新最后修改时间
    pub fn touch(&mut self) {
        self.last_updated = Utc::now();
    }

    /// 检查是否可以继续执行
    pub fn can_continue(&self) -> bool {
        self.iteration < self.max_iterations
            && matches!(self.state, AgentState::Running | AgentState::Idle)
    }

    /// 序列化为 JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// 从 JSON 反序列化
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    /// 转换为字典（兼容 Python 版本）
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new(&SessionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let config = SessionConfig::default();
        let session = Session::new(&config);

        assert!(session.messages.is_empty());
        assert_eq!(session.state, AgentState::Idle);
        assert_eq!(session.iteration, 0);
    }

    #[test]
    fn test_session_messages() {
        let config = SessionConfig::default();
        let mut session = Session::new(&config);

        session.add_user_message("Hello");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.iteration, 1);

        session.add_assistant_message("Hi there!");
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_session_can_continue() {
        let mut config = SessionConfig::default();
        config.max_iterations = 5;

        let mut session = Session::new(&config);
        assert!(session.can_continue());

        session.state = AgentState::Running;
        assert!(session.can_continue());

        session.state = AgentState::Stopped;
        assert!(!session.can_continue());
    }

    #[test]
    fn test_session_serialization() {
        let config = SessionConfig::default();
        let session = Session::new(&config);

        let json = session.to_json().unwrap();
        let restored = Session::from_json(&json).unwrap();

        assert_eq!(session.session_id, restored.session_id);
        assert_eq!(session.state, restored.state);
    }
}
