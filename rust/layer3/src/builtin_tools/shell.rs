//! # Shell Tools
//!
//! Shell 执行工具集。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

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
                    "description": "Optional: timeout in milliseconds (default: 30000)"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Optional: working directory for the command"
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
        let command = args["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing command parameter"))?;

        let timeout_ms = args["timeout"].as_u64().unwrap_or(30000);
        let working_dir = args["working_dir"].as_str().map(|s| s.to_string());

        // Build the command
        #[cfg(windows)]
        let mut cmd = Command::new("cmd");
        #[cfg(windows)]
        cmd.args(["/C", command]);

        #[cfg(not(windows))]
        let mut cmd = Command::new("sh");
        #[cfg(not(windows))]
        cmd.args(["-c", command]);

        // Set working directory
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Execute with timeout
        let timeout_duration = Duration::from_millis(timeout_ms);

        let output = timeout(timeout_duration, cmd.output())
            .await
            .map_err(|_| anyhow::anyhow!("Command timed out after {}ms", timeout_ms))?
            .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;

        // Process output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.trim().to_string())
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            let mut error_msg = format!("Exit code: {}", exit_code);
            if !stderr.is_empty() {
                error_msg.push_str(&format!("\nError: {}", stderr.trim()));
            }
            if !stdout.is_empty() {
                error_msg.push_str(&format!("\nOutput: {}", stdout.trim()));
            }
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_bash_tool_dangerous() {
        let tool = BashTool;
        assert!(tool.is_dangerous());
        assert!(tool.requires_confirmation());
    }

    #[tokio::test]
    async fn test_bash_execute_success() {
        let tool = BashTool;

        #[cfg(windows)]
        let result = tool.execute(json!({"command": "echo hello"})).await;
        #[cfg(not(windows))]
        let result = tool.execute(json!({"command": "echo hello"})).await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_execute_failure() {
        let tool = BashTool;

        #[cfg(windows)]
        let result = tool.execute(json!({"command": "exit 1"})).await;
        #[cfg(not(windows))]
        let result = tool.execute(json!({"command": "exit 1"})).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Exit code: 1"));
    }

    #[tokio::test]
    async fn test_bash_execute_timeout() {
        let tool = BashTool;

        #[cfg(windows)]
        let result = tool
            .execute(json!({"command": "ping -n 10 localhost", "timeout": 100}))
            .await;
        #[cfg(not(windows))]
        let result = tool
            .execute(json!({"command": "sleep 10", "timeout": 100}))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_bash_working_directory() {
        let tool = BashTool;
        let temp_dir = std::env::temp_dir();

        #[cfg(windows)]
        let result = tool
            .execute(json!({"command": "cd", "working_dir": temp_dir.to_str()}))
            .await;
        #[cfg(not(windows))]
        let result = tool
            .execute(json!({"command": "pwd", "working_dir": temp_dir.to_str()}))
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        let temp_str = temp_dir.to_string_lossy().to_string();
        assert!(output.contains(&temp_str) || output.contains("Temp"));
    }

    #[tokio::test]
    async fn test_bash_missing_command() {
        let tool = BashTool;
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing command"));
    }
}
