//! Git commit 模块
//!
//! 支持 commit 消息自动生成。

use super::GitError;
use super::GitResult;
use std::path::Path;
use std::process::Command;

/// Commit 消息生成器
pub struct CommitGenerator {
    /// 最大消息长度
    max_length: usize,
    /// 是否包含详细描述
    include_body: bool,
}

impl Default for CommitGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CommitGenerator {
    pub fn new() -> Self {
        Self {
            max_length: 72,
            include_body: true,
        }
    }

    /// 设置最大消息长度
    pub fn with_max_length(mut self, len: usize) -> Self {
        self.max_length = len;
        self
    }

    /// 设置是否包含详细描述
    pub fn with_body(mut self, include: bool) -> Self {
        self.include_body = include;
        self
    }

    /// 根据变更自动生成 commit 消息
    pub fn generate(
        &self,
        diff: &super::diff::GitDiff,
        status: &super::status::GitStatus,
    ) -> String {
        // 分析变更内容
        let changes = self.analyze_changes(diff, status);

        // 生成标题
        let title = self.generate_title(&changes);

        // 生成描述
        let body = if self.include_body {
            self.generate_body(&changes)
        } else {
            String::new()
        };

        if body.is_empty() {
            title
        } else {
            format!("{}\n\n{}", title, body)
        }
    }

    /// 分析变更
    fn analyze_changes(
        &self,
        diff: &super::diff::GitDiff,
        status: &super::status::GitStatus,
    ) -> ChangeAnalysis {
        let mut analysis = ChangeAnalysis::default();

        // 分析文件类型
        for entry in &status.entries {
            let path = &entry.path;

            // 检测变更类型
            if path.ends_with(".rs") {
                analysis.rust_changes += 1;
            } else if path.ends_with(".py") {
                analysis.python_changes += 1;
            } else if path.ends_with(".ts") || path.ends_with(".tsx") {
                analysis.typescript_changes += 1;
            } else if path.ends_with(".js") || path.ends_with(".jsx") {
                analysis.javascript_changes += 1;
            } else if path.ends_with(".toml") || path.ends_with("Cargo.toml") {
                analysis.config_changes += 1;
            } else if path.contains("test") || path.contains("_test") {
                analysis.test_changes += 1;
            } else if path.ends_with(".md") {
                analysis.docs_changes += 1;
            }

            // 检测变更操作
            match entry.status {
                super::status::FileStatus::New => analysis.new_files += 1,
                super::status::FileStatus::Deleted => analysis.deleted_files += 1,
                super::status::FileStatus::Modified => analysis.modified_files += 1,
                super::status::FileStatus::Renamed => analysis.renamed_files += 1,
                _ => {}
            }
        }

        analysis.files_changed = diff.files_changed;
        analysis.additions = diff.total_additions;
        analysis.deletions = diff.total_deletions;

        analysis
    }

    /// 生成标题
    fn generate_title(&self, analysis: &ChangeAnalysis) -> String {
        // 根据变更类型选择前缀
        let prefix = if analysis.test_changes > 0
            && analysis.rust_changes == 0
            && analysis.python_changes == 0
        {
            "test"
        } else if analysis.docs_changes > 0 && analysis.rust_changes == 0 {
            "docs"
        } else if analysis.config_changes > 0 && analysis.files_changed == 1 {
            "chore"
        } else if analysis.deleted_files > analysis.new_files {
            "refactor"
        } else if analysis.new_files > analysis.modified_files {
            "feat"
        } else {
            "fix"
        };

        // 生成描述
        let description = if analysis.files_changed == 1 {
            let file = if analysis.rust_changes > 0 {
                "rust module"
            } else if analysis.python_changes > 0 {
                "python module"
            } else if analysis.typescript_changes > 0 {
                "typescript module"
            } else {
                "file"
            };
            format!("update {}", file)
        } else if analysis.new_files > 0 {
            format!("add {} new file(s)", analysis.new_files)
        } else if analysis.deleted_files > 0 {
            format!("remove {} file(s)", analysis.deleted_files)
        } else if analysis.renamed_files > 0 {
            format!("rename {} file(s)", analysis.renamed_files)
        } else {
            format!("update {} file(s)", analysis.files_changed)
        };

        // 组合标题
        let title = format!("{}: {}", prefix, description);

        // 截断到最大长度
        if title.len() > self.max_length {
            format!("{}...", &title[..self.max_length - 3])
        } else {
            title
        }
    }

    /// 生成描述
    fn generate_body(&self, analysis: &ChangeAnalysis) -> String {
        let mut lines = Vec::new();

        if analysis.additions > 0 || analysis.deletions > 0 {
            lines.push(format!(
                "- {} addition(s), {} deletion(s) in {} file(s)",
                analysis.additions, analysis.deletions, analysis.files_changed
            ));
        }

        if analysis.test_changes > 0 {
            lines.push("- includes test changes".to_string());
        }

        if analysis.new_files > 0 {
            lines.push(format!("- {} new file(s) added", analysis.new_files));
        }

        if analysis.deleted_files > 0 {
            lines.push(format!("- {} file(s) removed", analysis.deleted_files));
        }

        lines.join("\n")
    }
}

/// 变更分析结果
#[derive(Debug, Default)]
struct ChangeAnalysis {
    files_changed: usize,
    additions: usize,
    deletions: usize,
    new_files: usize,
    deleted_files: usize,
    modified_files: usize,
    renamed_files: usize,
    rust_changes: usize,
    python_changes: usize,
    typescript_changes: usize,
    javascript_changes: usize,
    config_changes: usize,
    test_changes: usize,
    docs_changes: usize,
}

/// 执行 git commit
pub fn commit(repo_path: &Path, message: &str, amend: bool) -> GitResult<String> {
    let mut args = vec!["commit"];

    if amend {
        args.push("--amend");
    }

    args.extend(["-m", message]);

    let output = Command::new("git")
        .args(&args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("nothing to commit") {
            return Err(GitError::NoChanges);
        }
        return Err(GitError::CommandFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 暂存文件
pub fn add(repo_path: &Path, paths: &[&str]) -> GitResult<()> {
    let mut args = vec!["add"];
    args.extend(paths);

    let output = Command::new("git")
        .args(&args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

/// 暂存所有更改
pub fn add_all(repo_path: &Path) -> GitResult<()> {
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_generator_default() {
        let gen = CommitGenerator::new();
        assert_eq!(gen.max_length, 72);
        assert!(gen.include_body);
    }

    #[test]
    fn test_generate_title() {
        let gen = CommitGenerator::new();
        let analysis = ChangeAnalysis {
            files_changed: 3,
            new_files: 1,
            ..Default::default()
        };

        let title = gen.generate_title(&analysis);
        assert!(title.contains(":"));
        assert!(title.len() <= 72);
    }

    #[test]
    fn test_generate_with_diff() {
        let gen = CommitGenerator::new();
        let mut diff = super::super::diff::GitDiff::new();
        diff.files_changed = 2;
        diff.total_additions = 10;
        diff.total_deletions = 5;

        let status = super::super::status::GitStatus::new();

        let message = gen.generate(&diff, &status);
        assert!(!message.is_empty());
    }
}
