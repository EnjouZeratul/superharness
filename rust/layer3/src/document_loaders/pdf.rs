//! # PDF Document Loader
//!
//! PDF 文件加载器（需要 pdf-extract 或类似库）。

use crate::document_loaders::{DocumentLoader, LoadOptions};
use crate::retriever_engine::Document;
use crate::types::Layer3Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// PDF Loader 实现
///
/// 注意：完整实现需要添加 pdf 解析库依赖。
/// 当前为 stub 实现。
pub struct PdfLoader {
    options: LoadOptions,
}

impl PdfLoader {
    pub fn new() -> Self {
        Self {
            options: LoadOptions::default(),
        }
    }
}

impl Default for PdfLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentLoader for PdfLoader {
    async fn load(&self, path: PathBuf) -> Layer3Result<Document> {
        // Stub: 实际实现需要 pdf 解析库
        // 例如: pdf-extract, lopdf 等
        Ok(Document::new("[PDF content extraction not implemented]")
            .with_source(path.to_string_lossy().to_string()))
    }

    async fn load_and_split(&self, path: PathBuf) -> Layer3Result<Vec<Document>> {
        // Stub: 按页分割
        Ok(vec![self.load(path).await?])
    }

    fn supports(&self, path: &PathBuf) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "pdf")
            .unwrap_or(false)
    }

    fn extensions(&self) -> &[&str] {
        &["pdf"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_loader_extensions() {
        let loader = PdfLoader::new();
        assert!(loader.extensions().contains(&"pdf"));
    }
}