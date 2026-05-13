//! 输出流处理

use anyhow::Result;

/// 流式输出处理器
pub struct StreamingOutput;

impl StreamingOutput {
    /// 创建新的流式输出处理器
    pub fn new() -> Self {
        Self
    }

    /// 输出文本
    pub fn output(&self, text: &str) -> Result<()> {
        print!("{}", text);
        Ok(())
    }
}

impl Default for StreamingOutput {
    fn default() -> Self {
        Self::new()
    }
}
