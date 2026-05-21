//! Git 集成模块

pub mod commands;
pub mod diff;
pub mod status;
pub mod commit;
pub mod branch;
pub mod pr;

pub use commands::GitCommands;
pub use status::GitStatus;
pub use diff::GitDiff;
pub use commit::CommitGenerator;
pub use branch::BranchManager;
pub use pr::PrCreator;

/// Git 错误类型
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Not a git repository")]
    NotARepository,

    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("No changes to commit")]
    NoChanges,

    #[error("Branch already exists: {0}")]
    BranchExists(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Merge conflict detected")]
    MergeConflict,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Git 结果类型
pub type GitResult<T> = std::result::Result<T, GitError>;
