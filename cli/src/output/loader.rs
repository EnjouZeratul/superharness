//! 输出加载器

use std::path::Path;

/// 输出加载器
pub struct OutputLoader;

impl OutputLoader {
    /// 从文件加载输出
    pub fn from_file(path: &Path) -> anyhow::Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to load output from {:?}: {}", path, e))
    }

    /// 从字符串加载输出
    pub fn from_string(content: &str) -> String {
        content.to_string()
    }
}
