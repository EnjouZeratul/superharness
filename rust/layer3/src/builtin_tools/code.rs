//! # Code Analysis Tools
//!
//! 代码分析工具集：LSP 相关工具。

use crate::builtin_tools::BuiltinTool;
use crate::types::{CodeLocation, Layer3Result, ToolCategory};
use async_trait::async_trait;

/// Go to Definition Tool
pub struct GoToDefinitionTool;

#[async_trait]
impl BuiltinTool for GoToDefinitionTool {
    fn name(&self) -> &str {
        "go_to_definition"
    }

    fn description(&self) -> &str {
        "Find the definition of a symbol at a given location."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "The file path"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (1-based)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (1-based)"
                }
            },
            "required": ["file", "line", "column"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::CodeAnalysis
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        // Stub - will use LSP client in production
        Ok("Definition location placeholder".to_string())
    }
}

/// Find References Tool
pub struct FindReferencesTool;

#[async_trait]
impl BuiltinTool for FindReferencesTool {
    fn name(&self) -> &str {
        "find_references"
    }

    fn description(&self) -> &str {
        "Find all references to a symbol at a given location."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "The file path"
                },
                "line": {
                    "type": "integer",
                    "description": "Line number (1-based)"
                },
                "column": {
                    "type": "integer",
                    "description": "Column number (1-based)"
                }
            },
            "required": ["file", "line", "column"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::CodeAnalysis
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        Ok("References list placeholder".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto_definition_category() {
        let tool = GoToDefinitionTool;
        assert_eq!(tool.category(), ToolCategory::CodeAnalysis);
    }
}
