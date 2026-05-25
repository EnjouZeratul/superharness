//! # Code Analysis Tools
//!
//! 代码分析工具集：基于文件解析的符号查找工具。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Go to Definition Tool
///
/// 在文件中查找符号定义位置。使用正则表达式匹配函数、类、变量定义。
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
                },
                "symbol": {
                    "type": "string",
                    "description": "Optional: the symbol name to search for"
                }
            },
            "required": ["file", "line", "column"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::CodeAnalysis
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let file_path = args["file"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing file parameter"))?;

        let line = args["line"].as_u64().unwrap_or(1) as usize;
        let column = args["column"].as_u64().unwrap_or(1) as usize;
        let symbol = args["symbol"].as_str();

        // 读取文件
        let content = fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;

        let lines: Vec<&str> = content.lines().collect();

        // 获取当前行的符号
        let current_line = lines.get(line - 1).copied().unwrap_or("");
        let target_symbol = symbol.map(|s| s.to_string()).unwrap_or_else(|| {
            // 从当前位置提取符号名
            extract_symbol_at_position(current_line, column)
        });

        if target_symbol.is_empty() {
            return Ok("No symbol found at specified location".to_string());
        }

        // 根据文件类型确定定义模式
        let file_ext = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let definition_patterns = get_definition_patterns(file_ext, &target_symbol);

        // 搜索定义
        for pattern_str in definition_patterns {
            let pattern =
                Regex::new(&pattern_str).map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

            // 先在当前文件中搜索
            for (line_num, line_content) in lines.iter().enumerate() {
                if pattern.is_match(line_content) {
                    let match_info = pattern.find(line_content).unwrap();
                    return Ok(format!(
                        "Definition found in {} at line {}, column {}:\n{}",
                        file_path,
                        line_num + 1,
                        match_info.start() + 1,
                        line_content.trim()
                    ));
                }
            }

            // 如果当前文件没找到，搜索同目录的其他文件
            let dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() && entry_path != Path::new(file_path) {
                        let ext = entry_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");
                        if ext == file_ext {
                            if let Ok(other_content) = fs::read_to_string(&entry_path) {
                                for (line_num, line_content) in other_content.lines().enumerate() {
                                    if pattern.is_match(line_content) {
                                        return Ok(format!(
                                            "Definition found in {} at line {}:\n{}",
                                            entry_path.display(),
                                            line_num + 1,
                                            line_content.trim()
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(format!("No definition found for symbol: {}", target_symbol))
    }
}

/// Find References Tool
///
/// 查找符号的所有引用位置。
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
                },
                "symbol": {
                    "type": "string",
                    "description": "Optional: the symbol name to search for"
                },
                "include_declaration": {
                    "type": "boolean",
                    "description": "Include declaration in results (default: true)"
                }
            },
            "required": ["file", "line", "column"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::CodeAnalysis
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let file_path = args["file"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing file parameter"))?;

        let line = args["line"].as_u64().unwrap_or(1) as usize;
        let column = args["column"].as_u64().unwrap_or(1) as usize;
        let symbol = args["symbol"].as_str();
        let include_declaration = args["include_declaration"].as_bool().unwrap_or(true);

        // 读取文件
        let content = fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;

        let lines: Vec<&str> = content.lines().collect();
        let current_line = lines.get(line - 1).copied().unwrap_or("");

        // 提取符号
        let target_symbol = symbol
            .map(|s| s.to_string())
            .unwrap_or_else(|| extract_symbol_at_position(current_line, column));

        if target_symbol.is_empty() {
            return Ok("No symbol found at specified location".to_string());
        }

        // 构建引用搜索模式
        let reference_pattern = Regex::new(&format!(r"\b{}\b", target_symbol))
            .map_err(|e| anyhow::anyhow!("Invalid regex for symbol: {}", e))?;

        let mut results = Vec::new();

        // 搜索当前文件
        for (line_num, line_content) in lines.iter().enumerate() {
            if reference_pattern.is_match(line_content) {
                // 检查是否是定义行
                let is_declaration = is_definition_line(line_content, &target_symbol);
                if include_declaration || !is_declaration {
                    let matches: Vec<_> = reference_pattern.find_iter(line_content).collect();
                    for m in matches {
                        results.push(format!(
                            "{}:{}:{} - {}",
                            file_path,
                            line_num + 1,
                            m.start() + 1,
                            line_content.trim()
                        ));
                    }
                }
            }
        }

        // 搜索同目录的其他文件
        let dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
        let file_ext = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() && entry_path != Path::new(file_path) {
                    let ext = entry_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if ext == file_ext {
                        if let Ok(other_content) = fs::read_to_string(&entry_path) {
                            for (line_num, line_content) in other_content.lines().enumerate() {
                                if reference_pattern.is_match(line_content) {
                                    let is_decl = is_definition_line(line_content, &target_symbol);
                                    if include_declaration || !is_decl {
                                        let matches: Vec<_> =
                                            reference_pattern.find_iter(line_content).collect();
                                        for m in matches {
                                            results.push(format!(
                                                "{}:{}:{} - {}",
                                                entry_path.display(),
                                                line_num + 1,
                                                m.start() + 1,
                                                line_content.trim()
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            Ok(format!("No references found for symbol: {}", target_symbol))
        } else {
            Ok(format!(
                "Found {} references:\n{}",
                results.len(),
                results.join("\n")
            ))
        }
    }
}

/// 从指定位置提取符号名
fn extract_symbol_at_position(line: &str, column: usize) -> String {
    let line_bytes = line.as_bytes();
    if column == 0 || column > line.len() {
        return String::new();
    }

    // 找到符号的起始位置
    let start = line_bytes[..column - 1]
        .iter()
        .rposition(|&b| !is_identifier_char(b))
        .map(|p| p + 1)
        .unwrap_or(0);

    // 找到符号的结束位置
    let end = line_bytes[column - 1..]
        .iter()
        .position(|&b| !is_identifier_char(b))
        .map(|p| column - 1 + p)
        .unwrap_or(line.len());

    line[start..end].to_string()
}

/// 检查是否是标识符字符
fn is_identifier_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':'
}

/// 检查是否是定义行
fn is_definition_line(line: &str, symbol: &str) -> bool {
    let patterns = [
        Regex::new(&format!(r"\bfn\s+{}\s*\(", symbol)).ok(),
        Regex::new(&format!(r"\bdef\s+{}\s*\(", symbol)).ok(),
        Regex::new(&format!(r"\bclass\s+{}", symbol)).ok(),
        Regex::new(&format!(r"\bstruct\s+{}", symbol)).ok(),
        Regex::new(&format!(r"\benum\s+{}", symbol)).ok(),
        Regex::new(&format!(r"\bimpl\s+{}", symbol)).ok(),
        Regex::new(&format!(r"\btrait\s+{}", symbol)).ok(),
        Regex::new(&format!(r"\binterface\s+{}", symbol)).ok(),
        Regex::new(&format!(r"\btype\s+{}\s*=", symbol)).ok(),
        Regex::new(&format!(r"\bconst\s+{}", symbol)).ok(),
        Regex::new(&format!(r"\blet\s+{}\s*=", symbol)).ok(),
        Regex::new(&format!(r"\bvar\s+{}\s*=", symbol)).ok(),
        Regex::new(&format!(r"\bpublic\s+{}\s*\(", symbol)).ok(),
        Regex::new(&format!(r"\bprivate\s+{}\s*\(", symbol)).ok(),
    ];

    patterns
        .iter()
        .any(|p| p.as_ref().map_or(false, |r| r.is_match(line)))
}

/// 根据文件类型获取定义匹配模式
fn get_definition_patterns(file_ext: &str, symbol: &str) -> Vec<String> {
    match file_ext {
        "rs" => vec![
            format!(r"\bfn\s+{}\s*[<(]", symbol),
            format!(r"\bstruct\s+{}\s*[{{<\s]", symbol),
            format!(r"\benum\s+{}\s*[{{<\s]", symbol),
            format!(r"\btrait\s+{}\s*[{{<\s]", symbol),
            format!(r"\bimpl\s+(?:\w+\s+for\s+)?{}|impl\s+{}", symbol, symbol),
            format!(r"\btype\s+{}\s*=", symbol),
            format!(r"\bconst\s+{}\s*:", symbol),
            format!(r"\bstatic\s+{}\s*:", symbol),
            format!(r"\bmacro_rules!\s+{}", symbol),
        ],
        "py" => vec![
            format!(r"\bdef\s+{}\s*\(", symbol),
            format!(r"\bclass\s+{}\s*[:\(]", symbol),
            format!(r"\basync\s+def\s+{}\s*\(", symbol),
        ],
        "js" | "ts" | "tsx" => vec![
            format!(r"\bfunction\s+{}\s*\(", symbol),
            format!(r"\bclass\s+{}\s*[{{extends\s]", symbol),
            format!(r"\bconst\s+{}\s*=", symbol),
            format!(r"\blet\s+{}\s*=", symbol),
            format!(r"\bvar\s+{}\s*=", symbol),
            format!(r"\binterface\s+{}\s*[{{extends\s]", symbol),
            format!(r"\btype\s+{}\s*=", symbol),
            format!(r"\bexport\s+(?:default\s+)?(?:function|class)\s+{}", symbol),
        ],
        "java" | "kt" => vec![
            format!(r"\bclass\s+{}\s*[{{extends\s]", symbol),
            format!(r"\binterface\s+{}\s*[{{extends\s]", symbol),
            format!(
                r"\b(?:public|private|protected)\s+(?:static\s+)?(?:\w+\s+)?{}\s*\(",
                symbol
            ),
            format!(r"\benum\s+{}\s*[{{]", symbol),
        ],
        "go" => vec![
            format!(r"\bfunc\s+{}\s*\(", symbol),
            format!(r"\bfunc\s+\(\w+\s*\*?\w*\)\s+{}\s*\(", symbol), // 方法
            format!(r"\btype\s+{}\s+struct", symbol),
            format!(r"\btype\s+{}\s+interface", symbol),
            format!(r"\bvar\s+{}\s*=", symbol),
            format!(r"\bconst\s+{}\s*=", symbol),
        ],
        "c" | "cpp" | "h" | "hpp" => vec![
            format!(
                r"\b(?:void|int|char|float|double|auto|struct|class)\s+{}\s*\(",
                symbol
            ),
            format!(r"\bstruct\s+{}\s*[{{]", symbol),
            format!(r"\bclass\s+{}\s*[{{:]", symbol),
            format!(r"\btypedef\s+.*\s+{}\s*;", symbol),
        ],
        _ => vec![
            format!(r"\b{}\s*[:=]", symbol),
            format!(r"\b{}\s*\(", symbol),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_goto_definition_category() {
        let tool = GoToDefinitionTool;
        assert_eq!(tool.category(), ToolCategory::CodeAnalysis);
    }

    #[test]
    fn test_find_references_category() {
        let tool = FindReferencesTool;
        assert_eq!(tool.category(), ToolCategory::CodeAnalysis);
    }

    #[test]
    fn test_extract_symbol() {
        let line = "let my_variable = 42;";
        let symbol = extract_symbol_at_position(line, 10);
        assert_eq!(symbol, "my_variable");
    }

    #[test]
    fn test_extract_symbol_empty() {
        let line = "    = 42;";
        let symbol = extract_symbol_at_position(line, 5);
        assert_eq!(symbol, "");
    }

    #[test]
    fn test_is_definition_line() {
        assert!(is_definition_line("fn my_func() {", "my_func"));
        assert!(is_definition_line("struct MyStruct {", "MyStruct"));
        assert!(!is_definition_line("my_func();", "my_func"));
    }

    #[test]
    fn test_get_definition_patterns_rust() {
        let patterns = get_definition_patterns("rs", "foo");
        assert!(patterns.iter().any(|p| p.contains("fn")));
        assert!(patterns.iter().any(|p| p.contains("struct")));
    }

    #[tokio::test]
    async fn test_goto_definition_missing_file() {
        let tool = GoToDefinitionTool;
        let result = tool
            .execute(json!({"file": "nonexistent.rs", "line": 1, "column": 1}))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    #[tokio::test]
    async fn test_find_references_missing_file() {
        let tool = FindReferencesTool;
        let result = tool
            .execute(json!({"file": "nonexistent.rs", "line": 1, "column": 1}))
            .await;
        assert!(result.is_err());
    }
}
