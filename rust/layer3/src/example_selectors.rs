//! # Example Selectors
//!
//! 示例选择器：为提示词选择最相关的示例。

use crate::retriever_engine::RetrieverEngine;
use crate::types::Layer3Result;
use async_trait::async_trait;

/// 示例选择器 trait
///
/// 定义示例选择接口。
#[async_trait]
pub trait ExampleSelector: Send + Sync {
    /// 选择示例
    async fn select_examples(&self, query: &str, top_k: usize) -> Layer3Result<Vec<Example>>;

    /// 添加示例
    async fn add_example(&self, example: Example) -> Layer3Result<bool>;

    /// 获取所有示例数量
    async fn count(&self) -> usize;
}

/// 示例
#[derive(Debug, Clone)]
pub struct Example {
    /// 输入
    pub input: String,
    /// 输出
    pub output: String,
    /// 元数据
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Example {
    pub fn new(input: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            output: output.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// 语义相似度示例选择器
pub struct SemanticExampleSelector {
    retriever: Box<dyn RetrieverEngine>,
    examples: Vec<Example>,
}

impl SemanticExampleSelector {
    pub fn new(retriever: Box<dyn RetrieverEngine>) -> Self {
        Self {
            retriever,
            examples: Vec::new(),
        }
    }
}

#[async_trait]
impl ExampleSelector for SemanticExampleSelector {
    async fn select_examples(&self, query: &str, top_k: usize) -> Layer3Result<Vec<Example>> {
        let _results = self.retriever.retrieve(query, top_k).await?;
        // 根据检索结果匹配示例
        Ok(self.examples.iter().take(top_k).cloned().collect())
    }

    async fn add_example(&self, _example: Example) -> Layer3Result<bool> {
        // 实际实现需要索引到 retriever
        Ok(true)
    }

    async fn count(&self) -> usize {
        self.examples.len()
    }
}

/// 固定长度示例选择器
pub struct LengthBasedSelector {
    examples: Vec<Example>,
    max_length: usize,
}

impl LengthBasedSelector {
    pub fn new(max_length: usize) -> Self {
        Self {
            examples: Vec::new(),
            max_length,
        }
    }

    /// 计算示例长度
    fn example_length(&self, example: &Example) -> usize {
        example.input.len() + example.output.len()
    }
}

#[async_trait]
impl ExampleSelector for LengthBasedSelector {
    async fn select_examples(&self, query: &str, top_k: usize) -> Layer3Result<Vec<Example>> {
        let query_len = query.len();
        let mut selected = Vec::new();
        let mut total_len = 0;

        for example in &self.examples {
            let ex_len = self.example_length(example);
            if total_len + query_len + ex_len <= self.max_length {
                selected.push(example.clone());
                total_len += ex_len;
                if selected.len() >= top_k {
                    break;
                }
            }
        }

        Ok(selected)
    }

    async fn add_example(&self, _example: Example) -> Layer3Result<bool> {
        Ok(true)
    }

    async fn count(&self) -> usize {
        self.examples.len()
    }
}

/// 随机示例选择器
pub struct RandomSelector {
    examples: Vec<Example>,
}

impl RandomSelector {
    pub fn new() -> Self {
        Self {
            examples: Vec::new(),
        }
    }
}

#[async_trait]
impl ExampleSelector for RandomSelector {
    async fn select_examples(&self, _query: &str, top_k: usize) -> Layer3Result<Vec<Example>> {
        // 随机选择（简化实现）
        Ok(self.examples.iter().take(top_k).cloned().collect())
    }

    async fn add_example(&self, _example: Example) -> Layer3Result<bool> {
        Ok(true)
    }

    async fn count(&self) -> usize {
        self.examples.len()
    }
}

impl Default for RandomSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_creation() {
        let ex = Example::new("input", "output");
        assert_eq!(ex.input, "input");
    }

    #[test]
    fn test_random_selector() {
        let selector = RandomSelector::new();
        assert_eq!(selector.examples.len(), 0);
    }
}
