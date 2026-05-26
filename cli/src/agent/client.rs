//! Agent 客户端实现
//!
//! 连接真实 LLM API 进行对话。

use anyhow::Result;
use sh_layer4::sh_layer3::sh_layer2::sh_layer1::{
    config_manager::ConfigManager,
    llm_client::{LlmClient, LlmClientTrait, LlmProvider, LlmRequestConfig, Message, MessageRole},
    streaming::{StreamEvent as LlmStreamEvent, ContentDelta},
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Agent 状态
#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    /// 空闲状态
    Idle,
    /// 正在处理
    Processing,
    /// 错误状态
    Error,
}

/// 流式事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 开始
    Start,
    /// 响应块
    Chunk(String),
    /// 完成
    Done,
    /// 错误
    Error(String),
}

/// 聊天消息
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// 角色
    pub role: String,
    /// 内容
    pub content: String,
}

/// Agent 错误类型
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum AgentError {
    /// 配置错误（无API密钥等）
    ConfigError(String),
    /// API 调用错误
    ApiError(String),
    /// 网络错误
    NetworkError(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            AgentError::ApiError(msg) => write!(f, "API error: {}", msg),
            AgentError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for AgentError {}

/// Agent 客户端
///
/// 提供真实的 LLM Agent 功能，连接真实的 API。
pub struct AgentClient {
    /// 配置管理器
    config: Arc<RwLock<ConfigManager>>,
    /// LLM 客户端
    llm_client: Arc<RwLock<Option<LlmClient>>>,
    /// 当前状态
    state: Arc<RwLock<AgentState>>,
    /// 消息历史
    message_history: Arc<RwLock<Vec<ChatMessage>>>,
    /// 当前提供商名称
    current_provider: Arc<RwLock<String>>,
}

impl AgentClient {
    /// 创建新的 Agent 客户端
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ConfigManager::new())),
            llm_client: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(AgentState::Idle)),
            message_history: Arc::new(RwLock::new(Vec::new())),
            current_provider: Arc::new(RwLock::new(String::new())),
        }
    }

    /// 从配置文件初始化
    pub async fn init_from_config(&self) -> Result<(), AgentError> {
        // 加载完整配置
        let config = ConfigManager::load_full()
            .await
            .map_err(|e| AgentError::ConfigError(format!("Failed to load config: {}", e)))?;

        // 解析环境变量引用
        let mut config = config;
        config.resolve_env_refs();

        // 检查是否有配置的提供商
        if config.providers.is_empty() {
            return Err(AgentError::ConfigError(
                "No providers configured. Use 'continuum config add-provider' or set environment variables.".to_string()
            ));
        }

        // 获取当前提供商
        let provider_name = config.active_provider.clone();
        let provider_config = config.providers.get(&provider_name).cloned();

        if provider_config.is_none() {
            return Err(AgentError::ConfigError(format!(
                "Active provider '{}' not found in configuration",
                provider_name
            )));
        }

        let provider_config = provider_config.unwrap();

        // 检查 API 密钥
        if provider_config.api_key.is_empty() {
            return Err(AgentError::ConfigError(
                format!("API key not set for provider '{}'. Use 'continuum config set provider.{}.api_key YOUR_KEY' or set environment variable.",
                    provider_name, provider_name)
            ));
        }

        // 创建 LLM 客户端
        let llm_provider = Self::map_provider(&provider_name, &provider_config.base_url);
        let llm_client = LlmClient::new(llm_provider, provider_config.api_key.clone());

        // 更新状态
        {
            let mut cfg = self.config.write().await;
            *cfg = config;
        }
        {
            let mut client = self.llm_client.write().await;
            *client = Some(llm_client);
        }
        {
            let mut p = self.current_provider.write().await;
            *p = provider_name.clone();
        }

        tracing::info!("AgentClient initialized with provider: {}", provider_name);
        Ok(())
    }

    /// 映射提供商名称到 LlmProvider
    fn map_provider(name: &str, base_url: &str) -> LlmProvider {
        match name {
            "anthropic" => LlmProvider::Anthropic,
            "openai" => LlmProvider::OpenAI,
            "gemini" => LlmProvider::Gemini,
            _ => LlmProvider::Custom(base_url.to_string()),
        }
    }

    /// 发送消息并获取响应
    pub async fn send_message(&self, user_message: &str) -> Result<String, AgentError> {
        // 检查客户端是否初始化
        let client_guard = self.llm_client.read().await;
        if client_guard.is_none() {
            return Err(AgentError::ConfigError(
                "Agent not initialized. Call init_from_config() first.".to_string(),
            ));
        }

        // 设置状态为处理中
        {
            let mut state = self.state.write().await;
            *state = AgentState::Processing;
        }

        // 添加用户消息到历史
        {
            let mut history = self.message_history.write().await;
            history.push(ChatMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            });
        }

        // 构建请求
        let config = self.config.read().await;
        let provider_config = config
            .current()
            .map_err(|e| AgentError::ConfigError(e.to_string()))?;

        let request_config = LlmRequestConfig {
            model: provider_config.model.clone(),
            max_tokens: provider_config.default_max_tokens,
            temperature: provider_config.default_temperature,
            system_prompt: Some("You are a helpful AI assistant running in Continuum terminal. Be concise and helpful.".to_string()),
            stop_sequences: vec![],
        };

        // 构建消息历史
        let history = self.message_history.read().await;
        let messages: Vec<Message> = history
            .iter()
            .map(|m| Message {
                role: if m.role == "user" {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                content: m.content.clone(),
            })
            .collect();

        // 发送请求
        let client = client_guard.as_ref().unwrap();
        let result = client.send(messages, &request_config).await;

        // 处理响应
        match result {
            Ok(response) => {
                // 添加助手响应到历史
                {
                    let mut history = self.message_history.write().await;
                    history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: response.content.clone(),
                    });
                }

                // 设置状态为空闲
                {
                    let mut state = self.state.write().await;
                    *state = AgentState::Idle;
                }

                tracing::debug!(
                    "Agent response: {} tokens used",
                    response.usage.input_tokens + response.usage.output_tokens
                );
                Ok(response.content)
            }
            Err(e) => {
                // 设置状态为错误
                {
                    let mut state = self.state.write().await;
                    *state = AgentState::Error;
                }

                // 分类错误类型
                let error_msg = e.to_string();
                if error_msg.contains("connection") || error_msg.contains("network") {
                    Err(AgentError::NetworkError(error_msg))
                } else {
                    Err(AgentError::ApiError(error_msg))
                }
            }
        }
    }

    /// 获取当前状态
    pub async fn state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    /// 获取当前提供商名称
    pub async fn current_provider(&self) -> String {
        self.current_provider.read().await.clone()
    }

    /// 获取消息历史
    pub async fn message_history(&self) -> Vec<ChatMessage> {
        self.message_history.read().await.clone()
    }

    /// 清空消息历史
    pub async fn clear_history(&self) {
        let mut history = self.message_history.write().await;
        history.clear();
    }

    /// 获取配置信息
    pub async fn config_info(&self) -> String {
        let config = self.config.read().await;
        let provider = self.current_provider.read().await.clone();

        if let Some(provider_config) = config.providers.get(&provider) {
            format!(
                "Provider: {} | Model: {} | MaxTokens: {}",
                provider, provider_config.model, provider_config.default_max_tokens
            )
        } else {
            format!("Provider: {} (not configured)", provider)
        }
    }

    /// 检查是否已初始化
    pub async fn is_initialized(&self) -> bool {
        self.llm_client.read().await.is_some()
    }

    /// 发送消息并流式获取响应
    ///
    /// 使用真实的 SSE 流式响应，通过 MessageStream 处理。
    /// 返回一个接收器，通过该接收器可以接收流式响应块
    pub async fn send_message_stream(
        &self,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamEvent>, AgentError> {
        // 检查客户端是否初始化
        let client_guard = self.llm_client.read().await;
        if client_guard.is_none() {
            return Err(AgentError::ConfigError(
                "Agent not initialized. Call init_from_config() first.".to_string(),
            ));
        }

        // 添加用户消息到历史
        {
            let mut history = self.message_history.write().await;
            history.push(ChatMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            });
        }

        // 构建请求配置
        let config = self.config.read().await;
        let provider_config = config
            .current()
            .map_err(|e| AgentError::ConfigError(e.to_string()))?;

        let request_config = LlmRequestConfig {
            model: provider_config.model.clone(),
            max_tokens: provider_config.default_max_tokens,
            temperature: provider_config.default_temperature,
            system_prompt: Some(
                "You are a helpful AI assistant running in Continuum terminal. Be concise and helpful."
                    .to_string(),
            ),
            stop_sequences: vec![],
        };

        // 构建消息历史
        let history = self.message_history.read().await;
        let messages: Vec<Message> = history
            .iter()
            .map(|m| Message {
                role: if m.role == "user" {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                content: m.content.clone(),
            })
            .collect();

        // 创建通道
        let (tx, rx) = mpsc::channel(64);

        // 克隆必要的数据
        let llm_client = self.llm_client.clone();
        let message_history = self.message_history.clone();
        let state = self.state.clone();
        let messages_clone = messages;
        let request_config_clone = request_config;

        // 设置状态为处理中
        {
            let mut s = state.write().await;
            *s = AgentState::Processing;
        }

        // 发送开始事件
        let _ = tx.send(StreamEvent::Start).await;

        // 在后台任务中处理流式 API 调用
        tokio::spawn(async move {
            // 读取客户端
            let client_guard = llm_client.read().await;
            let client = match client_guard.as_ref() {
                Some(c) => c,
                None => {
                    let _ = tx
                        .send(StreamEvent::Error("Agent not initialized".to_string()))
                        .await;
                    let _ = tx.send(StreamEvent::Done).await;
                    let mut s = state.write().await;
                    *s = AgentState::Idle;
                    return;
                }
            };

            // 发送流式请求
            let stream_result = client.send_stream(messages_clone, &request_config_clone).await;

            match stream_result {
                Ok(mut message_stream) => {
                    let mut full_content = String::new();
                    let mut message_id = String::new();
                    let mut model_name = request_config_clone.model.clone();

                    // 处理流式事件
                    loop {
                        match message_stream.next_event().await {
                            Ok(Some(event)) => {
                                match event {
                                    LlmStreamEvent::MessageStart { id, model } => {
                                        message_id = id;
                                        model_name = model;
                                    }
                                    LlmStreamEvent::ContentBlockDelta { delta, .. } => {
                                        match delta {
                                            ContentDelta::Text(text) => {
                                                full_content.push_str(&text);
                                                if tx.send(StreamEvent::Chunk(text)).await.is_err() {
                                                    break;
                                                }
                                            }
                                            ContentDelta::Thinking(thinking) => {
                                                // 可选：发送思考内容作为特殊事件
                                                if tx.send(StreamEvent::Chunk(format!("[思考] {} ", thinking))).await.is_err() {
                                                    break;
                                                }
                                            }
                                            ContentDelta::ToolInput(input) => {
                                                // 工具调用输入
                                                if tx.send(StreamEvent::Chunk(format!("[工具] {} ", input))).await.is_err() {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    LlmStreamEvent::MessageDelta { stop_reason, .. } => {
                                        // 消息增量，记录结束原因
                                        tracing::debug!("Stream finished: {:?}", stop_reason);
                                    }
                                    LlmStreamEvent::MessageStop => {
                                        // 消息结束
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            Ok(None) => {
                                // 流结束
                                break;
                            }
                            Err(e) => {
                                let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                    }

                    // 添加助手响应到历史
                    if !full_content.is_empty() {
                        let mut history = message_history.write().await;
                        history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: full_content,
                        });
                    }

                    tracing::debug!("Stream completed: message_id={}, model={}", message_id, model_name);
                }
                Err(e) => {
                    let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                }
            }

            let _ = tx.send(StreamEvent::Done).await;

            // 设置状态为空闲
            let mut s = state.write().await;
            *s = AgentState::Idle;
        });

        Ok(rx)
    }

    /// 发送消息并使用回调处理流式响应
    ///
    /// 提供 on_chunk 回调函数，实时处理每个响应块
    pub async fn send_message_with_callback<F>(
        &self,
        user_message: &str,
        mut on_chunk: F,
    ) -> Result<String, AgentError>
    where
        F: FnMut(&str) + Send,
    {
        let mut receiver = self.send_message_stream(user_message).await?;

        let mut full_response = String::new();

        while let Some(event) = receiver.recv().await {
            match event {
                StreamEvent::Start => {
                    // 流开始
                }
                StreamEvent::Chunk(text) => {
                    on_chunk(&text);
                    full_response.push_str(&text);
                }
                StreamEvent::Done => {
                    break;
                }
                StreamEvent::Error(msg) => {
                    return Err(AgentError::ApiError(msg));
                }
            }
        }

        Ok(full_response)
    }
}

impl Default for AgentClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_client_creation() {
        let _client = AgentClient::new();
    }

    #[test]
    fn test_agent_error_display() {
        let error = AgentError::ConfigError("test error".to_string());
        assert!(error.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_chat_message_creation() {
        let message = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello");
    }

    #[tokio::test]
    async fn test_agent_state_initial() {
        let client = AgentClient::new();
        let state = client.state().await;
        assert_eq!(state, AgentState::Idle);
    }

    #[tokio::test]
    async fn test_message_history_empty() {
        let client = AgentClient::new();
        let history = client.message_history().await;
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_clear_history() {
        let client = AgentClient::new();
        client.clear_history().await;
        let history = client.message_history().await;
        assert!(history.is_empty());
    }
}
