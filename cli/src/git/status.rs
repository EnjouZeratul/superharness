//! Git status 模块

use super::GitError;
use super::GitResult;
use std::path::Path;
use std::process::Command;

/// 文件状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    /// 未跟踪
    Untracked,
    /// 已修改
    Modified,
    /// 已暂存
    Staged,
    /// 已删除
    Deleted,
    /// 新文件
    New,
    /// 重命名
    Renamed,
    /// 复制
    Copied,
    /// 忽略
    Ignored,
}

impl FileStatus {
    /// 从状态码解析 (支持 porcelain v1 和 v2 格式)
    ///
    /// porcelain v1: "??", "M ", " A", etc.
    /// porcelain v2: "?", "M.", ".M", "MM", "A.", etc.
    /// XY format: X=index status, Y=worktree status
    pub fn from_code(code: &str) -> Self {
        match code {
            // Untracked (v1: ??, v2: ?)
            "?" | "??" => FileStatus::Untracked,
            // Ignored (v1: !!, v2: !)
            "!" | "!!" => FileStatus::Ignored,
            // New/staged (A in index)
            "A" | "A " | "A." | "AM" => FileStatus::New,
            // Deleted in index
            "D" | "D " | "D." => FileStatus::Deleted,
            // Deleted in worktree
            " D" | ".D" => FileStatus::Deleted,
            // Renamed
            "R" | "R " | "R." => FileStatus::Renamed,
            // Copied
            "C" | "C " | "C." => FileStatus::Copied,
            // Modified in index (staged)
            "M" | "M " | "M." => FileStatus::Staged,
            // Modified in worktree (unstaged)
            " M" | ".M" => FileStatus::Modified,
            // Both staged and worktree modified
            "MM" => FileStatus::Modified,
            // Default fallback
            _ => {
                // Check first character for staged status
                let chars: Vec<char> = code.chars().collect();
                if chars.len() >= 2 {
                    match chars[0] {
                        'A' => FileStatus::New,
                        'M' => FileStatus::Staged,
                        'D' => FileStatus::Deleted,
                        'R' => FileStatus::Renamed,
                        'C' => FileStatus::Copied,
                        _ => FileStatus::Modified,
                    }
                } else {
                    FileStatus::Modified
                }
            }
        }
    }

    /// 是否已暂存
    pub fn is_staged(&self) -> bool {
        matches!(
            self,
            FileStatus::Staged | FileStatus::New | FileStatus::Renamed
        )
    }

    /// 获取显示文本
    pub fn display(&self) -> &'static str {
        match self {
            FileStatus::Untracked => "??",
            FileStatus::Modified => " M",
            FileStatus::Staged => "M ",
            FileStatus::Deleted => " D",
            FileStatus::New => "A ",
            FileStatus::Renamed => "R ",
            FileStatus::Copied => "C ",
            FileStatus::Ignored => "!!",
        }
    }
}

/// Git 状态条目
#[derive(Debug, Clone)]
pub struct StatusEntry {
    pub status: FileStatus,
    pub path: String,
    pub old_path: Option<String>,
}

/// Git 状态
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub entries: Vec<StatusEntry>,
    pub stash_count: usize,
    pub is_rebasing: bool,
    pub is_merging: bool,
}

impl GitStatus {
    /// 创建新的状态实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查是否有未提交的更改
    pub fn has_changes(&self) -> bool {
        !self.entries.is_empty()
    }

    /// 获取已暂存的文件
    pub fn staged_files(&self) -> Vec<&StatusEntry> {
        self.entries
            .iter()
            .filter(|e| e.status.is_staged())
            .collect()
    }

    /// 获取未暂存的文件
    pub fn unstaged_files(&self) -> Vec<&StatusEntry> {
        self.entries
            .iter()
            .filter(|e| !e.status.is_staged())
            .collect()
    }

    /// 获取未跟踪的文件
    pub fn untracked_files(&self) -> Vec<&StatusEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.status, FileStatus::Untracked))
            .collect()
    }

    /// 获取状态摘要
    pub fn summary(&self) -> String {
        let staged = self.staged_files().len();
        let unstaged = self.unstaged_files().len();
        let untracked = self.untracked_files().len();

        let mut parts = vec![format!("On branch {}", self.branch)];

        if self.ahead > 0 {
            parts.push(format!("ahead {}", self.ahead));
        }
        if self.behind > 0 {
            parts.push(format!("behind {}", self.behind));
        }

        if staged > 0 {
            parts.push(format!("{} staged", staged));
        }
        if unstaged > 0 {
            parts.push(format!("{} modified", unstaged));
        }
        if untracked > 0 {
            parts.push(format!("{} untracked", untracked));
        }

        if self.stash_count > 0 {
            parts.push(format!("{} stash", self.stash_count));
        }

        parts.join(", ")
    }

    /// 渲染状态输出
    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("On branch {}\n", self.branch));

        if self.ahead > 0 || self.behind > 0 {
            let mut tracking = String::new();
            if self.ahead > 0 {
                tracking.push_str(&format!("ahead {} ", self.ahead));
            }
            if self.behind > 0 {
                tracking.push_str(&format!("behind {} ", self.behind));
            }
            output.push_str(&format!("Your branch is {}.\n", tracking.trim()));
        }

        let staged = self.staged_files();
        let unstaged = self.unstaged_files();
        let untracked = self.untracked_files();

        if staged.is_empty() && unstaged.is_empty() && untracked.is_empty() {
            output.push_str("nothing to commit, working tree clean\n");
        } else {
            if !staged.is_empty() {
                output.push_str("\nChanges to be committed:\n");
                for entry in staged {
                    output.push_str(&format!("\t{}\t{}\n", entry.status.display(), entry.path));
                }
            }

            if !unstaged.is_empty() {
                output.push_str("\nChanges not staged for commit:\n");
                for entry in unstaged {
                    output.push_str(&format!("\t{}\t{}\n", entry.status.display(), entry.path));
                }
            }

            if !untracked.is_empty() {
                output.push_str("\nUntracked files:\n");
                for entry in untracked {
                    output.push_str(&format!("\t{}\n", entry.path));
                }
            }
        }

        output
    }
}

/// 获取 Git 状态
pub fn get_status(repo_path: &Path) -> GitResult<GitStatus> {
    // 执行 git status --porcelain=v2 --branch
    let output = Command::new("git")
        .args(["status", "--porcelain=v2", "--branch"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::NotARepository);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut status = GitStatus::new();

    for line in stdout.lines() {
        if line.starts_with("# branch.head") {
            status.branch = line
                .split_whitespace()
                .nth(2)
                .unwrap_or("unknown")
                .to_string();
        } else if line.starts_with("# branch.ab") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.starts_with('+') {
                    status.ahead = part[1..].parse().unwrap_or(0);
                } else if part.starts_with('-') {
                    status.behind = part[1..].parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("1 ")
            || line.starts_with("2 ")
            || line.starts_with("u ")
            || line.starts_with("? ")
            || line.starts_with("! ")
        {
            // 解析文件状态
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let (code, path) =
                    if line.starts_with("1 ") || line.starts_with("2 ") || line.starts_with("u ") {
                        // porcelain v2 ordinary/renamed/unmerged entries: XY status is in parts[1]
                        // e.g., "1 M. N... 100644 100644 100644 hash hash filename"
                        // parts[0]="1", parts[1]="M.", parts[last]="filename"
                        (parts[1], parts.last().unwrap())
                    } else {
                        // untracked "?" or ignored "!" entries
                        // e.g., "? untracked.txt"
                        // parts[0]="?", parts[1]="untracked.txt"
                        (parts[0], parts.last().unwrap())
                    };

                let entry = StatusEntry {
                    status: FileStatus::from_code(code.trim()),
                    path: path.to_string(),
                    old_path: None,
                };
                status.entries.push(entry);
            }
        }
    }

    // 获取 stash 数量
    let stash_output = Command::new("git")
        .args(["stash", "list"])
        .current_dir(repo_path)
        .output()
        .ok();

    if let Some(stash_out) = stash_output {
        let stash_stdout = String::from_utf8_lossy(&stash_out.stdout);
        status.stash_count = stash_stdout.lines().count();
    }

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_status_from_code() {
        // Untracked
        assert_eq!(FileStatus::from_code("??"), FileStatus::Untracked);
        assert_eq!(FileStatus::from_code("?"), FileStatus::Untracked);
        // Staged modified (v1: "M ", v2: "M.")
        assert_eq!(FileStatus::from_code("M "), FileStatus::Staged);
        assert_eq!(FileStatus::from_code("M."), FileStatus::Staged);
        // Unstaged modified (v1: " M", v2: ".M")
        assert_eq!(FileStatus::from_code(" M"), FileStatus::Modified);
        assert_eq!(FileStatus::from_code(".M"), FileStatus::Modified);
        // New file
        assert_eq!(FileStatus::from_code("A "), FileStatus::New);
        assert_eq!(FileStatus::from_code("A."), FileStatus::New);
        // Ignored
        assert_eq!(FileStatus::from_code("!!"), FileStatus::Ignored);
        assert_eq!(FileStatus::from_code("!"), FileStatus::Ignored);
    }

    #[test]
    fn test_git_status_default() {
        let status = GitStatus::default();
        assert!(status.entries.is_empty());
        assert_eq!(status.ahead, 0);
        assert_eq!(status.behind, 0);
    }

    #[test]
    fn test_git_status_has_changes() {
        let mut status = GitStatus::new();
        assert!(!status.has_changes());

        status.entries.push(StatusEntry {
            status: FileStatus::Modified,
            path: "test.rs".to_string(),
            old_path: None,
        });
        assert!(status.has_changes());
    }

    #[test]
    fn test_status_summary() {
        let mut status = GitStatus::new();
        status.branch = "main".to_string();
        status.ahead = 2;

        let summary = status.summary();
        assert!(summary.contains("main"));
        assert!(summary.contains("ahead 2"));
    }
}
