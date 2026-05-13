//! # JSON Document Loader
//!
//! JSON 文件加载器。

use crate::document_loaders::{DocumentLoader, LoadOptions};
use crate::retriever_engine::Document;
use crate::types::Layer3Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// JSON Loader 实现
pub struct JsonLoader {
    options: LoadOptions,
    /// JSON Pointer 或 jq 查询（可选）
    query: Option<String>,
}

impl JsonLoader {
    pub fn new() -> Self {
        Self {
            options: LoadOptions::default(),
            query: None,
        }
    }

    pub fn with_query(query: impl Into<String>) -> Self {
        Self {
            options: LoadOptions::default(),
            query: Some(query.into()),
        }
    }
}

impl Default for JsonLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentLoader for JsonLoader {
    async fn load(&self, path: PathBuf) -> Layer3Result<Document> {
        let content = tokio::fs::read_to_string(&path).await?;

        // 验证 JSON 格式
        let _: serde_json::Value = serde_json::from_str(&content)?;

        Ok(Document::new(content).with_source(path.to_string_lossy().to_string()))
    }

    async fn load_and_split(&self, path: PathBuf) -> Layer3Result<Vec<Document>> {
        let content = tokio::fs::read_to_string(&path).await?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        // 如果是数组，每个元素作为一个文档
        if let serde_json::Value::Array(arr) = json {
            return Ok(arr
                .into_iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    if let Ok(s) = serde_json::to_string(&v) {
                        Some(Document::new(s).with_source(format!(
                            "{}[{}]",
                            path.to_string_lossy(),
                            i
                        )))
                    } else {
                        None
                    }
                })
                .collect());
        }

        // 否则作为单个文档
        Ok(vec![self.load(path).await?])
    }

    fn supports(&self, path: &PathBuf) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "json")
            .unwrap_or(false)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_loader_extensions() {
        let loader = JsonLoader::new();
        assert!(loader.extensions().contains(&"json"));
    }
}
