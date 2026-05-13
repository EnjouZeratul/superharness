//! superharness session 子命令

use anyhow::Result;

use crate::cli::SessionCmd;

pub fn execute(cmd: SessionCmd) -> Result<()> {
    match cmd {
        SessionCmd::List { all } => {
            println!("Listing all sessions...");
            if all {
                println!("  (including completed sessions)");
            }
            // TODO: 调用 SessionManager
        }
        SessionCmd::Resume { session_id } => {
            println!("Resuming session: {}", session_id);
            // TODO: 恢复会话
        }
        SessionCmd::Delete { session_id, force } => {
            println!("Deleting session: {} (force: {})", session_id, force);
            // TODO: 删除会话
        }
        SessionCmd::Show { session_id } => {
            println!("Showing session details: {}", session_id);
            // TODO: 显示会话详情
        }
    }

    Ok(())
}
