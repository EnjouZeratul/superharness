//! MCP Bridge 模块
//!
//! Model Context Protocol 实现。
//!
//! ## 功能
//! - MCP 协议消息处理
//! - 工具注册与发现
//! - 多种传输层支持 (stdio, tcp, unix)
//! - 流式响应
//!
//! ## 用法
//! ```rust,ignore
//! use sh_layer4::mcp_bridge::{McpBridge, McpBridgeConfig};
//!
//! let bridge = McpBridge::new(McpBridgeConfig::default());
//! bridge.register_simple_tool("echo", "Echo tool", |name, args| {
//!     Ok(ToolResult {
//!         is_error: false,
//!         content: vec![ContentBlock::Text { text: args.to_string() }],
//!     })
//! });
//!
//! bridge.start().await?;
//! bridge.initialize("my-client", "1.0.0").await?;
//! ```

pub mod bridge;
pub mod client;
pub mod handler;
pub mod protocol;
pub mod transport;

// 主要导出
pub use bridge::{McpBridge, McpBridgeConfig};
pub use client::{McpClientManager, McpServerConfig};
pub use handler::{DefaultHandler, McpHandler, SimpleToolExecutor, ToolExecutor};
pub use protocol::{
    error_codes, ClientCapabilities, ContentBlock, Implementation, InitializeParams,
    InitializeResult, McpError, McpErrorData, McpMessage, McpNotification, McpRequest, McpResponse,
    RequestId, ResourceContent, ServerCapabilities, ToolDefinition, ToolResult, MCP_VERSION,
};
pub use transport::{
    McpTransport, McpTransportType, MemoryTransport, StdioTransport, TcpTransport,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // 测试主要类型可以导出
        let _config = McpBridgeConfig::default();
    }
}
