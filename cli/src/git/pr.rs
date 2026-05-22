//! Git PR (Pull Request) 模块

use super::GitError;
use super::GitResult;
use std::path::Path;
use std::process::Command;

/// PR 信息
#[derive(Debug, Clone)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: PrState,
    pub author: String,
    pub base_branch: String,
    pub head_branch: String,
    pub url: String,
    pub draft: bool,
}

/// PR 状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

impl std::fmt::Display for PrState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrState::Open => write!(f, "OPEN"),
            PrState::Closed => write!(f, "CLOSED"),
            PrState::Merged => write!(f, "MERGED"),
        }
    }
}

/// PR 创建器
pub struct PrCreator {
    repo_path: std::path::PathBuf,
}

impl PrCreator {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            repo_path: repo_path.to_path_buf(),
        }
    }

    /// 检测远程仓库类型
    pub fn detect_remote_type(&self) -> GitResult<RemoteType> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if url.contains("github.com") {
            Ok(RemoteType::GitHub)
        } else if url.contains("gitlab.com") {
            Ok(RemoteType::GitLab)
        } else if url.contains("bitbucket.org") {
            Ok(RemoteType::Bitbucket)
        } else {
            Ok(RemoteType::Unknown)
        }
    }

    /// 获取仓库的 owner/repo
    pub fn get_repo_slug(&self) -> GitResult<String> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // 解析 GitHub URL
        // https://github.com/owner/repo.git
        // git@github.com:owner/repo.git
        let slug = if url.starts_with("https://") || url.starts_with("http://") {
            url.trim_end_matches(".git")
                .split('/')
                .skip(3) // skip https://github.com
                .collect::<Vec<_>>()
                .join("/")
        } else if url.starts_with("git@") {
            url.trim_end_matches(".git")
                .split(':')
                .nth(1)
                .unwrap_or("")
                .to_string()
        } else {
            url
        };

        Ok(slug)
    }

    /// 创建 PR
    pub fn create(
        &self,
        title: &str,
        body: &str,
        base: &str,
        head: &str,
        draft: bool,
    ) -> GitResult<PullRequest> {
        let remote_type = self.detect_remote_type()?;

        match remote_type {
            RemoteType::GitHub => self.create_github_pr(title, body, base, head, draft),
            RemoteType::GitLab => self.create_gitlab_mr(title, body, base, head),
            _ => Err(GitError::CommandFailed(
                "Unsupported remote repository type".to_string(),
            )),
        }
    }

    /// 通过 GitHub CLI 创建 PR
    fn create_github_pr(
        &self,
        title: &str,
        body: &str,
        base: &str,
        head: &str,
        draft: bool,
    ) -> GitResult<PullRequest> {
        let mut args = vec![
            "pr", "create", "--title", title, "--body", body, "--base", base, "--head", head,
        ];

        if draft {
            args.push("--draft");
        }

        let output = Command::new("gh")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(format!("gh CLI not found: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        // 解析 PR URL
        let url = stdout.trim().to_string();
        let number = self.extract_pr_number(&url).unwrap_or(0);

        Ok(PullRequest {
            number,
            title: title.to_string(),
            state: PrState::Open,
            author: String::new(),
            base_branch: base.to_string(),
            head_branch: head.to_string(),
            url,
            draft,
        })
    }

    /// 通过 GitLab CLI 创建 MR
    fn create_gitlab_mr(
        &self,
        title: &str,
        body: &str,
        base: &str,
        head: &str,
    ) -> GitResult<PullRequest> {
        let args = vec![
            "mr",
            "create",
            "--title",
            title,
            "--description",
            body,
            "--target-branch",
            base,
            "--source-branch",
            head,
        ];

        let output = Command::new("glab")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(format!("glab CLI not found: {}", e)))?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let url = stdout.trim().to_string();

        Ok(PullRequest {
            number: 0,
            title: title.to_string(),
            state: PrState::Open,
            author: String::new(),
            base_branch: base.to_string(),
            head_branch: head.to_string(),
            url,
            draft: false,
        })
    }

    /// 列出 PR
    pub fn list(&self, state: PrState) -> GitResult<Vec<PullRequest>> {
        let state_arg = match state {
            PrState::Open => "open",
            PrState::Closed => "closed",
            PrState::Merged => "merged",
        };

        let output = Command::new("gh")
            .args([
                "pr",
                "list",
                "--state",
                state_arg,
                "--json",
                "number,title,state,author,baseRefName,headRefName,url,isDraft",
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // 解析 JSON 输出
        let stdout = String::from_utf8_lossy(&output.stdout);
        let prs = self.parse_pr_list(&stdout);

        Ok(prs)
    }

    /// 解析 PR 列表
    fn parse_pr_list(&self, json: &str) -> Vec<PullRequest> {
        let mut prs = Vec::new();

        // 简单的 JSON 解析（避免引入 serde_json 依赖给 git 模块）
        // 格式: [{"number":1,"title":"...","state":"OPEN",...},...]
        if let Ok(values) = serde_json::from_str::<serde_json::Value>(json) {
            if let Some(arr) = values.as_array() {
                for item in arr {
                    let pr = PullRequest {
                        number: item.get("number").and_then(|v| v.as_u64()).unwrap_or(0),
                        title: item
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        state: match item.get("state").and_then(|v| v.as_str()).unwrap_or("") {
                            "OPEN" => PrState::Open,
                            "CLOSED" => PrState::Closed,
                            "MERGED" => PrState::Merged,
                            _ => PrState::Open,
                        },
                        author: item
                            .get("author")
                            .and_then(|v| v.get("login"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        base_branch: item
                            .get("baseRefName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        head_branch: item
                            .get("headRefName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        url: item
                            .get("url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        draft: item
                            .get("isDraft")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    };
                    prs.push(pr);
                }
            }
        }

        prs
    }

    /// 从 URL 提取 PR 编号
    fn extract_pr_number(&self, url: &str) -> Option<u64> {
        // https://github.com/owner/repo/pull/123
        url.split('/').last()?.parse().ok()
    }

    /// 生成 PR body（基于 commit 历史）
    pub fn generate_body(&self, base_branch: &str) -> GitResult<String> {
        let output = Command::new("git")
            .args(["log", &format!("{}..HEAD", base_branch), "--oneline"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        let commits = String::from_utf8_lossy(&output.stdout).to_string();

        if commits.is_empty() {
            return Ok("No commits found between base and HEAD.".to_string());
        }

        let mut body = String::from("## Summary\n\n");

        for line in commits.lines() {
            if !line.is_empty() {
                body.push_str(&format!("- {}\n", line));
            }
        }

        Ok(body)
    }
}

/// 远程仓库类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteType {
    GitHub,
    GitLab,
    Bitbucket,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_state_display() {
        assert_eq!(PrState::Open.to_string(), "OPEN");
        assert_eq!(PrState::Closed.to_string(), "CLOSED");
        assert_eq!(PrState::Merged.to_string(), "MERGED");
    }

    #[test]
    fn test_extract_pr_number() {
        let creator = PrCreator::new(Path::new("."));
        let num = creator.extract_pr_number("https://github.com/owner/repo/pull/123");
        assert_eq!(num, Some(123));
    }

    #[test]
    fn test_pr_creation() {
        let pr = PullRequest {
            number: 42,
            title: "Test PR".to_string(),
            state: PrState::Open,
            author: "testuser".to_string(),
            base_branch: "main".to_string(),
            head_branch: "feature".to_string(),
            url: "https://github.com/owner/repo/pull/42".to_string(),
            draft: false,
        };

        assert_eq!(pr.number, 42);
        assert!(!pr.draft);
    }
}
