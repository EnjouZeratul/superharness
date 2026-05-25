//! continuum run 子命令
//!
//! 执行 Agent 任务，支持交互模式和非交互模式。

use crate::commands::tool_exec;
use anyhow::Result;

/// 执行 run 命令
pub fn execute(
    task: Option<String>,
    _config: Option<String>,
    _budget: Option<f64>,
    debug: bool,
    non_interactive: bool,
) -> Result<()> {
    if debug {
        println!("Debug mode enabled");
    }

    match task {
        Some(t) => {
            println!("Running task: {}", t);

            // 解析任务并执行相应工具
            let response = execute_task(&t, debug)?;

            // 输出结果
            println!("\n--- Result ---");
            println!("{}", response);
        }
        None => {
            if non_interactive {
                println!("No task specified. Use --help for usage.");
            } else {
                println!("Starting interactive mode...");
                println!("(Interactive TUI mode - use 'continuum tui' command)");
                println!("\nQuick commands:");
                println!("  continuum run \"list files\"     - List files in current directory");
                println!("  continuum run \"bash: echo hi\"  - Run shell command");
                println!("  continuum run \"grep: pattern\"  - Search for pattern");
                println!("  continuum run \"glob: *.rs\"     - Find Rust files");
            }
        }
    }

    Ok(())
}

/// 执行任务
fn execute_task(task: &str, debug: bool) -> Result<String> {
    let (tool_name, params) = parse_task(task);

    if debug {
        println!("Using tool: {} with params: {:?}", tool_name, params);
    }

    match tool_name.as_str() {
        "bash" => {
            let command = params
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or(task);
            let result = tool_exec::execute_bash(command, None, 30, false)?;
            if result.exit_code != 0 {
                Ok(format!(
                    "Error (exit {}): {}",
                    result.exit_code, result.stderr
                ))
            } else {
                Ok(result.stdout)
            }
        }
        "read_file" => {
            let path = params.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            tool_exec::execute_read(path, None, None, false)
        }
        "write_file" => {
            let path = params
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("output.txt");
            let content = params.get("content").and_then(|c| c.as_str()).unwrap_or("");
            tool_exec::execute_write(path, Some(content), false, false)
        }
        "grep" => {
            let pattern = params.get("pattern").and_then(|p| p.as_str()).unwrap_or("");
            let path = params.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            let results = tool_exec::execute_grep(pattern, path, None, false, true, None)?;
            if results.is_empty() {
                Ok("(no matches)".to_string())
            } else {
                Ok(results
                    .iter()
                    .map(|m| format!("{}:{}: {}", m.file, m.line_number, m.line))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
        "glob" => {
            let pattern = params
                .get("pattern")
                .and_then(|p| p.as_str())
                .unwrap_or("*");
            let path = params.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            let results = tool_exec::execute_glob(pattern, path)?;
            if results.is_empty() {
                Ok("(no matches)".to_string())
            } else {
                Ok(results.join("\n"))
            }
        }
        "list_directory" => {
            let path = params.get("path").and_then(|p| p.as_str()).unwrap_or(".");
            list_directory(path)
        }
        _ => {
            // 尝试作为 shell 命令执行
            let result = tool_exec::execute_bash(task, None, 30, false)?;
            if result.exit_code != 0 {
                Ok(format!(
                    "Error (exit {}): {}",
                    result.exit_code, result.stderr
                ))
            } else {
                Ok(result.stdout)
            }
        }
    }
}

/// 列出目录内容
fn list_directory(path: &str) -> Result<String> {
    use std::fs;

    let entries = fs::read_dir(path)?;
    let mut result = Vec::new();

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let file_type = if entry.path().is_dir() { "dir" } else { "file" };
        result.push(format!("{}  [{}]", name, file_type));
    }

    if result.is_empty() {
        Ok("(empty directory)".to_string())
    } else {
        Ok(result.join("\n"))
    }
}

/// 解析任务字符串
fn parse_task(task: &str) -> (String, serde_json::Value) {
    let task_lower = task.to_lowercase();

    // 检查是否有显式工具前缀
    if task_lower.starts_with("bash:") || task_lower.starts_with("run:") {
        let cmd = task
            .split_once(':')
            .map(|(_, rest)| rest)
            .unwrap_or(task)
            .trim();
        return ("bash".to_string(), serde_json::json!({"command": cmd}));
    }

    if task_lower.starts_with("grep:") || task_lower.starts_with("search:") {
        let pattern = task
            .split_once(':')
            .map(|(_, rest)| rest)
            .unwrap_or(task)
            .trim();
        return (
            "grep".to_string(),
            serde_json::json!({"pattern": pattern, "path": "."}),
        );
    }

    if task_lower.starts_with("glob:") || task_lower.starts_with("find:") {
        let pattern = task
            .split_once(':')
            .map(|(_, rest)| rest)
            .unwrap_or(task)
            .trim();
        return (
            "glob".to_string(),
            serde_json::json!({"pattern": pattern, "path": "."}),
        );
    }

    if task_lower.starts_with("read:") || task_lower.starts_with("cat:") {
        let path = task
            .split_once(':')
            .map(|(_, rest)| rest)
            .unwrap_or(task)
            .trim();
        return ("read_file".to_string(), serde_json::json!({"path": path}));
    }

    if task_lower.starts_with("write:") {
        let parts: Vec<&str> = task.splitn(3, ':').collect();
        if parts.len() >= 3 {
            return (
                "write_file".to_string(),
                serde_json::json!({"path": parts[1].trim(), "content": parts[2].trim()}),
            );
        }
    }

    // 基于���键词自动选择工具
    if task_lower.contains("list")
        && (task_lower.contains("file")
            || task_lower.contains("dir")
            || task_lower.contains("directory"))
    {
        return (
            "list_directory".to_string(),
            serde_json::json!({"path": "."}),
        );
    }

    if task_lower.contains("find") && (task_lower.contains(".py") || task_lower.contains("python"))
    {
        return (
            "glob".to_string(),
            serde_json::json!({"pattern": "**/*.py", "path": "."}),
        );
    }

    if task_lower.contains("find") && (task_lower.contains(".rs") || task_lower.contains("rust")) {
        return (
            "glob".to_string(),
            serde_json::json!({"pattern": "**/*.rs", "path": "."}),
        );
    }

    if task_lower.contains("search") || task_lower.contains("grep") {
        let words: Vec<&str> = task.split_whitespace().collect();
        if words.len() > 1 {
            // 尝试提取最后一个词作为 pattern
            return (
                "grep".to_string(),
                serde_json::json!({"pattern": words.last().unwrap(), "path": "."}),
            );
        }
    }

    // 默认：尝试作为 shell 命令执行
    ("bash".to_string(), serde_json::json!({"command": task}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_bash() {
        let (tool, args) = parse_task("bash: echo hello");
        assert_eq!(tool, "bash");
        assert_eq!(args["command"], "echo hello");
    }

    #[test]
    fn test_parse_task_grep() {
        let (tool, args) = parse_task("grep: fn main");
        assert_eq!(tool, "grep");
        assert_eq!(args["pattern"], "fn main");
    }

    #[test]
    fn test_parse_task_implicit_list() {
        let (tool, _args) = parse_task("list files");
        assert_eq!(tool, "list_directory");
    }

    #[test]
    fn test_parse_task_implicit_glob() {
        let (tool, args) = parse_task("find python files");
        assert_eq!(tool, "glob");
        assert_eq!(args["pattern"], "**/*.py");
    }
}
