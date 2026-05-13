//! # Output Parsers
//!
//! 输出解析器：解析 LLM 输出为结构化数据。

use crate::types::Layer3Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 输出解析器 trait
///
/// 定义 LLM 输出解析接口。
#[async_trait]
pub trait OutputParser: Send + Sync {
    /// 解析器名称
    fn name(&self) -> &str;

    /// 解析 LLM 输出
    async fn parse(&self, output: &str) -> Layer3Result<ParsedOutput>;

    /// 获取解析指令（用于提示词）
    fn get_format_instructions(&self) -> String;
}

/// 解析后的输出
#[derive(Debug, Clone)]
pub struct ParsedOutput {
    /// 解析结果（JSON）
    pub data: serde_json::Value,
    /// 原始输出
    pub raw: String,
    /// 是否成功
    pub success: bool,
    /// 解析错误（如果有）
    pub error: Option<String>,
}

/// JSON 解析器
pub struct JsonParser {
    /// 是否严格模式
    strict: bool,
}

impl JsonParser {
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new(false)
    }
}

#[async_trait]
impl OutputParser for JsonParser {
    fn name(&self) -> &str {
        "json"
    }

    async fn parse(&self, output: &str) -> Layer3Result<ParsedOutput> {
        // 尝试提取 JSON
        let trimmed = output.trim();

        // 尝试直接解析
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return Ok(ParsedOutput {
                data,
                raw: output.to_string(),
                success: true,
                error: None,
            });
        }

        // 尝试从文本中提取 JSON 块
        let json_start = trimmed.find('{').or_else(|| trimmed.find('['));
        let json_end = trimmed.rfind('}').or_else(|| trimmed.rfind(']'));

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &trimmed[start..=end];
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                return Ok(ParsedOutput {
                    data,
                    raw: output.to_string(),
                    success: true,
                    error: None,
                });
            }
        }

        Ok(ParsedOutput {
            data: serde_json::Value::Null,
            raw: output.to_string(),
            success: false,
            error: Some("Failed to parse JSON".to_string()),
        })
    }

    fn get_format_instructions(&self) -> String {
        "Output should be a valid JSON object.".to_string()
    }
}

/// 结构化解析器
pub struct StructuredParser<T: for<'de> Deserialize<'de> + Serialize + Send + Sync> {
    schema: serde_json::Value,
    _marker: std::marker::PhantomData<T>,
}

impl<T: for<'de> Deserialize<'de> + Serialize + Send + Sync> StructuredParser<T> {
    pub fn new() -> Self {
        Self {
            schema: serde_json::Value::Null,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_schema(schema: serde_json::Value) -> Self {
        Self {
            schema,
            _marker: std::marker::PhantomData,
        }
    }
}

/// 列表解析器
pub struct ListParser {
    delimiter: String,
}

impl ListParser {
    pub fn new(delimiter: impl Into<String>) -> Self {
        Self {
            delimiter: delimiter.into(),
        }
    }
}

impl Default for ListParser {
    fn default() -> Self {
        Self::new("\n")
    }
}

#[async_trait]
impl OutputParser for ListParser {
    fn name(&self) -> &str {
        "list"
    }

    async fn parse(&self, output: &str) -> Layer3Result<ParsedOutput> {
        let items: Vec<String> = output
            .split(&self.delimiter)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(ParsedOutput {
            data: serde_json::to_value(items)?,
            raw: output.to_string(),
            success: true,
            error: None,
        })
    }

    fn get_format_instructions(&self) -> String {
        format!("Output should be a list separated by '{}'.", self.delimiter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_parser() {
        let parser = JsonParser::default();
        let result = parser.parse("{\"key\": \"value\"}").await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_list_parser() {
        let parser = ListParser::default();
        let result = parser.parse("a\nb\nc").await.unwrap();
        assert!(result.success);
        let items: Vec<String> = serde_json::from_value(result.data).unwrap();
        assert_eq!(items.len(), 3);
    }
}
