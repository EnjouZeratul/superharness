//! # Continuum CLI
//!
//! 终端 Agent 产品入口点。

// 允许未使用的代码（测试和未来功能）
#![allow(dead_code)]

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod agent;
mod cli;
mod commands;
mod config;
mod git;
mod integration;
mod output;
mod tui;

use cli::{CliApp, CliArgs, CliConfig};

fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let args = CliArgs::parse_args();

    // 初始化日志
    if args.debug {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .with_level(false),
            )
            .init();
    }

    // 构建配置
    let config = CliConfig::new()
        .with_debug(args.debug)
        .with_config_path(args.config.unwrap_or_default());

    // 创建并运行应用
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut app = CliApp::new(args.command, config);
        app.run().await
    })
}
