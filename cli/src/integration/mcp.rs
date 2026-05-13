//! # MCP CLI 集成
//!
//! 封装 mcp_bridge 为 CLI 易用接口。

use anyhow::Result;
use sh_core::layer4::{ContentBlock, ToolDefinition, ToolResult};
use sh_core::layer4::{McpBridge, McpBridgeConfig};
use std::sync::Arc;

/// MCP CLI 服务
///
/// 提供简化的 MCP 协议操作接口。
pub struct McpService {
    bridge: Arc<McpBridge>,
}

impl McpService {
    /// 创建新的 MCP 服务
    pub fn new() -> Self {
        let config = McpBridgeConfig::default();
        Self {
            bridge: Arc::new(McpBridge::new(config)),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(config: McpBridgeConfig) -> Self {
        Self {
            bridge: Arc::new(McpBridge::new(config)),
        }
    }

    /// 注册自定义工具
    pub fn register_tool(
        &self,
        name: &str,
        description: &str,
        handler: impl ToolHandler + 'static,
    ) {
        let tool = ToolDefinition {
            name: name.to_string(),
            description: Some(description.to_string()),
            input_schema: None,
        };

        let executor = Arc::new(CliToolExecutor::new(handler));
        self.bridge.register_tool(tool, executor);
    }

    /// 启动 MCP 服务
    pub async fn start(&self) -> Result<()> {
        self.bridge.start().await?;
        Ok(())
    }

    /// 停止 MCP 服务
    pub async fn stop(&self) -> Result<()> {
        self.bridge.stop().await?;
        Ok(())
    }

    /// 初始化连接
    pub async fn initialize(&self, client_name: &str, version: &str) -> Result<()> {
        self.bridge.initialize(client_name, version).await?;
        Ok(())
    }

    /// 列出可用工具
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        let tools = self.bridge.list_tools().await?;
        Ok(tools
            .into_iter()
            .map(|t| ToolInfo {
                name: t.name,
                description: t.description.unwrap_or_default(),
            })
            .collect())
    }

    /// 调用工具
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        self.bridge.call_tool(name, arguments).await
    }

    /// 获取服务信息
    pub fn info(&self) -> ServiceInfo {
        ServiceInfo {
            server_name: "SuperHarness MCP".to_string(),
            version: "0.1.0".to_string(),
            is_running: self.bridge.is_running(),
        }
    }
}

impl Default for McpService {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具信息
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

/// 服务信息
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub server_name: String,
    pub version: String,
    pub is_running: bool,
}

/// 工具处理器 trait
pub trait ToolHandler: Send + Sync {
    /// 处理工具调用
    fn handle(&self, name: &str, args: serde_json::Value) -> Result<ToolResult>;
}

/// CLI 工具执行器
struct CliToolExecutor {
    handler: Box<dyn ToolHandler>,
}

impl CliToolExecutor {
    fn new(handler: impl ToolHandler + 'static) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }
}

#[async_trait::async_trait]
impl sh_core::layer4::ToolExecutor for CliToolExecutor {
    async fn execute(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        self.handler.handle(name, arguments)
    }
}

// ============================================================================
// 示例工具处理器
// ============================================================================

/// 示例：Echo 工具
pub struct EchoTool;

impl ToolHandler for EchoTool {
    fn handle(&self, _name: &str, args: serde_json::Value) -> Result<ToolResult> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message provided");

        Ok(ToolResult {
            is_error: false,
            content: vec![ContentBlock::Text {
                text: message.to_string(),
            }],
        })
    }
}

/// 示例：计算工具
pub struct CalcTool;

impl ToolHandler for CalcTool {
    fn handle(&self, _name: &str, args: serde_json::Value) -> Result<ToolResult> {
        let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let op = args.get("op").and_then(|v| v.as_str()).unwrap_or("add");

        let result = match op {
            "add" => a + b,
            "sub" => a - b,
            "mul" => a * b,
            "div" => {
                if b == 0.0 {
                    return Ok(ToolResult {
                        is_error: true,
                        content: vec![ContentBlock::Text {
                            text: "Division by zero".to_string(),
                        }],
                    });
                }
                a / b
            }
            _ => {
                return Ok(ToolResult {
                    is_error: true,
                    content: vec![ContentBlock::Text {
                        text: format!("Unknown operation: {}", op),
                    }],
                });
            }
        };

        Ok(ToolResult {
            is_error: false,
            content: vec![ContentBlock::Text {
                text: result.to_string(),
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_service_creation() {
        let service = McpService::new();
        let info = service.info();
        assert!(!info.is_running);
    }

    #[test]
    fn test_echo_tool() {
        let tool = EchoTool;
        let args = serde_json::json!({"message": "Hello"});
        let result = tool.handle("echo", args).unwrap();
        assert!(!result.is_error);
    }

    #[test]
    fn test_calc_tool_add() {
        let tool = CalcTool;
        let args = serde_json::json!({"a": 5, "b": 3, "op": "add"});
        let result = tool.handle("calc", args).unwrap();
        assert!(!result.is_error);
    }
}
