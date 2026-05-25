//! Git 命令整合

use super::branch::BranchManager;
use super::commit::CommitGenerator;
use super::diff::DiffType;
use super::pr::PrCreator;
use super::status::get_status;
use super::{GitError, GitResult};
use std::path::Path;

/// Git 命令执行器
pub struct GitCommands {
    repo_path: std::path::PathBuf,
}

impl GitCommands {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            repo_path: repo_path.to_path_buf(),
        }
    }

    /// 从当前目录创建
    pub fn from_cwd() -> GitResult<Self> {
        let cwd = std::env::current_dir().map_err(|e| GitError::CommandFailed(e.to_string()))?;
        Ok(Self::new(&cwd))
    }

    /// 获取 Git 状态
    pub fn status(&self) -> GitResult<String> {
        let status = get_status(&self.repo_path)?;
        Ok(status.render())
    }

    /// 获取简洁状态
    pub fn status_short(&self) -> GitResult<String> {
        let status = get_status(&self.repo_path)?;
        Ok(status.summary())
    }

    /// 获取 Diff
    pub fn diff(&self, staged: bool, paths: &[&str]) -> GitResult<String> {
        let diff_type = if staged {
            DiffType::Staged
        } else {
            DiffType::Working
        };
        let diff = super::diff::get_diff(&self.repo_path, diff_type, paths)?;
        Ok(diff.render())
    }

    /// 获取 Diff 统计
    pub fn diff_stat(&self) -> GitResult<String> {
        super::diff::get_diff_stat(&self.repo_path)
    }

    /// 生成 commit 消息
    pub fn generate_commit_message(&self) -> GitResult<String> {
        let status = get_status(&self.repo_path)?;
        let diff = super::diff::get_diff(&self.repo_path, DiffType::Staged, &[])?;

        let generator = CommitGenerator::new();
        Ok(generator.generate(&diff, &status))
    }

    /// 执行 commit
    pub fn commit(&self, message: Option<&str>, amend: bool) -> GitResult<String> {
        let msg = if let Some(m) = message {
            m.to_string()
        } else {
            self.generate_commit_message()?
        };

        super::commit::commit(&self.repo_path, &msg, amend)
    }

    /// 暂存文件
    pub fn add(&self, paths: &[&str]) -> GitResult<()> {
        if paths.is_empty() || paths.contains(&".") {
            super::commit::add_all(&self.repo_path)
        } else {
            super::commit::add(&self.repo_path, paths)
        }
    }

    /// 获取分支列表
    pub fn branch_list(&self, include_remote: bool) -> GitResult<String> {
        let manager = BranchManager::new(&self.repo_path);
        manager.render_list(include_remote)
    }

    /// 获取当前分支
    pub fn branch_current(&self) -> GitResult<String> {
        let manager = BranchManager::new(&self.repo_path);
        manager.current()
    }

    /// 创建分支
    pub fn branch_create(&self, name: &str, switch: bool) -> GitResult<()> {
        let manager = BranchManager::new(&self.repo_path);
        if switch {
            manager.create_and_switch(name)
        } else {
            manager.create(name)
        }
    }

    /// 切换分支
    pub fn branch_switch(&self, name: &str) -> GitResult<()> {
        let manager = BranchManager::new(&self.repo_path);
        manager.switch(name)
    }

    /// 删除分支
    pub fn branch_delete(&self, name: &str, force: bool) -> GitResult<()> {
        let manager = BranchManager::new(&self.repo_path);
        manager.delete(name, force)
    }

    /// 创建 PR
    pub fn pr_create(
        &self,
        title: &str,
        body: Option<&str>,
        base: &str,
        draft: bool,
    ) -> GitResult<String> {
        let creator = PrCreator::new(&self.repo_path);

        // 获取当前分支名
        let head = self.branch_current()?;

        // 生成 body
        let body = if let Some(b) = body {
            b.to_string()
        } else {
            creator.generate_body(base)?
        };

        let pr = creator.create(title, &body, base, &head, draft)?;

        Ok(format!(
            "Created {} PR #{}: {}\n{}",
            if pr.draft { "draft" } else { "" },
            pr.number,
            pr.title,
            pr.url
        ))
    }

    /// 列出 PR
    pub fn pr_list(&self, state: super::pr::PrState) -> GitResult<Vec<super::pr::PullRequest>> {
        let creator = PrCreator::new(&self.repo_path);
        creator.list(state)
    }

    /// 检查是否为 Git 仓库
    pub fn is_repo(&self) -> bool {
        std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&self.repo_path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_commands_creation() {
        // 验证 GitCommands 可以在非 Git 目录创建
        let temp = tempfile::tempdir().unwrap();
        let cmds = GitCommands::new(temp.path());
        // 非 Git 目录应该识别为非仓库
        assert!(!cmds.is_repo());
    }

    #[test]
    fn test_is_repo() {
        // 在非 Git 目录测试
        let temp = tempfile::tempdir().unwrap();
        let cmds = GitCommands::new(temp.path());
        // 临时目录不是 Git 仓库
        assert!(!cmds.is_repo());
    }
}
