//! # Shell Tools
//!
//! Shell 执行工具集。

use crate::types::{Layer3Result, ToolCategory};
use crate::builtin_tools::BuiltinTool;
use async_trait::async_trait;

/// Bash Tool - Execute shell commands
pub struct BashTool;

#[async_trait]
impl BuiltinTool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash shell command with timeout."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Optional: timeout in milliseconds"
                }
            },
            "required": ["command"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Shell
    }

    fn is_dangerous(&self) -> bool {
        true
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let command = args["command"].as_str().ok_or_else(|| anyhow::anyhow!("Missing command parameter"))?;
        // Stub - will use tokio::process::Command in production
        Ok(format!("Executed: {}", command))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_tool_dangerous() {
        let tool = BashTool;
        assert!(tool.is_dangerous());
        assert!(tool.requires_confirmation());
    }
}