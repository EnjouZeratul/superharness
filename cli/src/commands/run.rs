//! continuum run 子命令

use anyhow::Result;

pub fn execute(
    task: Option<String>,
    _config: Option<String>,
    _budget: Option<f64>,
    debug: bool,
    non_interactive: bool,
) -> Result<()> {
    if debug {
        println!("Debug mode enabled");
    }

    if non_interactive {
        println!("Non-interactive mode");
    }

    match task {
        Some(t) => {
            println!("Running task: {}", t);
            // TODO: 调用 Agent Runtime
        }
        None => {
            println!("Starting interactive mode...");
            // TODO: 启动交互模式
        }
    }

    Ok(())
}
