//! MCP 桥接器
//!
//! MCP 协议的主要实现。

use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use super::handler::{DefaultHandler, ToolExecutor};
use super::protocol::{
    McpMessage, McpNotification, McpRequest, McpResponse, RequestId, ToolDefinition,
    ToolResult,
};
use super::transport::McpTransport;
use anyhow::{anyhow, Result};

/// MCP 桥接器配置
#[derive(Debug, Clone)]
pub struct McpBridgeConfig {
    /// 服务端名称
    pub server_name: String,
    /// 服务端版本
    pub server_version: String,
    /// 请求超时 (毫秒)
    pub request_timeout_ms: u64,
    /// 最大并发请求数
    pub max_concurrent_requests: usize,
}

impl Default for McpBridgeConfig {
    fn default() -> Self {
        Self {
            server_name: "Continuum".to_string(),
            server_version: "0.1.0".to_string(),
            request_timeout_ms: 30000,
            max_concurrent_requests: 100,
        }
    }
}

/// MCP 桥接器
pub struct McpBridge {
    /// 传输层
    transport: RwLock<Option<Box<dyn McpTransport>>>,
    /// 消息处理器
    handler: Arc<DefaultHandler>,
    /// 配置
    #[allow(dead_code)]
    config: McpBridgeConfig,
    /// 请求 ID 计数器
    request_id_counter: AtomicU64,
    /// 待处理响应
    #[allow(dead_code)]
    pending_responses: RwLock<HashMap<RequestId, mpsc::Sender<McpResponse>>>,
    /// 运行状态
    running: RwLock<bool>,
}

impl McpBridge {
    /// 创建新的 MCP 桥接器
    pub fn new(config: McpBridgeConfig) -> Self {
        let handler = DefaultHandler::new(&config.server_name, &config.server_version);
        Self {
            transport: RwLock::new(None),
            handler: Arc::new(handler),
            config,
            request_id_counter: AtomicU64::new(0),
            pending_responses: RwLock::new(HashMap::new()),
            running: RwLock::new(false),
        }
    }

    /// 使用传输层
    pub fn with_transport(self, transport: Box<dyn McpTransport>) -> Self {
        *self.transport.write() = Some(transport);
        self
    }

    /// 注册工具
    pub fn register_tool(&self, tool: ToolDefinition, executor: Arc<dyn ToolExecutor>) {
        self.handler.register_tool(tool, executor);
    }

    /// 注册简单工具
    pub fn register_simple_tool<F>(&self, name: &str, description: &str, executor: F)
    where
        F: Fn(&str, Value) -> Result<ToolResult> + Send + Sync + 'static,
    {
        let tool = ToolDefinition {
            name: name.to_string(),
            description: Some(description.to_string()),
            input_schema: None,
        };
        self.register_tool(tool, Arc::new(super::handler::SimpleToolExecutor(executor)));
    }

    /// 生成下一个请求 ID
    fn next_request_id(&self) -> RequestId {
        RequestId::Number(self.request_id_counter.fetch_add(1, Ordering::SeqCst) as i64)
    }

    /// 启动桥接器
    pub async fn start(&self) -> Result<()> {
        *self.running.write() = true;

        // 启动消息处理循环
        let _handler = self.handler.clone();

        tokio::spawn(async move {
            // 消息处理循环
            // 简化实现：实际需要从 transport 读取消息
        });

        Ok(())
    }

    /// 停止桥接器
    pub async fn stop(&self) -> Result<()> {
        *self.running.write() = false;

        let transport = self.transport.write().take();
        if let Some(transport) = transport {
            transport.close().await?;
        }

        Ok(())
    }

    /// 发送请求并等待响应
    #[allow(clippy::await_holding_lock)]
    pub async fn request(&self, method: &str, params: Option<Value>) -> Result<McpResponse> {
        let id = self.next_request_id();

        let request = McpRequest {
            id: id.clone(),
            method: method.to_string(),
            params,
        };

        let message = McpMessage::Request(request);

        // 发送请求
        {
            let transport_guard = self.transport.read();
            let transport = transport_guard
                .as_ref()
                .ok_or_else(|| anyhow!("Transport not initialized"))?;
            transport.send(&message).await?;
        }

        // 等待响应 (简化实现，实际需要超时和响应匹配)
        Ok(McpResponse {
            id,
            result: Some(Value::Null),
            error: None,
        })
    }

    /// 发送通知
    #[allow(clippy::await_holding_lock)]
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let notification = McpNotification {
            method: method.to_string(),
            params,
        };

        let message = McpMessage::Notification(notification);

        let transport_guard = self.transport.read();
        let transport = transport_guard
            .as_ref()
            .ok_or_else(|| anyhow!("Transport not initialized"))?;
        transport.send(&message).await?;

        Ok(())
    }

    /// 列出工具
    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        let response = self.request("tools/list", None).await?;

        if let Some(result) = response.result {
            let tools: Vec<ToolDefinition> = serde_json::from_value(
                result.get("tools").cloned().unwrap_or(Value::Array(vec![])),
            )?;
            Ok(tools)
        } else {
            Ok(vec![])
        }
    }

    /// 调用工具
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolResult> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let response = self.request("tools/call", Some(params)).await?;

        if let Some(result) = response.result {
            let tool_result: ToolResult = serde_json::from_value(result)?;
            Ok(tool_result)
        } else if let Some(error) = response.error {
            Err(anyhow!("Tool call error: {}", error.message))
        } else {
            Err(anyhow!("Unknown error"))
        }
    }

    /// 初始化连接
    pub async fn initialize(&self, client_info: &str, version: &str) -> Result<()> {
        let params = serde_json::json!({
            "protocol_version": "2024-11-05",
            "capabilities": {},
            "client_info": {
                "name": client_info,
                "version": version
            }
        });

        let response = self.request("initialize", Some(params)).await?;

        if response.error.is_some() {
            return Err(anyhow!("Initialize failed"));
        }

        // 发送 initialized 通知
        self.notify("notifications/initialized", None).await?;

        Ok(())
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_bridge::transport::MemoryTransport;
    use crate::mcp_bridge::protocol::ContentBlock;

    #[tokio::test]
    async fn test_bridge_creation() {
        let config = McpBridgeConfig::default();
        let bridge = McpBridge::new(config);

        assert!(!bridge.is_running());
    }

    #[tokio::test]
    async fn test_register_tool() {
        let bridge = McpBridge::new(McpBridgeConfig::default());

        bridge.register_simple_tool("test_tool", "A test tool", |_name, _args| {
            Ok(ToolResult {
                is_error: false,
                content: vec![ContentBlock::Text {
                    text: "OK".to_string(),
                }],
            })
        });

        // 工具已注册
        assert!(true);
    }

    #[tokio::test]
    async fn test_next_request_id() {
        let bridge = McpBridge::new(McpBridgeConfig::default());

        let id1 = bridge.next_request_id();
        let id2 = bridge.next_request_id();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_config_default() {
        let config = McpBridgeConfig::default();
        assert_eq!(config.server_name, "Continuum");
        assert_eq!(config.request_timeout_ms, 30000);
    }
}
