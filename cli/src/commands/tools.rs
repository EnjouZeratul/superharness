//! superharness tools 子命令

use anyhow::Result;

pub fn execute(filter: Option<String>, verbose: bool) -> Result<()> {
    if verbose {
        println!("Available tools (verbose):");
    } else {
        println!("Available tools:");
    }

    // TODO: 从 ToolRegistry 获取工具列表
    let tools = vec![
        ("file_read", "Read file contents", "file"),
        ("file_write", "Write to file", "file"),
        ("bash", "Execute shell commands", "execution"),
        ("glob", "Find files by pattern", "search"),
        ("grep", "Search file contents", "search"),
        ("lsp", "Language server operations", "lsp"),
    ];

    for (name, desc, category) in tools {
        if let Some(ref f) = filter {
            if !name.contains(f) && !category.contains(f) {
                continue;
            }
        }
        if verbose {
            println!("  {} [{}] - {}", name, category, desc);
        } else {
            println!("  {} - {}", name, desc);
        }
    }

    Ok(())
}
