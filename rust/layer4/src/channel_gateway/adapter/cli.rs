//! # CLI Channel Adapter
//!
//! 终端命令行渠道适配器。

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::VecDeque;

use crate::channel_gateway::{Channel, ChannelType, InboundMessage, OutboundMessage};
use crate::types::Layer4Result;

/// CLI 渠道适配器
pub struct CliChannel {
    channel_id: String,
    connected: RwLock<bool>,
    input_queue: RwLock<VecDeque<InboundMessage>>,
    output_callback: RwLock<Option<Box<dyn Fn(&str) + Send + Sync>>>,
}

impl CliChannel {
    /// 创建新的 CLI 渠道
    pub fn new(channel_id: impl Into<String>) -> Self {
        Self {
            channel_id: channel_id.into(),
            connected: RwLock::new(true),
            input_queue: RwLock::new(VecDeque::new()),
            output_callback: RwLock::new(None),
        }
    }

    /// 创建默认 CLI 渠道
    pub fn default_channel() -> Self {
        Self::new("cli-default")
    }

    /// 设置输出回调
    pub fn set_output_callback<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        *self.output_callback.write() = Some(Box::new(callback));
    }

    /// 推送输入消息（用于测试或外部输入）
    pub fn push_input(&self, user_id: &str, content: &str) {
        let message = InboundMessage::new(&self.channel_id, user_id, content);
        self.input_queue.write().push_back(message);
    }

    /// 推送带会话 ID 的输入消息
    pub fn push_input_with_session(&self, user_id: &str, content: &str, session_id: &str) {
        let message =
            InboundMessage::new(&self.channel_id, user_id, content).with_session(session_id);
        self.input_queue.write().push_back(message);
    }

    /// 获取待处理消息数量
    pub fn pending_count(&self) -> usize {
        self.input_queue.read().len()
    }
}

#[async_trait]
impl Channel for CliChannel {
    fn id(&self) -> &str {
        &self.channel_id
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Cli
    }

    async fn send(&self, message: &OutboundMessage) -> Layer4Result<()> {
        if !*self.connected.read() {
            return Err(anyhow::anyhow!("Channel not connected"));
        }

        // 调用输出回调（如果设置）
        if let Some(callback) = self.output_callback.read().as_ref() {
            callback(&message.content);
        } else {
            // 默认输出到 stdout
            println!("{}", message.content);
        }

        Ok(())
    }

    async fn try_receive(&self) -> Layer4Result<Option<InboundMessage>> {
        if !*self.connected.read() {
            return Err(anyhow::anyhow!("Channel not connected"));
        }

        Ok(self.input_queue.write().pop_front())
    }

    fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    async fn close(&self) -> Layer4Result<()> {
        *self.connected.write() = false;
        self.input_queue.write().clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_channel_creation() {
        let channel = CliChannel::new("test-cli");
        assert_eq!(channel.id(), "test-cli");
        assert!(channel.is_connected());
    }

    #[test]
    fn test_cli_channel_push_input() {
        let channel = CliChannel::new("test-cli");
        channel.push_input("user-1", "Hello");

        assert_eq!(channel.pending_count(), 1);
    }

    #[tokio::test]
    async fn test_cli_channel_receive() {
        let channel = CliChannel::new("test-cli");
        channel.push_input("user-1", "Hello");

        let msg = channel.try_receive().await.unwrap();
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.user_id, "user-1");
    }

    #[tokio::test]
    async fn test_cli_channel_send() {
        use crate::channel_gateway::MessageTarget;

        let channel = CliChannel::new("test-cli");
        let output = RwLock::new(String::new());

        channel.set_output_callback(|content| {
            // 在实际测试中这里会更新 output
        });

        let message = OutboundMessage::new("Test message", MessageTarget::All);
        let result = channel.send(&message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cli_channel_close() {
        let channel = CliChannel::new("test-cli");
        channel.close().await.unwrap();

        assert!(!channel.is_connected());
    }
}
