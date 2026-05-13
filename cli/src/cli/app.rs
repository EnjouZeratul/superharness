//! # CLI Application
//!
//! CLI 应用状态和生命周期管理。

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use super::args::Commands;
use crate::integration::audit::AuditService;
use crate::integration::mcp::McpService;

/// CLI 应用配置
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// 配置文件路径
    pub config_path: Option<PathBuf>,
    /// 是否启用调试模式
    pub debug: bool,
    /// 工作目录
    pub working_dir: PathBuf,
    /// 是否启用审计
    pub enable_audit: bool,
    /// 是否启用 MCP
    pub enable_mcp: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            config_path: None,
            debug: false,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            enable_audit: true,
            enable_mcp: false,
        }
    }
}

impl CliConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置配置文件路径
    pub fn with_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// 启用调试模式
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// 设置工作目录
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = dir.into();
        self
    }

    /// 设置审计开关
    pub fn with_audit(mut self, enable: bool) -> Self {
        self.enable_audit = enable;
        self
    }

    /// 设置 MCP 开关
    pub fn with_mcp(mut self, enable: bool) -> Self {
        self.enable_mcp = enable;
        self
    }
}

/// CLI 应用状态
pub struct CliApp {
    /// 应用配置
    config: CliConfig,
    /// 当前命令
    command: Commands,
    /// 是否正在运行
    running: bool,
    /// 会话状态（用于持久化）
    session_state: Arc<RwLock<SessionState>>,
    /// 审计服务
    audit_service: Option<Arc<AuditService>>,
    /// MCP 服务
    mcp_service: Option<Arc<McpService>>,
}

/// 会话状态
#[derive(Debug, Default, Clone)]
pub struct SessionState {
    /// 当前会话 ID
    pub current_session_id: Option<String>,
    /// 用户 ID
    pub user_id: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            current_session_id: None,
            user_id: whoami::username(),
            created_at: chrono::Utc::now(),
        }
    }
}

impl CliApp {
    /// 创建新的 CLI 应用
    ///
    /// 如果 command 为 None，默认启动 TUI
    pub fn new(command: Option<Commands>, config: CliConfig) -> Self {
        // 处理默认行为：无参数时进入TUI
        let command = command.unwrap_or(Commands::Tui { session: None });

        let audit_service = if config.enable_audit {
            Some(Arc::new(AuditService::new()))
        } else {
            None
        };

        let mcp_service = if config.enable_mcp {
            Some(Arc::new(McpService::new()))
        } else {
            None
        };

        Self {
            config,
            command,
            running: true,
            session_state: Arc::new(RwLock::new(SessionState::new())),
            audit_service,
            mcp_service,
        }
    }

    /// 运行应用
    pub async fn run(&mut self) -> Result<()> {
        if self.config.debug {
            tracing::debug!("Debug mode enabled");
        }

        // 记录启动审计
        if let Some(ref audit) = self.audit_service {
            let state = self.session_state.read().await;
            use sh_core::layer4::AuditAction;
            audit
                .log_success(&state.user_id, AuditAction::Login, "cli", None)
                .await?;
        }

        // 分发到对应的子命令处理器
        let result = match &self.command {
            Commands::Run {
                task,
                session,
                budget,
                non_interactive,
            } => {
                self.run_command(task.clone(), session.clone(), *budget, *non_interactive)
                    .await
            }
            Commands::Session { cmd } => self.session_command(cmd.clone()).await,
            Commands::Config { cmd } => self.config_command(cmd.clone()).await,
            Commands::Tools { filter, verbose } => {
                self.tools_command(filter.clone(), *verbose).await
            }
            Commands::Tui { session } => self.tui_command(session.clone()).await,
            Commands::Checkpoint { cmd } => self.checkpoint_command(cmd.clone()).await,
        };

        // 记录退出审计
        if let Some(ref audit) = self.audit_service {
            let state = self.session_state.read().await;
            use sh_core::layer4::AuditAction;
            audit
                .log_success(&state.user_id, AuditAction::Logout, "cli", None)
                .await?;
        }

        self.running = false;
        result
    }

    /// 执行 run 子命令
    async fn run_command(
        &self,
        task: Option<String>,
        session: Option<String>,
        budget: Option<f64>,
        non_interactive: bool,
    ) -> Result<()> {
        use crate::commands::run;

        // 更新会话状态
        if let Some(ref session_id) = session {
            let mut state = self.session_state.write().await;
            state.current_session_id = Some(session_id.clone());
        }

        // 记录执行审计
        if let Some(ref audit) = self.audit_service {
            let state = self.session_state.read().await;
            use sh_core::layer4::AuditAction;
            audit
                .log_success(
                    &state.user_id,
                    AuditAction::Execute,
                    "task",
                    task.as_deref(),
                )
                .await?;
        }

        run::execute(
            task,
            self.config
                .config_path
                .as_ref()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string()),
            budget,
            self.config.debug,
            non_interactive,
        )
    }

    /// 执行 session 子命令
    async fn session_command(&self, cmd: super::args::SessionCmd) -> Result<()> {
        use crate::commands::session;
        session::execute(cmd)
    }

    /// 执行 config 子命令
    async fn config_command(&self, cmd: super::args::ConfigCmd) -> Result<()> {
        use crate::commands::config;
        config::execute(cmd)
    }

    /// 执行 tools 子命令
    async fn tools_command(&self, filter: Option<String>, verbose: bool) -> Result<()> {
        use crate::commands::tools;

        // 如果启用 MCP，显示 MCP 工具
        if let Some(ref mcp) = self.mcp_service {
            let mcp_tools = mcp.list_tools().await?;
            println!("MCP Tools:");
            for tool in mcp_tools {
                if verbose {
                    println!("  {} [mcp] - {}", tool.name, tool.description);
                } else {
                    println!("  {} - {}", tool.name, tool.description);
                }
            }
        }

        tools::execute(filter, verbose)
    }

    /// 执行 tui 子命令
    async fn tui_command(&self, session: Option<String>) -> Result<()> {
        use crate::tui;
        tui::run_with_session(session)
    }

    /// 执行 checkpoint 子命令
    async fn checkpoint_command(&self, cmd: super::args::CheckpointCmd) -> Result<()> {
        use crate::commands;
        // checkpoint 命令在 commands 模块中实现
        match cmd {
            super::args::CheckpointCmd::List { session } => {
                println!("Listing checkpoints for session: {:?}", session);
                // TODO: 调用 CheckpointSystem
            }
            super::args::CheckpointCmd::Restore { checkpoint_id } => {
                println!("Restoring checkpoint: {}", checkpoint_id);
                // TODO: 调用 CheckpointSystem
            }
            super::args::CheckpointCmd::Delete { checkpoint_id } => {
                println!("Deleting checkpoint: {}", checkpoint_id);
                // TODO: 调用 CheckpointSystem
            }
        }
        Ok(())
    }

    /// 检查应用是否正在运行
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// 获取配置
    pub fn config(&self) -> &CliConfig {
        &self.config
    }

    /// 获取会话状态
    pub async fn session_state(&self) -> SessionState {
        self.session_state.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_config_default() {
        let config = CliConfig::default();
        assert!(!config.debug);
        assert!(config.config_path.is_none());
    }

    #[test]
    fn test_cli_config_builder() {
        let config = CliConfig::new()
            .with_debug(true)
            .with_config_path("/tmp/config.toml");

        assert!(config.debug);
        assert!(config.config_path.is_some());
    }

    #[test]
    fn test_session_state_creation() {
        let state = SessionState::new();
        assert!(state.current_session_id.is_none());
        assert!(!state.user_id.is_empty());
    }
}
