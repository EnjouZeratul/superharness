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

pub mod types;
pub mod channel_gateway;
pub mod plugin_loader;
pub mod worktree_manager;
pub mod mcp_bridge;
pub mod audit_logger;

// 导出核心类型
pub use types::{
    IntegrationConfig, Layer4Error, Layer4Result, MessagePriority, PluginPermission,
};

// 导出渠道网关
pub use channel_gateway::{
    Channel, ChannelGateway, ChannelType, InboundMessage, MessageTarget, MessageType,
    OutboundMessage, MessageRouter,
};

// 导出渠道适配器
pub use channel_gateway::adapter::{CliChannel, HttpChannel, WebSocketChannel};

// 导出插件加载器
pub use plugin_loader::{
    Plugin, PluginContext, PluginInfo, PluginLoader, PluginMeta, PluginRegistry, PluginState,
};

// 导出 Worktree 管理器
pub use worktree_manager::{
    Worktree, WorktreeConfig, WorktreeManager, WorktreeStatus,
};

// 导出 MCP 桥接 (Terminal 1)
pub use mcp_bridge::{
    McpBridge, McpBridgeConfig,
    McpHandler, DefaultHandler, ToolExecutor, SimpleToolExecutor,
    McpTransport, McpTransportType, MemoryTransport, TcpTransport, StdioTransport,
    ContentBlock, ToolDefinition, ToolResult, McpError, McpMessage, MCP_VERSION,
};

// 导出审计日志 (Terminal 1)
pub use audit_logger::{
    AuditLogger, AuditConfig,
    AuditEntry, AuditAction, AuditResult, AuditFilter, ExportFormat,
    AuditStorage, MemoryStorage, FileStorage,
};

// 导出 traits
pub mod traits {
    pub use super::channel_gateway::Channel;
    pub use super::plugin_loader::Plugin;
}