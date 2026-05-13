//! # Search Tools
//!
//! 搜索工具集：grep、glob、文件搜索等。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;

/// Grep Tool - Search content in files
pub struct GrepTool;

#[async_trait]
impl BuiltinTool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for a pattern in files using regex."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "The file or directory to search in"
                },
                "glob": {
                    "type": "string",
                    "description": "Optional: glob pattern to filter files"
                }
            },
            "required": ["pattern"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Search
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        // Stub implementation - will use ripgrep in production
        Ok("Grep results placeholder".to_string())
    }
}

/// Glob Tool - Find files by pattern
pub struct GlobTool;

#[async_trait]
impl BuiltinTool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The glob pattern (e.g., '**/*.rs')"
                },
                "path": {
                    "type": "string",
                    "description": "Optional: the directory to search in"
                }
            },
            "required": ["pattern"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Search
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing pattern parameter"))?;
        // Stub implementation
        Ok(format!("Files matching '{}': ...", pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_tool_category() {
        let tool = GrepTool;
        assert_eq!(tool.category(), ToolCategory::Search);
    }
}
