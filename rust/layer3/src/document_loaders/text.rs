//! # Text Document Loader
//!
//! 纯文本文件加载器。

use crate::document_loaders::{DocumentLoader, LoadOptions};
use crate::retriever_engine::Document;
use crate::types::Layer3Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// Text Loader 实现
pub struct TextLoader {
    extensions: Vec<&'static str>,
    options: LoadOptions,
}

impl TextLoader {
    pub fn new() -> Self {
        Self {
            extensions: vec!["txt", "text", "log", "md"],
            options: LoadOptions::default(),
        }
    }

    pub fn with_options(options: LoadOptions) -> Self {
        Self {
            extensions: vec!["txt", "text", "log", "md"],
            options,
        }
    }
}

impl Default for TextLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentLoader for TextLoader {
    async fn load(&self, path: PathBuf) -> Layer3Result<Document> {
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(Document::new(content).with_source(path.to_string_lossy().to_string()))
    }

    async fn load_and_split(&self, path: PathBuf) -> Layer3Result<Vec<Document>> {
        let content = tokio::fs::read_to_string(&path).await?;
        // 按段落分割
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        Ok(paragraphs
            .into_iter()
            .filter(|p| !p.trim().is_empty())
            .enumerate()
            .map(|(i, p)| {
                Document::new(p.to_string()).with_source(format!(
                    "{}#{}",
                    path.to_string_lossy(),
                    i
                ))
            })
            .collect())
    }

    fn supports(&self, path: &PathBuf) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| self.extensions.contains(&e))
            .unwrap_or(false)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_loader_extensions() {
        let loader = TextLoader::new();
        assert!(loader.extensions().contains(&"txt"));
        assert!(loader.extensions().contains(&"md"));
    }
}
