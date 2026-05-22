//! Git 集成模块

pub mod branch;
pub mod commands;
pub mod commit;
pub mod diff;
pub mod pr;
pub mod status;

pub use branch::BranchManager;
pub use commands::GitCommands;
pub use commit::CommitGenerator;
pub use diff::GitDiff;
pub use pr::PrCreator;
pub use status::GitStatus;

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
