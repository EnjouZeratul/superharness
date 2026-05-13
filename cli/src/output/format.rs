//! 输出格式化

use anyhow::Result;

/// 输出格式器
pub struct OutputFormatter {
    format: OutputFormat,
}

/// 输出格式类型
#[derive(Debug, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Plain,
    Json,
    Table,
}

impl OutputFormatter {
    /// 创建新的输出格式器
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// 格式化输出
    pub fn format(&self, data: &str) -> Result<String> {
        match self.format {
            OutputFormat::Plain => Ok(data.to_string()),
            OutputFormat::Json => Ok(serde_json::to_string_pretty(&data)?),
            OutputFormat::Table => Ok(data.to_string()),
        }
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new(OutputFormat::default())
    }
}
