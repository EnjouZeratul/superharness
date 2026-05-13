//! # SuperHarness Layer 4: Integration
//!
//! 外部系统和协议集成。
//!
//! ## 模块结构
//! - `types`: 核心类型定义
//! - `channel_gateway`: 多渠道网关
//! - `plugin_loader`: 插件加载器
//! - `worktree_manager`: Git Worktree 管理
//! - `mcp_bridge`: MCP 协议桥接 (Terminal 1)
//! - `audit_logger`: 审计日志 (Terminal 1)

pub mod audit_logger;
pub mod channel_gateway;
pub mod mcp_bridge;
pub mod plugin_loader;
pub mod types;
pub mod worktree_manager;

// 导出核心类型
pub use types::{IntegrationConfig, Layer4Error, Layer4Result, MessagePriority, PluginPermission};

// 导出渠道网关
pub use channel_gateway::{
    Channel, ChannelGateway, ChannelType, InboundMessage, MessageRouter, MessageTarget,
    MessageType, OutboundMessage,
};

// 导出渠道适配器
pub use channel_gateway::adapter::{CliChannel, HttpChannel, WebSocketChannel};

// 导出插件加载器
pub use plugin_loader::{
    Plugin, PluginContext, PluginInfo, PluginLoader, PluginMeta, PluginRegistry, PluginState,
};

// 导出 Worktree 管理器
pub use worktree_manager::{Worktree, WorktreeConfig, WorktreeManager, WorktreeStatus};

// 导出 MCP 桥接 (Terminal 1)
pub use mcp_bridge::{
    ContentBlock, DefaultHandler, McpBridge, McpBridgeConfig, McpError, McpHandler, McpMessage,
    McpTransport, McpTransportType, MemoryTransport, SimpleToolExecutor, StdioTransport,
    TcpTransport, ToolDefinition, ToolExecutor, ToolResult, MCP_VERSION,
};

// 导出审计日志 (Terminal 1)
pub use audit_logger::{
    AuditAction, AuditConfig, AuditEntry, AuditFilter, AuditLogger, AuditResult, AuditStorage,
    ExportFormat, FileStorage, MemoryStorage,
};

// 导出 traits
pub mod traits {
    pub use super::channel_gateway::Channel;
    pub use super::plugin_loader::Plugin;
}
