//! # HTTP Channel Adapter
//!
//! HTTP REST API 渠道适配器。

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::VecDeque;

use crate::channel_gateway::{Channel, ChannelType, InboundMessage, OutboundMessage};
use crate::types::Layer4Result;

/// HTTP 渠道配置
pub struct HttpChannelConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub headers: HashMap<String, String>,
}

impl Default for HttpChannelConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout_ms: 30000,
            headers: HashMap::new(),
        }
    }
}

use std::collections::HashMap;

/// HTTP 渠道适配器
pub struct HttpChannel {
    channel_id: String,
    config: HttpChannelConfig,
    connected: RwLock<bool>,
    request_queue: RwLock<VecDeque<InboundMessage>>,
}

impl HttpChannel {
    /// 创建新的 HTTP 渠道
    pub fn new(channel_id: impl Into<String>, config: HttpChannelConfig) -> Self {
        Self {
            channel_id: channel_id.into(),
            config,
            connected: RwLock::new(true),
            request_queue: RwLock::new(VecDeque::new()),
        }
    }

    /// 创建默认 HTTP 渠道
    pub fn default_channel() -> Self {
        Self::new("http-default", HttpChannelConfig::default())
    }

    /// 处理 HTTP 请求（模拟接收）
    pub fn handle_request(&self, user_id: &str, content: &str) {
        let message = InboundMessage::new(&self.channel_id, user_id, content).with_metadata(
            serde_json::json!({
                "source": "http",
                "method": "POST"
            }),
        );
        self.request_queue.write().push_back(message);
    }

    /// 处理带会话的请求
    pub fn handle_request_with_session(&self, user_id: &str, content: &str, session_id: &str) {
        let message = InboundMessage::new(&self.channel_id, user_id, content)
            .with_session(session_id)
            .with_metadata(serde_json::json!({
                "source": "http",
                "method": "POST"
            }));
        self.request_queue.write().push_back(message);
    }
}

#[async_trait]
impl Channel for HttpChannel {
    fn id(&self) -> &str {
        &self.channel_id
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Http
    }

    async fn send(&self, message: &OutboundMessage) -> Layer4Result<()> {
        if !*self.connected.read() {
            return Err(anyhow::anyhow!("Channel not connected"));
        }

        // 实际 HTTP 发送需要 HTTP 客户端
        // 这里是占位实现
        tracing::debug!("HTTP channel sending: {}", message.content);

        Ok(())
    }

    async fn try_receive(&self) -> Layer4Result<Option<InboundMessage>> {
        if !*self.connected.read() {
            return Err(anyhow::anyhow!("Channel not connected"));
        }

        Ok(self.request_queue.write().pop_front())
    }

    fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    async fn close(&self) -> Layer4Result<()> {
        *self.connected.write() = false;
        self.request_queue.write().clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_channel_creation() {
        let channel = HttpChannel::default_channel();
        assert_eq!(channel.id(), "http-default");
        assert!(channel.is_connected());
    }

    #[test]
    fn test_http_channel_handle_request() {
        let channel = HttpChannel::default_channel();
        channel.handle_request("user-1", "POST /api/agent");

        let count = channel.request_queue.read().len();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_http_channel_close() {
        let channel = HttpChannel::default_channel();
        channel.close().await.unwrap();

        assert!(!channel.is_connected());
    }
}
