//! MCP 协议定义
//!
//! Model Context Protocol 消息类型和常量。

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MCP 协议版本
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpMessage {
    /// 请求消息
    Request(McpRequest),
    /// 响应消息
    Response(McpResponse),
    /// 通知消息
    Notification(McpNotification),
    /// 错误消息
    Error(McpError),
}

/// MCP 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    /// 请求 ID
    pub id: RequestId,
    /// 方法名
    pub method: String,
    /// 参数
    #[serde(default)]
    pub params: Option<Value>,
}

/// MCP 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    /// 对应的请求 ID
    pub id: RequestId,
    /// 结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// 错误
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpErrorData>,
}

/// MCP 通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotification {
    /// 方法名
    pub method: String,
    /// 参数
    #[serde(default)]
    pub params: Option<Value>,
}

/// MCP 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    /// 对应的请求 ID (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    /// 错误数据
    pub error: McpErrorData,
}

/// MCP 错误数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpErrorData {
    /// 错误码
    pub code: i32,
    /// 错误消息
    pub message: String,
    /// 额外数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// 请求 ID 类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

impl Default for RequestId {
    fn default() -> Self {
        RequestId::Number(0)
    }
}

// ========== MCP 方法常量 ==========

/// 初始化
pub const METHOD_INITIALIZE: &str = "initialize";
/// 初始化完成通知
pub const METHOD_INITIALIZED: &str = "notifications/initialized";
/// 关闭
pub const METHOD_SHUTDOWN: &str = "shutdown";
/// 列出工具
pub const METHOD_LIST_TOOLS: &str = "tools/list";
/// 调用工具
pub const METHOD_CALL_TOOL: &str = "tools/call";
/// 列出资源
pub const METHOD_LIST_RESOURCES: &str = "resources/list";
/// 读取资源
pub const METHOD_READ_RESOURCE: &str = "resources/read";
/// 列出提示词
pub const METHOD_LIST_PROMPTS: &str = "prompts/list";
/// 获取提示词
pub const METHOD_GET_PROMPT: &str = "prompts/get";

// ========== MCP 标准类型 ==========

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    #[serde(default)]
    pub description: Option<String>,
    /// 输入参数 Schema
    #[serde(default)]
    pub input_schema: Option<Value>,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 是否错误
    #[serde(default)]
    pub is_error: bool,
    /// 内容列表
    pub content: Vec<ContentBlock>,
}

/// 内容块
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// 文本内容
    Text { text: String },
    /// 图片内容
    Image { data: String, mime_type: String },
    /// 资源内容
    Resource { resource: ResourceContent },
}

/// 资源内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// URI
    pub uri: String,
    /// MIME 类型
    #[serde(default)]
    pub mime_type: Option<String>,
    /// 文本内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// 二进制内容 (base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// 初始化参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// 协议版本
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,
    /// 客户端能力
    pub capabilities: ClientCapabilities,
    /// 客户端信息
    #[serde(default)]
    pub client_info: Option<Implementation>,
}

fn default_protocol_version() -> String {
    MCP_VERSION.to_string()
}

/// 客户端能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// 实验性功能
    #[serde(default)]
    pub experimental: Option<Value>,
    /// 根目录支持
    #[serde(default)]
    pub roots: Option<RootsCapability>,
    /// 采样支持
    #[serde(default)]
    pub sampling: Option<Value>,
}

/// 根目录能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootsCapability {
    /// 是否支持列表变更通知
    #[serde(default)]
    pub list_changed: Option<bool>,
}

/// 实现信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    /// 名称
    pub name: String,
    /// 版本
    pub version: String,
}

/// 初始化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// 协议版本
    pub protocol_version: String,
    /// 服务端能力
    pub capabilities: ServerCapabilities,
    /// 服务端信息
    pub server_info: Implementation,
    /// 指令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// 服务端能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// 实验性功能
    #[serde(default)]
    pub experimental: Option<Value>,
    /// 日志支持
    #[serde(default)]
    pub logging: Option<Value>,
    /// 提示词支持
    #[serde(default)]
    pub prompts: Option<PromptsCapability>,
    /// 资源支持
    #[serde(default)]
    pub resources: Option<ResourcesCapability>,
    /// 工具支持
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
}

/// 提示词能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// 是否支持列表变更通知
    #[serde(default)]
    pub list_changed: Option<bool>,
}

/// 资源能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// 是否支持订阅
    #[serde(default)]
    pub subscribe: Option<bool>,
    /// 是否支持列表变更通知
    #[serde(default)]
    pub list_changed: Option<bool>,
}

/// 工具能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// 是否支持列表变更通知
    #[serde(default)]
    pub list_changed: Option<bool>,
}

// ========== 错误码 ==========

/// 标准错误码
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // MCP 特定错误码
    pub const SERVER_NOT_INITIALIZED: i32 = -32002;
    pub const UNKNOWN_ERROR: i32 = -32001;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_request() {
        let request = McpRequest {
            id: RequestId::Number(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("tools/list"));
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{"id":1,"result":{"tools":[]}}"#;
        let response: McpResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, RequestId::Number(1));
    }

    #[test]
    fn test_tool_definition() {
        let tool = ToolDefinition {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: None,
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("test_tool"));
    }

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::Text {
            text: "Hello".to_string(),
        };

        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Hello"));
    }
}
