//! continuum tools 子命令
//!
//! 列出和管理可用工具。

use anyhow::Result;
use sh_layer3::tool_executor::DefaultToolExecutor;
use sh_layer3::ToolExecutor;

/// 执行 tools 子命令
pub fn execute(filter: Option<String>, verbose: bool) -> Result<()> {
    if verbose {
        println!("Available tools (verbose):\n");
    } else {
        println!("Available tools:\n");
    }

    // 使用 DefaultToolExecutor 获取工具列表
    let executor = DefaultToolExecutor::new();
    let tools = executor.list_tools();

    if tools.is_empty() {
        println!("  (no tools registered)");
        return Ok(());
    }

    // 按 category 分组
    let mut categories: std::collections::HashMap<String, Vec<_>> =
        std::collections::HashMap::new();

    for meta in tools {
        let category = format!("{:?}", meta.category).to_lowercase();
        categories
            .entry(category)
            .or_default()
            .push((meta.name, meta.description));
    }

    // 按类别输出
    for (category, mut tools_in_category) in categories {
        tools_in_category.sort_by(|a, b| a.0.cmp(&b.0));

        // 应用过滤
        let filtered: Vec<_> = tools_in_category
            .iter()
            .filter(|(name, _)| {
                if let Some(ref f) = filter {
                    name.contains(f) || category.contains(f)
                } else {
                    true
                }
            })
            .collect();

        if filtered.is_empty() {
            continue;
        }

        if verbose {
            println!("[{}]", category);
            for (name, desc) in filtered {
                println!("  {} - {}", name, desc);
            }
            println!();
        } else {
            for (name, desc) in filtered {
                println!("  {} - {}", name, desc);
            }
        }
    }

    // 显示统计
    if verbose {
        println!("---");
        println!("Total: {} tools registered", executor.list_tools().len());
    }

    Ok(())
}

/// 检查工具是否可用
pub fn is_tool_available(name: &str) -> bool {
    let executor = DefaultToolExecutor::new();
    executor.is_available(name)
}

/// 获取工具详情
pub fn get_tool_info(name: &str) -> Option<(String, String)> {
    let executor = DefaultToolExecutor::new();
    executor.get_meta(name).map(|m| (m.name, m.description))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools() {
        let result = execute(None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tools_verbose() {
        let result = execute(None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tools_with_filter() {
        let result = execute(Some("file".to_string()), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_tool_available() {
        // 测试工具可用性检查
        let available = is_tool_available("bash");
        // bash 工具应该存在
        assert!(available);
    }
}
