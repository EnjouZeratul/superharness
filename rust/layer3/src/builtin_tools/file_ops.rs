//! # File Operations Tools
//!
//! 文件操作工具集：读写、编辑、创建、删除等。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;

/// Read File Tool
pub struct ReadFileTool;

#[async_trait]
impl BuiltinTool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file from the filesystem."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Optional: line number to start reading from"
                },
                "limit": {
                    "type": "integer",
                    "description": "Optional: number of lines to read"
                }
            },
            "required": ["path"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileOps
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
        let content = tokio::fs::read_to_string(path).await?;
        Ok(content)
    }
}

/// Write File Tool
pub struct WriteFileTool;

#[async_trait]
impl BuiltinTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file, creating it if it doesn't exist."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileOps
    }

    fn is_dangerous(&self) -> bool {
        true
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content parameter"))?;
        tokio::fs::write(path, content).await?;
        Ok(format!("Successfully wrote to {}", path))
    }
}

/// Edit File Tool (search and replace)
pub struct EditFileTool;

#[async_trait]
impl BuiltinTool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing specific text with new text."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "The text to search for and replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The text to replace with"
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileOps
    }

    fn is_dangerous(&self) -> bool {
        true
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
        let old_string = args["old_string"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing old_string parameter"))?;
        let new_string = args["new_string"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing new_string parameter"))?;

        let content = tokio::fs::read_to_string(path).await?;
        if !content.contains(old_string) {
            return Err(anyhow::anyhow!("old_string not found in file"));
        }

        let new_content = content.replace(old_string, new_string);
        tokio::fs::write(path, new_content).await?;
        Ok(format!("Successfully edited {}", path))
    }
}

/// List Directory Tool
pub struct ListDirectoryTool;

#[async_trait]
impl BuiltinTool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List files and directories in a given path."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The directory path to list"
                }
            },
            "required": ["path"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::FileOps
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut result = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = if entry.file_type().await?.is_dir() {
                "dir"
            } else {
                "file"
            };
            result.push(format!("{} [{}]", name, file_type));
        }

        Ok(result.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file_tool_meta() {
        let tool = ReadFileTool;
        assert_eq!(tool.name(), "read_file");
        assert_eq!(tool.category(), ToolCategory::FileOps);
    }

    #[test]
    fn test_write_file_tool_dangerous() {
        let tool = WriteFileTool;
        assert!(tool.is_dangerous());
        assert!(tool.requires_confirmation());
    }
}
