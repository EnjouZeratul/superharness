//! # Search Tools
//!
//! 搜索工具集：grep、glob、文件搜索等。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Grep Tool - Search content in files
pub struct GrepTool;

impl GrepTool {
    /// Search for pattern in a single file
    fn search_file(
        &self,
        path: &Path,
        pattern: &Regex,
        max_results: usize,
    ) -> Layer3Result<Vec<(usize, String)>> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            if results.len() >= max_results {
                break;
            }
            let line = line_result?;
            if pattern.is_match(&line) {
                results.push((line_num + 1, line));
            }
        }

        Ok(results)
    }

    /// Recursively collect files in directory
    fn collect_files(&self, dir: &Path, glob_pattern: Option<&str>) -> Layer3Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        fn walk_dir(dir: &Path, files: &mut Vec<std::path::PathBuf>, glob_filter: Option<&str>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip hidden directories
                        if !path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(false) {
                            walk_dir(&path, files, glob_filter);
                        }
                    } else if path.is_file() {
                        // Apply glob filter if provided
                        let include = if let Some(glob) = glob_filter {
                            // Simple glob matching: *.ext or **/*.ext
                            let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                            if glob.starts_with("**/") {
                                let suffix = &glob[3..];
                                file_name.ends_with(suffix.trim_start_matches('*'))
                            } else if glob.starts_with("*") {
                                file_name.ends_with(&glob[1..])
                            } else {
                                file_name == glob
                            }
                        } else {
                            true
                        };
                        if include {
                            files.push(path);
                        }
                    }
                }
            }
        }

        walk_dir(dir, &mut files, glob_pattern);
        Ok(files)
    }
}

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
                    "description": "Optional: glob pattern to filter files (e.g., '*.rs')"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Optional: case sensitive search (default: false)"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Optional: maximum results to return (default: 100)"
                }
            },
            "required": ["pattern"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Search
    }

    #[allow(unused_assignments)]
    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let pattern_str = args["pattern"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing pattern parameter"))?;

        let path_str = args["path"].as_str().unwrap_or(".");
        let glob_pattern = args["glob"].as_str();
        let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
        let max_results = args["max_results"].as_u64().unwrap_or(100) as usize;

        // Build regex
        let mut regex_builder = Regex::new(pattern_str);
        if !case_sensitive {
            regex_builder = Regex::new(&format!("(?i){}", pattern_str));
        }

        let pattern = regex_builder
            .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

        let search_path = Path::new(path_str);

        if !search_path.exists() {
            return Err(anyhow::anyhow!("Path not found: {}", path_str));
        }

        let mut output_lines = Vec::new();
        let mut total_matches = 0;

        if search_path.is_file() {
            // Search single file
            let results = self.search_file(search_path, &pattern, max_results)?;
            for (line_num, line) in results {
                output_lines.push(format!("{}:{}: {}", search_path.display(), line_num, line));
                total_matches += 1;
            }
        } else if search_path.is_dir() {
            // Search directory
            let files = self.collect_files(search_path, glob_pattern)?;

            for file in files {
                if total_matches >= max_results {
                    break;
                }
                if let Ok(results) = self.search_file(&file, &pattern, max_results - total_matches) {
                    for (line_num, line) in results {
                        output_lines.push(format!("{}:{}: {}", file.display(), line_num, line));
                        total_matches += 1;
                        if total_matches >= max_results {
                            break;
                        }
                    }
                }
            }
        }

        if output_lines.is_empty() {
            Ok("(no matches)".to_string())
        } else {
            Ok(output_lines.join("\n"))
        }
    }
}

/// Glob Tool - Find files by pattern
pub struct GlobTool;

impl GlobTool {
    /// Simple glob matching
    fn matches_pattern(file_name: &str, pattern: &str) -> bool {
        if pattern == "**/*" {
            return true;
        }

        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            if suffix.starts_with('*') {
                return file_name.ends_with(&suffix[1..]);
            }
            return file_name == suffix;
        }

        if pattern.starts_with('*') {
            return file_name.ends_with(&pattern[1..]);
        }

        if pattern.ends_with('*') {
            return file_name.starts_with(&pattern[..pattern.len() - 1]);
        }

        file_name == pattern
    }

    /// Collect files matching pattern
    fn collect_matching_files(
        &self,
        dir: &Path,
        pattern: &str,
    ) -> Layer3Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        fn walk_dir(dir: &Path, files: &mut Vec<std::path::PathBuf>, pattern: &str) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip hidden directories
                        if !path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(false) {
                            walk_dir(&path, files, pattern);
                        }
                    } else if path.is_file() {
                        let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                        if GlobTool::matches_pattern(&file_name, pattern) {
                            files.push(path);
                        }
                    }
                }
            }
        }

        walk_dir(dir, &mut files, pattern);
        // Sort by modification time (newest first)
        files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
            let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        Ok(files)
    }
}

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
                    "description": "The glob pattern (e.g., '**/*.rs', '*.txt')"
                },
                "path": {
                    "type": "string",
                    "description": "Optional: the directory to search in (default: current directory)"
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

        let path_str = args["path"].as_str().unwrap_or(".");
        let search_path = Path::new(path_str);

        if !search_path.exists() {
            return Err(anyhow::anyhow!("Path not found: {}", path_str));
        }

        if !search_path.is_dir() {
            return Err(anyhow::anyhow!("Not a directory: {}", path_str));
        }

        let files = self.collect_matching_files(search_path, pattern)?;

        if files.is_empty() {
            Ok("(no matches)".to_string())
        } else {
            let output: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
            Ok(output.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_grep_tool_category() {
        let tool = GrepTool;
        assert_eq!(tool.category(), ToolCategory::Search);
    }

    #[test]
    fn test_glob_tool_category() {
        let tool = GlobTool;
        assert_eq!(tool.category(), ToolCategory::Search);
    }

    #[tokio::test]
    async fn test_grep_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "hello world").unwrap();
        writeln!(file, "foo bar").unwrap();
        writeln!(file, "hello again").unwrap();

        let tool = GrepTool;
        let result = tool.execute(json!({
            "pattern": "hello",
            "path": file_path.to_str().unwrap()
        })).await.unwrap();

        assert!(result.contains("hello"));
        assert!(!result.contains("foo"));
    }

    #[tokio::test]
    async fn test_grep_directory() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let mut f1 = fs::File::create(&file1).unwrap();
        writeln!(f1, "fn main() {{ }}").unwrap();

        let file2 = temp_dir.path().join("file2.txt");
        let mut f2 = fs::File::create(&file2).unwrap();
        writeln!(f2, "fn test() {{ }}").unwrap();

        let tool = GrepTool;
        let result = tool.execute(json!({
            "pattern": "fn\\s+\\w+",
            "path": temp_dir.path().to_str().unwrap(),
            "glob": "*.txt"
        })).await.unwrap();

        assert!(result.contains("fn main"));
        assert!(result.contains("fn test"));
    }

    #[tokio::test]
    async fn test_grep_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "HELLO World").unwrap();

        let tool = GrepTool;
        let result = tool.execute(json!({
            "pattern": "hello",
            "path": file_path.to_str().unwrap(),
            "case_sensitive": false
        })).await.unwrap();

        assert!(result.contains("HELLO"));
    }

    #[tokio::test]
    async fn test_grep_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "hello world").unwrap();

        let tool = GrepTool;
        let result = tool.execute(json!({
            "pattern": "nonexistent",
            "path": file_path.to_str().unwrap()
        })).await.unwrap();

        assert!(result.contains("no matches"));
    }

    #[tokio::test]
    async fn test_grep_invalid_pattern() {
        let tool = GrepTool;
        let result = tool.execute(json!({
            "pattern": "[invalid("
        })).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid regex"));
    }

    #[tokio::test]
    async fn test_glob_find_files() {
        let temp_dir = TempDir::new().unwrap();

        fs::File::create(temp_dir.path().join("file1.rs")).unwrap();
        fs::File::create(temp_dir.path().join("file2.rs")).unwrap();
        fs::File::create(temp_dir.path().join("file3.txt")).unwrap();

        let tool = GlobTool;
        let result = tool.execute(json!({
            "pattern": "*.rs",
            "path": temp_dir.path().to_str().unwrap()
        })).await.unwrap();

        assert!(result.contains("file1.rs"));
        assert!(result.contains("file2.rs"));
        assert!(!result.contains("file3.txt"));
    }

    #[tokio::test]
    async fn test_glob_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("nested");
        fs::create_dir(&subdir).unwrap();

        fs::File::create(subdir.join("deep.rs")).unwrap();

        let tool = GlobTool;
        let result = tool.execute(json!({
            "pattern": "**/*.rs",
            "path": temp_dir.path().to_str().unwrap()
        })).await.unwrap();

        assert!(result.contains("deep.rs"));
    }

    #[tokio::test]
    async fn test_glob_no_matches() {
        let temp_dir = TempDir::new().unwrap();

        let tool = GlobTool;
        let result = tool.execute(json!({
            "pattern": "*.xyz",
            "path": temp_dir.path().to_str().unwrap()
        })).await.unwrap();

        assert!(result.contains("no matches"));
    }

    #[tokio::test]
    async fn test_glob_nonexistent_path() {
        let tool = GlobTool;
        let result = tool.execute(json!({
            "pattern": "*.rs",
            "path": "/nonexistent/path"
        })).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path not found"));
    }

    #[test]
    fn test_glob_pattern_matching() {
        assert!(GlobTool::matches_pattern("test.rs", "*.rs"));
        assert!(GlobTool::matches_pattern("test.rs", "**/*.rs"));
        assert!(!GlobTool::matches_pattern("test.txt", "*.rs"));
        assert!(GlobTool::matches_pattern("test.txt", "*.txt"));
    }
}