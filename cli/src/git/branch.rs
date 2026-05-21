//! Git branch 管理模块

use super::GitError;
use super::GitResult;
use std::path::Path;
use std::process::Command;

/// 分支信息
#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

/// 分支管理器
pub struct BranchManager {
    repo_path: std::path::PathBuf,
}

impl BranchManager {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            repo_path: repo_path.to_path_buf(),
        }
    }

    /// 列出所有分支
    pub fn list(&self, include_remote: bool) -> GitResult<Vec<Branch>> {
        let mut args = vec!["branch"];

        if include_remote {
            args.push("-a");
        }

        args.push("-vv");

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::NotARepository);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut branches = Vec::new();

        for line in stdout.lines() {
            let trimmed = line.trim_start();
            let is_current = trimmed.starts_with('*');
            let branch_str = if is_current {
                trimmed[1..].trim_start()
            } else {
                trimmed
            };

            // 跳过 HEAD detached 状态
            if branch_str.starts_with('(') {
                continue;
            }

            // 解析分支名
            let name = branch_str.split_whitespace().next().unwrap_or("");

            if name.is_empty() {
                continue;
            }

            let is_remote = name.starts_with("remotes/");

            // 解析 upstream 和 ahead/behind
            let upstream = None;
            let (ahead, behind) = self.parse_ahead_behind(line);

            branches.push(Branch {
                name: name.to_string(),
                is_current,
                is_remote,
                upstream,
                ahead,
                behind,
            });
        }

        Ok(branches)
    }

    /// 解析 ahead/behind
    fn parse_ahead_behind(&self, line: &str) -> (usize, usize) {
        let mut ahead = 0;
        let mut behind = 0;

        // 查找 [ahead N, behind M] 格式
        if let Some(start) = line.find("[ahead") {
            let bracket_end = line[start..].find(']').unwrap_or(line.len() - start) + start;
            let bracket_content = &line[start..bracket_end];

            for part in bracket_content.split(&['[', ']', ',', ' '][..]) {
                if part.starts_with("ahead") {
                    if let Some(n) = part.strip_prefix("ahead") {
                        ahead = n.parse().unwrap_or(0);
                    }
                } else if part.starts_with("behind") {
                    if let Some(n) = part.strip_prefix("behind") {
                        behind = n.parse().unwrap_or(0);
                    }
                }
            }
        }

        (ahead, behind)
    }

    /// 获取当前分支名
    pub fn current(&self) -> GitResult<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let current = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if current.is_empty() {
            // 可能处于 detached HEAD 状态
            let rev_output = Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .current_dir(&self.repo_path)
                .output()
                .map_err(|e| GitError::CommandFailed(e.to_string()))?;

            Ok(format!("HEAD detached at {}", String::from_utf8_lossy(&rev_output.stdout).trim()))
        } else {
            Ok(current)
        }
    }

    /// 创建新分支
    pub fn create(&self, name: &str) -> GitResult<()> {
        // 检查分支是否已存在
        let branches = self.list(false)?;
        if branches.iter().any(|b| b.name == name) {
            return Err(GitError::BranchExists(name.to_string()));
        }

        let output = Command::new("git")
            .args(["branch", name])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    /// 创建并切换到新分支
    pub fn create_and_switch(&self, name: &str) -> GitResult<()> {
        let branches = self.list(false)?;
        if branches.iter().any(|b| b.name == name) {
            return Err(GitError::BranchExists(name.to_string()));
        }

        let output = Command::new("git")
            .args(["checkout", "-b", name])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    /// 切换分支
    pub fn switch(&self, name: &str) -> GitResult<()> {
        let output = Command::new("git")
            .args(["checkout", name])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.contains("did not match any file") {
                return Err(GitError::BranchNotFound(name.to_string()));
            }
            return Err(GitError::CommandFailed(stderr));
        }

        Ok(())
    }

    /// 删除分支
    pub fn delete(&self, name: &str, force: bool) -> GitResult<()> {
        let current = self.current()?;
        if name == current {
            return Err(GitError::CommandFailed("Cannot delete current branch".to_string()));
        }

        let mut args = vec!["branch"];
        if force {
            args.push("-D");
        } else {
            args.push("-d");
        }
        args.push(name);

        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(())
    }

    /// 合并分支
    pub fn merge(&self, name: &str) -> GitResult<()> {
        let output = Command::new("git")
            .args(["merge", name])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.contains("CONFLICT") {
                return Err(GitError::MergeConflict);
            }
            return Err(GitError::CommandFailed(stderr));
        }

        Ok(())
    }

    /// 渲染分支列表
    pub fn render_list(&self, include_remote: bool) -> GitResult<String> {
        let branches = self.list(include_remote)?;

        let mut output = String::new();

        for branch in branches {
            let prefix = if branch.is_current { "* " } else { "  " };
            let suffix = if branch.ahead > 0 || branch.behind > 0 {
                format!(" (ahead {}, behind {})", branch.ahead, branch.behind)
            } else {
                String::new()
            };

            output.push_str(&format!("{}{}{}\n", prefix, branch.name, suffix));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_display() {
        let branch = Branch {
            name: "main".to_string(),
            is_current: true,
            is_remote: false,
            upstream: Some("origin/main".to_string()),
            ahead: 2,
            behind: 0,
        };

        assert!(branch.is_current);
        assert!(!branch.is_remote);
    }
}
