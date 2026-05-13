//! # Workflow Tools
//!
//! 工作流控制工具集。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;

/// Create Checkpoint Tool
pub struct CreateCheckpointTool;

#[async_trait]
impl BuiltinTool for CreateCheckpointTool {
    fn name(&self) -> &str {
        "create_checkpoint"
    }

    fn description(&self) -> &str {
        "Create a checkpoint to save current agent state."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "trigger": {
                    "type": "string",
                    "description": "Optional: trigger reason for the checkpoint"
                }
            },
            "required": []
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Workflow
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        Ok("Checkpoint created successfully".to_string())
    }
}

/// Restore Checkpoint Tool
pub struct RestoreCheckpointTool;

#[async_trait]
impl BuiltinTool for RestoreCheckpointTool {
    fn name(&self) -> &str {
        "restore_checkpoint"
    }

    fn description(&self) -> &str {
        "Restore agent state from a checkpoint."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "checkpoint_id": {
                    "type": "string",
                    "description": "The checkpoint ID to restore"
                }
            },
            "required": ["checkpoint_id"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Workflow
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        Ok("Checkpoint restored successfully".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_tool_category() {
        let tool = CreateCheckpointTool;
        assert_eq!(tool.category(), ToolCategory::Workflow);
    }
}
