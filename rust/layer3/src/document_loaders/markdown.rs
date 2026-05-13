//! # Markdown Document Loader
//!
//! Markdown 文件加载器，支持提取结构信息。

use crate::document_loaders::{DocumentLoader, LoadOptions};
use crate::retriever_engine::Document;
use crate::types::Layer3Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// Markdown Loader 实现
pub struct MarkdownLoader {
    options: LoadOptions,
}

impl MarkdownLoader {
    pub fn new() -> Self {
        Self {
            options: LoadOptions::default(),
        }
    }
}

impl Default for MarkdownLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentLoader for MarkdownLoader {
    async fn load(&self, path: PathBuf) -> Layer3Result<Document> {
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(Document::new(content).with_source(path.to_string_lossy().to_string()))
    }

    async fn load_and_split(&self, path: PathBuf) -> Layer3Result<Vec<Document>> {
        let content = tokio::fs::read_to_string(&path).await?;

        // 按标题分割
        let mut documents = Vec::new();
        let mut current_section = String::new();
        let mut current_title = String::from("intro");

        for line in content.lines() {
            if line.starts_with("#") {
                // 新标题，保存当前节
                if !current_section.trim().is_empty() {
                    documents.push(
                        Document::new(current_section.trim().to_string())
                            .with_source(format!("{}#{}", path.to_string_lossy(), current_title))
                    );
                }
                current_title = line.trim_start_matches('#').trim().to_string();
                current_section = format!("{}\n\n", line);
            } else {
                current_section.push_str(line);
                current_section.push('\n');
            }
        }

        // 保存最后一节
        if !current_section.trim().is_empty() {
            documents.push(
                Document::new(current_section.trim().to_string())
                    .with_source(format!("{}#{}", path.to_string_lossy(), current_title))
            );
        }

        Ok(documents)
    }

    fn supports(&self, path: &PathBuf) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "md" || e == "markdown")
            .unwrap_or(false)
    }

    fn extensions(&self) -> &[&str] {
        &["md", "markdown"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_loader_extensions() {
        let loader = MarkdownLoader::new();
        assert!(loader.extensions().contains(&"md"));
    }
}