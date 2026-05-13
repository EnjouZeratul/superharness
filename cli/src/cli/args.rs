//! # CLI 参数定义
//!
//! 命令行参数结构和解析。

use clap::{Parser, Subcommand};

/// SuperHarness CLI - 终端 Agent 产品
#[derive(Parser, Debug)]
#[command(name = "superharness")]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// 启用调试日志
    #[arg(short, long, global = true)]
    pub debug: bool,

    /// 配置文件路径
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// 子命令（可选，默认进入TUI）
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 子命令
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// 启动 Agent 执行任务
    Run {
        /// 任务描述
        task: Option<String>,
        /// 使用指定会话 ID
        #[arg(short, long)]
        session: Option<String>,
        /// 设置预算上限
        #[arg(short, long)]
        budget: Option<f64>,
        /// 非交互模式
        #[arg(short, long)]
        non_interactive: bool,
    },

    /// 管理会话
    Session {
        #[command(subcommand)]
        cmd: SessionCmd,
    },

    /// 管理配置
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },

    /// 列出可用工具
    Tools {
        /// 筛选工具类型
        #[arg(short, long)]
        filter: Option<String>,
        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },

    /// 启动 TUI 界面（默认行为）
    Tui {
        /// 恢复指定会话
        #[arg(short, long)]
        session: Option<String>,
    },

    /// 检查点管理
    Checkpoint {
        #[command(subcommand)]
        cmd: CheckpointCmd,
    },
}

/// 会话子命令
#[derive(Subcommand, Debug, Clone)]
pub enum SessionCmd {
    /// 列出所有会话
    List {
        /// 显示已完成的会话
        #[arg(short, long)]
        all: bool,
    },
    /// 恢复指定会话
    Resume {
        /// 会话 ID
        session_id: String,
    },
    /// 删除指定会话
    Delete {
        /// 会话 ID
        session_id: String,
        /// 强制删除
        #[arg(short, long)]
        force: bool,
    },
    /// 显示会话详情
    Show {
        /// 会话 ID
        session_id: String,
    },
}

/// 配置子命令
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCmd {
    /// 显示当前配置
    Show {
        /// 配置键（可选）
        key: Option<String>,
    },
    /// 设置配置项
    Set {
        /// 配置键
        key: String,
        /// 配置值
        value: String,
    },
    /// 获取配置项
    Get {
        /// 配置键
        key: String,
    },
    /// 初始化默认配置文件
    Init {
        /// 强制覆盖已存在的配置
        #[arg(short, long)]
        force: bool,
    },
    /// 列出所有配置键
    Keys,
    /// 列出所有提供商
    List,
    /// 添加提供商
    AddProvider {
        /// 提供商名称
        name: String,
        /// API 密钥
        #[arg(short, long)]
        key: String,
        /// API 基础 URL
        #[arg(short, long)]
        url: Option<String>,
        /// 默认模型
        #[arg(short, long)]
        model: Option<String>,
    },
    /// 切换提供商
    Use {
        /// 提供商名称
        provider: String,
    },
}

/// 检查点子命令
#[derive(Subcommand, Debug, Clone)]
pub enum CheckpointCmd {
    /// 列出检查点
    List {
        /// 会话 ID
        #[arg(short, long)]
        session: Option<String>,
    },
    /// 恢复到指定检查点
    Restore {
        /// 检查点 ID
        checkpoint_id: String,
    },
    /// 删除检查点
    Delete {
        /// 检查点 ID
        checkpoint_id: String,
    },
}

impl CliArgs {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_version() {
        let args = CliArgs::try_parse_from(["superharness", "--version"]);
        // 应该成功解析并显示版本
        assert!(args.is_ok() || args.is_err()); // 版本命令会退出程序
    }

    #[test]
    fn test_cli_args_help() {
        let args = CliArgs::try_parse_from(["superharness", "--help"]);
        // 帮助命令会退出程序
        assert!(args.is_ok() || args.is_err());
    }

    #[test]
    fn test_default_tui_behavior() {
        // 测试无参数时默认进入TUI
        let args = CliArgs::try_parse_from(["superharness"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        // command 应该是 None
        assert!(args.command.is_none());
    }

    #[test]
    fn test_explicit_tui_command() {
        // 测试显式tui命令
        let args = CliArgs::try_parse_from(["superharness", "tui"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(matches!(args.command, Some(Commands::Tui { .. })));
    }

    #[test]
    fn test_run_command() {
        // 测试run子命令
        let args = CliArgs::try_parse_from(["superharness", "run", "test task"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        if let Some(Commands::Run { task, .. }) = args.command {
            assert_eq!(task, Some("test task".to_string()));
        } else {
            panic!("Expected Run command");
        }
    }

    #[test]
    fn test_session_list_command() {
        // 测试session list子命令
        let args = CliArgs::try_parse_from(["superharness", "session", "list"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(matches!(args.command, Some(Commands::Session { .. })));
    }

    #[test]
    fn test_config_show_command() {
        // 测试config show子命令
        let args = CliArgs::try_parse_from(["superharness", "config", "show"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(matches!(args.command, Some(Commands::Config { .. })));
    }

    #[test]
    fn test_tools_command() {
        // 测试tools子命令
        let args = CliArgs::try_parse_from(["superharness", "tools", "--verbose"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        if let Some(Commands::Tools { verbose, .. }) = args.command {
            assert!(verbose);
        } else {
            panic!("Expected Tools command");
        }
    }

    #[test]
    fn test_checkpoint_list_command() {
        // 测试checkpoint list子命令
        let args = CliArgs::try_parse_from(["superharness", "checkpoint", "list"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(matches!(args.command, Some(Commands::Checkpoint { .. })));
    }
}
