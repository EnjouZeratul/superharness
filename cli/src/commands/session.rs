//! continuum session 子命令
//!
//! 会话管理：列出、恢复、删除、查看会话。

use anyhow::Result;

use crate::cli::SessionCmd;
use sh_layer2::session_manager::ConcurrentSessionManager;
use sh_layer2::session_manager::SessionManagerTrait;
use std::sync::OnceLock;

/// 全局会话管理器
static SESSION_MANAGER: OnceLock<ConcurrentSessionManager> = OnceLock::new();

/// 获取会话管理器
fn get_session_manager() -> &'static ConcurrentSessionManager {
    SESSION_MANAGER.get_or_init(|| ConcurrentSessionManager::default_config())
}

/// 执行 session 子命令
pub fn execute(cmd: SessionCmd) -> Result<()> {
    // 检查是否在 tokio 运行时内
    let handle = tokio::runtime::Handle::try_current();

    match handle {
        Ok(handle) => {
            // 已在运行时内，使用 block_in_place
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let manager = get_session_manager();
                    match cmd {
                        SessionCmd::List { all } => list_sessions(manager, all).await,
                        SessionCmd::Resume { session_id } => {
                            resume_session(manager, &session_id).await
                        }
                        SessionCmd::Delete { session_id, force } => {
                            delete_session(manager, &session_id, force).await
                        }
                        SessionCmd::Show { session_id } => show_session(manager, &session_id).await,
                    }
                })
            })
        }
        Err(_) => {
            // 不在运行时内，创建新的
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let manager = get_session_manager();
                match cmd {
                    SessionCmd::List { all } => list_sessions(manager, all).await,
                    SessionCmd::Resume { session_id } => resume_session(manager, &session_id).await,
                    SessionCmd::Delete { session_id, force } => {
                        delete_session(manager, &session_id, force).await
                    }
                    SessionCmd::Show { session_id } => show_session(manager, &session_id).await,
                }
            })
        }
    }
}

/// 列出会话
async fn list_sessions(manager: &ConcurrentSessionManager, _all: bool) -> Result<()> {
    println!("Listing all sessions...\n");

    let sessions = manager.list().await?;

    if sessions.is_empty() {
        println!("  (no active sessions)");
        return Ok(());
    }

    println!("Total sessions: {}\n", sessions.len());

    for meta in sessions {
        let state = format!("{:?}", meta.state).to_lowercase();
        let created = meta.created_at.format("%Y-%m-%d %H:%M:%S");
        println!(
            "  {} [{}]\n    Agent: {} | Messages: {} | Created: {}",
            meta.session_id.0, state, meta.agent_id, meta.message_count, created
        );
    }

    Ok(())
}

/// 恢复会话
async fn resume_session(manager: &ConcurrentSessionManager, session_id: &str) -> Result<()> {
    use sh_layer2::types::SessionId;

    println!("Resuming session: {}", session_id);

    let id = SessionId::from(session_id);

    match manager.get(&id).await? {
        Some(session) => {
            println!("\nSession restored:");
            println!("  ID: {}", session.session_id.0);
            println!("  State: {:?}", session.state);
            println!("  Messages: {}", session.messages.len());
            println!("  Iterations: {}", session.iteration);

            if !session.messages.is_empty() {
                println!("\nRecent messages:");
                for msg in session.messages.iter().rev().take(5) {
                    let role = format!("{:?}", msg.role).to_lowercase();
                    let preview = if msg.content.len() > 50 {
                        format!("{}...", &msg.content[..50])
                    } else {
                        msg.content.clone()
                    };
                    println!("  [{}] {}", role, preview);
                }
            }
        }
        None => {
            println!("Session not found: {}", session_id);
            println!("\nUse 'continuum session list' to see available sessions.");
        }
    }

    Ok(())
}

/// 删除会话
async fn delete_session(
    manager: &ConcurrentSessionManager,
    session_id: &str,
    force: bool,
) -> Result<()> {
    use sh_layer2::types::SessionId;

    println!("Deleting session: {} (force: {})", session_id, force);

    let id = SessionId::from(session_id);

    // 先检查会话是否存在
    match manager.get(&id).await? {
        Some(session) => {
            if !force {
                println!("\nSession details:");
                println!("  ID: {}", session.session_id.0);
                println!("  Messages: {}", session.messages.len());
                println!("  State: {:?}", session.state);
                println!("\nUse --force to confirm deletion.");
                return Ok(());
            }

            match manager.delete(&id).await? {
                true => println!("Session deleted successfully."),
                false => println!("Failed to delete session."),
            }
        }
        None => {
            println!("Session not found: {}", session_id);
        }
    }

    Ok(())
}

/// 显示会话详情
async fn show_session(manager: &ConcurrentSessionManager, session_id: &str) -> Result<()> {
    use sh_layer2::types::SessionId;

    println!("Showing session details: {}", session_id);

    let id = SessionId::from(session_id);

    match manager.get(&id).await? {
        Some(session) => {
            println!("\n=== Session Details ===\n");
            println!("ID: {}", session.session_id.0);
            println!("Agent: {}", session.agent_id);
            println!("State: {:?}", session.state);
            println!(
                "Created: {}",
                session.created_at.format("%Y-%m-%d %H:%M:%S")
            );
            println!(
                "Last Updated: {}",
                session.last_updated.format("%Y-%m-%d %H:%M:%S")
            );
            println!("Iteration: {}", session.iteration);
            println!("Checkpoint Count: {}", session.checkpoint_count);
            println!("Messages: {}", session.messages.len());

            if !session.messages.is_empty() {
                println!("\n--- Message History ---\n");
                for (i, msg) in session.messages.iter().enumerate() {
                    let role = format!("{:?}", msg.role).to_lowercase();
                    println!("[{}] {}:", i + 1, role);
                    println!("{}\n", msg.content);
                }
            }

            // 直接从 Session 字段显示配置
            println!("\n--- Configuration ---\n");
            println!("Model: {}", session.model);
            println!("Temperature: {}", session.temperature);
            println!("Max Iterations: {}", session.max_iterations);
            if !session.system_prompt.is_empty() {
                let preview = &session.system_prompt[..50.min(session.system_prompt.len())];
                println!("System Prompt: {}...", preview);
            }
        }
        None => {
            println!("Session not found: {}", session_id);
            println!("\nUse 'continuum session list' to see available sessions.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_list() {
        let result = execute(SessionCmd::List { all: false });
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_show_nonexistent() {
        // 显示不存在的会话应该返回 Ok (打印 "not found") 而非 Err
        let result = execute(SessionCmd::Show {
            session_id: "nonexistent-session-id".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_delete_without_force() {
        // 不带 --force 删除应该成功但提示需要确认
        let result = execute(SessionCmd::Delete {
            session_id: "nonexistent-session-id".to_string(),
            force: false,
        });
        assert!(result.is_ok());
    }
}
