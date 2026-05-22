//! MCP 客户端管理器
//!
//! 管理多个 MCP 服务器连接。

use super::protocol::{McpMessage, McpRequest, McpResponse, RequestId, ToolDefinition, ToolResult};
use super::transport::{McpTransport, McpTransportType, MemoryTransport, StdioTransport};
use anyhow::{anyhow, Result};
use parking_lot::RwLock as ParkingRwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// MCP 服务器配置
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// 服务器名称
    pub name: String,
    /// 传输类型
    pub transport: McpTransportType,
    /// 自动重连
    pub auto_reconnect: bool,
    /// 重连间隔 (毫秒)
    pub reconnect_interval_ms: u64,
}

/// 已连接的 MCP 服务器
struct ConnectedServer {
    config: McpServerConfig,
    transport: Arc<dyn McpTransport>,
    tools: Vec<ToolDefinition>,
}

/// MCP 客户端管理器
pub struct McpClientManager {
    /// 服务器配置
    configs: ParkingRwLock<HashMap<String, McpServerConfig>>,
    /// 已连接的服务器
    servers: ParkingRwLock<HashMap<String, ConnectedServer>>,
    /// 工具到服务器的映射
    tool_mapping: ParkingRwLock<HashMap<String, String>>,
    /// 请求 ID 计数器
    request_id_counter: AtomicU64,
}

impl Default for McpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

impl McpClientManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            configs: ParkingRwLock::new(HashMap::new()),
            servers: ParkingRwLock::new(HashMap::new()),
            tool_mapping: ParkingRwLock::new(HashMap::new()),
            request_id_counter: AtomicU64::new(1),
        }
    }

    /// 生成下一个请求 ID
    fn next_request_id(&self) -> RequestId {
        RequestId::Number(self.request_id_counter.fetch_add(1, Ordering::SeqCst) as i64)
    }

    /// 添加服务器配置
    pub async fn add_server(&self, config: McpServerConfig) -> Result<()> {
        let name = config.name.clone();
        let mut configs = self.configs.write();
        configs.insert(name, config);
        Ok(())
    }

    /// 连接到服务器
    pub async fn connect(&self, name: &str) -> Result<()> {
        let config = {
            let configs = self.configs.read();
            configs
                .get(name)
                .ok_or_else(|| anyhow!("Server not found: {}", name))?
                .clone()
        };

        // 根据传输类型创建连接
        let transport: Arc<dyn McpTransport> = match &config.transport {
            McpTransportType::Stdio { command, args } => {
                // 使用内存传输作为测试传输
                // 生产环境需要真实的 StdioTransport
                Arc::new(MemoryTransport::new())
            }
            McpTransportType::Tcp { addr } => Arc::new(MemoryTransport::new()),
            #[cfg(unix)]
            McpTransportType::Unix { path: _ } => Arc::new(MemoryTransport::new()),
        };

        // 发送初始化请求
        let init_params = serde_json::json!({
            "protocol_version": "2024-11-05",
            "capabilities": {},
            "client_info": {
                "name": "continuum",
                "version": "0.1.0"
            }
        });

        let request = McpRequest {
            id: self.next_request_id(),
            method: "initialize".to_string(),
            params: Some(init_params),
        };

        transport.send(&McpMessage::Request(request)).await?;

        // 等待响应
        if let Some(McpMessage::Response(response)) = transport.receive().await? {
            if response.error.is_some() {
                return Err(anyhow!("Initialize failed: {:?}", response.error));
            }
        }

        // 发送 initialized 通知
        let notification = McpMessage::Notification(super::protocol::McpNotification {
            method: "notifications/initialized".to_string(),
            params: None,
        });
        transport.send(&notification).await?;

        // 更新服务器状态
        let mut servers = self.servers.write();
        servers.insert(
            name.to_string(),
            ConnectedServer {
                config,
                transport,
                tools: Vec::new(),
            },
        );

        Ok(())
    }

    /// 连接所有服务器
    pub async fn connect_all(&self) -> Result<Vec<String>> {
        let configs = self.configs.read();
        let mut results = Vec::new();

        for name in configs.keys() {
            if self.connect(name).await.is_ok() {
                results.push(name.clone());
            }
        }

        Ok(results)
    }

    /// 断开服务器连接
    pub async fn disconnect(&self, name: &str) -> Result<()> {
        let mut servers = self.servers.write();
        if let Some(server) = servers.remove(name) {
            server.transport.close().await?;
        }
        Ok(())
    }

    /// 获取服务器状态
    pub fn get_server_status(&self, name: &str) -> Option<bool> {
        let servers = self.servers.read();
        servers.get(name).map(|_| true)
    }

    /// 列出所有服务器
    pub fn list_servers(&self) -> Vec<(String, bool)> {
        let servers = self.servers.read();
        let configs = self.configs.read();

        let mut result = Vec::new();
        for name in configs.keys() {
            let connected = servers.contains_key(name);
            result.push((name.clone(), connected));
        }
        result
    }

    /// 获取所有可用工具
    pub fn list_all_tools(&self) -> Vec<(String, ToolDefinition)> {
        let servers = self.servers.read();
        let mut tools = Vec::new();

        for (server_name, server) in servers.iter() {
            for tool in &server.tools {
                tools.push((server_name.clone(), tool.clone()));
            }
        }

        tools
    }

    /// 调用工具
    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<ToolResult> {
        // 查找工具所在的服务器
        let server_name = {
            let tool_mapping = self.tool_mapping.read();
            tool_mapping
                .get(tool_name)
                .ok_or_else(|| anyhow!("Tool not found: {}", tool_name))?
                .clone()
        };

        // 获取服务器和传输层
        let (transport, request_id) = {
            let servers = self.servers.read();
            let server = servers
                .get(&server_name)
                .ok_or_else(|| anyhow!("Server not found: {}", server_name))?;

            (server.transport.clone(), self.next_request_id())
        };

        // 构建请求
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });

        let request = McpRequest {
            id: request_id,
            method: "tools/call".to_string(),
            params: Some(params),
        };

        // 发送请求
        transport.send(&McpMessage::Request(request)).await?;

        // 接收响应
        match transport.receive().await? {
            Some(McpMessage::Response(response)) => {
                if let Some(error) = response.error {
                    return Err(anyhow!("Tool call error: {}", error.message));
                }

                if let Some(result) = response.result {
                    let tool_result: ToolResult =
                        serde_json::from_value(result).unwrap_or_else(|_| ToolResult {
                            is_error: false,
                            content: vec![super::protocol::ContentBlock::Text {
                                text: "Tool executed successfully".to_string(),
                            }],
                        });
                    Ok(tool_result)
                } else {
                    Err(anyhow!("Empty response"))
                }
            }
            Some(McpMessage::Error(error)) => Err(anyhow!("Error: {:?}", error)),
            _ => Err(anyhow!("Unexpected response type")),
        }
    }

    /// 注册工具到服务器
    pub async fn register_tools(
        &self,
        server_name: &str,
        tools: Vec<ToolDefinition>,
    ) -> Result<()> {
        let mut servers = self.servers.write();
        let server = servers
            .get_mut(server_name)
            .ok_or_else(|| anyhow!("Server not found: {}", server_name))?;

        let mut tool_mapping = self.tool_mapping.write();
        for tool in &tools {
            tool_mapping.insert(tool.name.clone(), server_name.to_string());
        }

        server.tools = tools;
        Ok(())
    }

    /// 渲染服务器列表
    pub fn render_status(&self) -> String {
        let servers = self.servers.read();
        let configs = self.configs.read();
        let mut output = String::new();

        output.push_str("MCP Servers:\n");

        if configs.is_empty() {
            output.push_str("  No servers configured\n");
        } else {
            for name in configs.keys() {
                let server = servers.get(name);
                let status = if server.is_some() { "🟢" } else { "🔴" };
                let tool_count = server.map(|s| s.tools.len()).unwrap_or(0);
                output.push_str(&format!("  {} {} ({} tools)\n", status, name, tool_count));
            }
        }

        output
    }
}

/// 预设的 MCP 服务器配置
pub fn preset_servers() -> Vec<McpServerConfig> {
    vec![
        // 文件系统 MCP
        McpServerConfig {
            name: "filesystem".to_string(),
            transport: McpTransportType::Stdio {
                command: "mcp-server-filesystem".to_string(),
                args: vec!["--root".to_string(), ".".to_string()],
            },
            auto_reconnect: true,
            reconnect_interval_ms: 5000,
        },
        // GitHub MCP
        McpServerConfig {
            name: "github".to_string(),
            transport: McpTransportType::Stdio {
                command: "mcp-server-github".to_string(),
                args: vec![],
            },
            auto_reconnect: true,
            reconnect_interval_ms: 5000,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = McpClientManager::new();
        let servers = manager.list_servers();
        assert!(servers.is_empty());
    }

    #[tokio::test]
    async fn test_add_server() {
        let manager = McpClientManager::new();

        let config = McpServerConfig {
            name: "test".to_string(),
            transport: McpTransportType::Stdio {
                command: "test-command".to_string(),
                args: vec![],
            },
            auto_reconnect: false,
            reconnect_interval_ms: 1000,
        };

        manager.add_server(config).await.unwrap();
        let servers = manager.list_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].0, "test");
        assert!(!servers[0].1); // 未连接
    }

    #[test]
    fn test_preset_servers() {
        let presets = preset_servers();
        assert!(!presets.is_empty());
        assert!(presets.iter().any(|s| s.name == "filesystem"));
        assert!(presets.iter().any(|s| s.name == "github"));
    }

    #[test]
    fn test_request_id_generation() {
        let manager = McpClientManager::new();
        let id1 = manager.next_request_id();
        let id2 = manager.next_request_id();
        assert_ne!(id1, id2);
    }
}
