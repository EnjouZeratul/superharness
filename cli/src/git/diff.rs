//! Git diff 模块

use super::GitError;
use super::GitResult;
use std::path::Path;
use std::process::Command;

/// Diff 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    /// 工作区 vs 暂存区
    Working,
    /// 暂存区 vs HEAD
    Staged,
    /// 两个 commit 之间
    Commits,
}

/// Diff 条目
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub old_path: String,
    pub new_path: String,
    pub additions: usize,
    pub deletions: usize,
    pub diff_content: String,
}

/// Diff 结果
#[derive(Debug, Clone, Default)]
pub struct GitDiff {
    pub entries: Vec<DiffEntry>,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub files_changed: usize,
}

impl GitDiff {
    pub fn new() -> Self {
        Self::default()
    }

    /// 渲染 diff 输出
    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            " {} file(s) changed, {} insertion(s), {} deletion(s)\n\n",
            self.files_changed, self.total_additions, self.total_deletions
        ));

        for entry in &self.entries {
            output.push_str(&format!(
                "diff -- {} -> {}\n",
                entry.old_path, entry.new_path
            ));
            output.push_str(&format!("  +{} -{}\n", entry.additions, entry.deletions));

            if !entry.diff_content.is_empty() {
                output.push_str(&entry.diff_content);
                if !entry.diff_content.ends_with('\n') {
                    output.push('\n');
                }
            }
            output.push('\n');
        }

        output
    }

    /// 获取简短统计
    pub fn stat(&self) -> String {
        format!(
            "{} files changed, {} insertions(+), {} deletions(-)",
            self.files_changed, self.total_additions, self.total_deletions
        )
    }
}

/// 获取 Git diff
pub fn get_diff(repo_path: &Path, diff_type: DiffType, paths: &[&str]) -> GitResult<GitDiff> {
    let mut args = vec!["diff"];

    match diff_type {
        DiffType::Staged => args.push("--staged"),
        DiffType::Working => {}
        DiffType::Commits => {}
    }

    args.push("--stat");
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

    let _stat_stdout = String::from_utf8_lossy(&output.stdout);

    // 获取详细 diff
    let mut detail_args = vec!["diff"];
    if diff_type == DiffType::Staged {
        detail_args.push("--staged");
    }
    detail_args.extend(paths);

    let detail_output = Command::new("git")
        .args(&detail_args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;

    let detail_stdout = String::from_utf8_lossy(&detail_output.stdout).to_string();

    // 解析统计
    let mut diff = GitDiff::new();
    let mut current_additions = 0;
    let mut current_deletions = 0;

    for line in detail_stdout.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            current_additions += 1;
            diff.total_additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            current_deletions += 1;
            diff.total_deletions += 1;
        } else if line.starts_with("diff --git") {
            // 新文件差异
            if current_additions > 0 || current_deletions > 0 {
                // 之前文件有内容（已在下面处理）
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let file_paths = parts.get(3).unwrap_or(&"b/unknown");
            let path = file_paths.strip_prefix("b/").unwrap_or(file_paths);

            diff.entries.push(DiffEntry {
                old_path: path.to_string(),
                new_path: path.to_string(),
                additions: 0,
                deletions: 0,
                diff_content: String::new(),
            });
            diff.files_changed += 1;
            current_additions = 0;
            current_deletions = 0;
        }
    }

    // 如果没有解析到任何 diff 条目但有内容，整体作为一个条目
    if diff.entries.is_empty() && !detail_stdout.is_empty() {
        diff.entries.push(DiffEntry {
            old_path: "unknown".to_string(),
            new_path: "unknown".to_string(),
            additions: diff.total_additions,
            deletions: diff.total_deletions,
            diff_content: detail_stdout.clone(),
        });
        diff.files_changed = 1;
    }

    Ok(diff)
}

/// 获取简短 diff 统计
pub fn get_diff_stat(repo_path: &Path) -> GitResult<String> {
    let output = Command::new("git")
        .args(["diff", "--stat"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_diff_default() {
        let diff = GitDiff::default();
        assert!(diff.entries.is_empty());
        assert_eq!(diff.files_changed, 0);
    }

    #[test]
    fn test_diff_stat() {
        let mut diff = GitDiff::new();
        diff.files_changed = 3;
        diff.total_additions = 10;
        diff.total_deletions = 5;

        let stat = diff.stat();
        assert!(stat.contains("3 files"));
        assert!(stat.contains("10 insertions"));
        assert!(stat.contains("5 deletions"));
    }

    #[test]
    fn test_diff_render() {
        let mut diff = GitDiff::new();
        diff.files_changed = 1;
        diff.total_additions = 3;
        diff.total_deletions = 1;

        diff.entries.push(DiffEntry {
            old_path: "test.rs".to_string(),
            new_path: "test.rs".to_string(),
            additions: 3,
            deletions: 1,
            diff_content: "+new line\n-old line".to_string(),
        });

        let rendered = diff.render();
        assert!(rendered.contains("test.rs"));
        assert!(rendered.contains("+3"));
    }
}
