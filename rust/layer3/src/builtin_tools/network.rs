//! # Network Tools
//!
//! 网络请求工具集。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;

/// HTTP Request Tool
pub struct HttpRequestTool;

#[async_trait]
impl BuiltinTool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Make an HTTP request to a URL."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to request"
                },
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE"],
                    "description": "HTTP method"
                },
                "headers": {
                    "type": "object",
                    "description": "Optional: request headers"
                },
                "body": {
                    "type": "string",
                    "description": "Optional: request body"
                }
            },
            "required": ["url"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Network
    }

    async fn execute(&self, _args: serde_json::Value) -> Layer3Result<String> {
        // Stub implementation
        Ok("HTTP response placeholder".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_tool_category() {
        let tool = HttpRequestTool;
        assert_eq!(tool.category(), ToolCategory::Network);
    }
}
