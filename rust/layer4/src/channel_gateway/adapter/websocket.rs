//! # WebSocket Channel Adapter
//!
//! WebSocket 实时通信渠道适配器。

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::channel_gateway::{Channel, ChannelType, InboundMessage, OutboundMessage};
use crate::types::Layer4Result;

/// WebSocket 渠道配置
pub struct WebSocketChannelConfig {
    pub url: String,
    pub reconnect_attempts: u32,
    pub ping_interval_ms: u64,
}

impl Default for WebSocketChannelConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:8080/ws".to_string(),
            reconnect_attempts: 3,
            ping_interval_ms: 30000,
        }
    }
}

/// WebSocket 渠道适配器
pub struct WebSocketChannel {
    channel_id: String,
    #[allow(dead_code)]
    config: WebSocketChannelConfig,
    connected: RwLock<bool>,
    message_queue: RwLock<VecDeque<InboundMessage>>,
    sessions: RwLock<HashMap<String, String>>, // session_id -> user_id
}

impl WebSocketChannel {
    /// 创建新的 WebSocket 渠道
    pub fn new(channel_id: impl Into<String>, config: WebSocketChannelConfig) -> Self {
        Self {
            channel_id: channel_id.into(),
            config,
            connected: RwLock::new(true),
            message_queue: RwLock::new(VecDeque::new()),
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// 创建默认 WebSocket 渠道
    pub fn default_channel() -> Self {
        Self::new("ws-default", WebSocketChannelConfig::default())
    }

    /// 注册会话
    pub fn register_session(&self, session_id: &str, user_id: &str) {
        self.sessions
            .write()
            .insert(session_id.to_string(), user_id.to_string());
    }

    /// 注销会话
    pub fn unregister_session(&self, session_id: &str) {
        self.sessions.write().remove(session_id);
    }

    /// 接收 WebSocket 消息（模拟）
    pub fn receive_message(&self, session_id: &str, content: &str) {
        let user_id = self
            .sessions
            .read()
            .get(session_id)
            .cloned()
            .unwrap_or_default();
        let message = InboundMessage::new(&self.channel_id, &user_id, content)
            .with_session(session_id)
            .with_metadata(serde_json::json!({
                "source": "websocket",
                "session_id": session_id
            }));
        self.message_queue.write().push_back(message);
    }

    /// 获取活跃会话数量
    pub fn active_sessions(&self) -> usize {
        self.sessions.read().len()
    }
}

#[async_trait]
impl Channel for WebSocketChannel {
    fn id(&self) -> &str {
        &self.channel_id
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::WebSocket
    }

    async fn send(&self, message: &OutboundMessage) -> Layer4Result<()> {
        if !*self.connected.read() {
            return Err(anyhow::anyhow!("Channel not connected"));
        }

        // 实际 WebSocket 发送需要 WebSocket 客户端
        // 这里是占位实现
        tracing::debug!("WebSocket channel sending: {}", message.content);

        Ok(())
    }

    async fn try_receive(&self) -> Layer4Result<Option<InboundMessage>> {
        if !*self.connected.read() {
            return Err(anyhow::anyhow!("Channel not connected"));
        }

        Ok(self.message_queue.write().pop_front())
    }

    fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    async fn close(&self) -> Layer4Result<()> {
        *self.connected.write() = false;
        self.message_queue.write().clear();
        self.sessions.write().clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_channel_creation() {
        let channel = WebSocketChannel::default_channel();
        assert_eq!(channel.id(), "ws-default");
        assert!(channel.is_connected());
    }

    #[test]
    fn test_websocket_session_management() {
        let channel = WebSocketChannel::default_channel();
        channel.register_session("session-1", "user-1");

        assert_eq!(channel.active_sessions(), 1);

        channel.unregister_session("session-1");
        assert_eq!(channel.active_sessions(), 0);
    }

    #[test]
    fn test_websocket_receive_message() {
        let channel = WebSocketChannel::default_channel();
        channel.register_session("session-1", "user-1");
        channel.receive_message("session-1", "Hello");

        let count = channel.message_queue.read().len();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_websocket_channel_close() {
        let channel = WebSocketChannel::default_channel();
        channel.register_session("session-1", "user-1");
        channel.close().await.unwrap();

        assert!(!channel.is_connected());
        assert_eq!(channel.active_sessions(), 0);
    }
}
