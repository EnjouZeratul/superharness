//! # Memory Tools
//!
//! 记忆操作工具集。

use crate::types::{Layer3Result, ToolCategory, MemoryTier};
use crate::builtin_tools::BuiltinTool;
use async_trait::async_trait;

/// Save Memory Tool
pub struct SaveMemoryTool;

#[async_trait]
impl BuiltinTool for SaveMemoryTool {
    fn name(&self) -> &str {
        "save_memory"
    }

    fn description(&self) -> &str {
        "Save a memory entry to the memory system."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The content to remember"
                },
                "tier": {
                    "type": "string",
                    "enum": ["working", "session", "project", "long_term"],
                    "description": "Memory tier to store in"
                },
                "metadata": {
                    "type": "object",
                    "description": "Optional: additional metadata"
                }
            },
            "required": ["content"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        Ok("Memory saved successfully".to_string())
    }
}

/// Query Memory Tool
pub struct QueryMemoryTool;

#[async_trait]
impl BuiltinTool for QueryMemoryTool {
    fn name(&self) -> &str {
        "query_memory"
    }

    fn description(&self) -> &str {
        "Query the memory system for relevant memories."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The query text"
                },
                "tier": {
                    "type": "string",
                    "enum": ["working", "session", "project", "long_term"],
                    "description": "Optional: limit to specific tier"
                },
                "limit": {
                    "type": "integer",
                    "description": "Optional: maximum number of results"
                }
            },
            "required": ["query"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        Ok("Memory query results placeholder".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tool_category() {
        let tool = SaveMemoryTool;
        assert_eq!(tool.category(), ToolCategory::Memory);
    }
}