//! # PDF Document Loader
//!
//! PDF 文件加载器（需要 pdf-extract 或类似库）。
//!
//! **[EXPERIMENTAL]** PDF 提取功能为实验性实现
//! - 当前仅返回占位文本，不进行实际 PDF 解析
//! - 完整实现计划在 v1.2.0 版本
//! - 需要添加 `pdf-extract` 或 `lopdf` 依赖

use crate::document_loaders::{DocumentLoader, LoadOptions};
use crate::retriever_engine::Document;
use crate::types::Layer3Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// PDF Loader 实现
///
/// # 实验性功能
///
/// 当前实现：
/// - ✅ 文件扩展名识别
/// - ⚠️ 内容提取返回占位文本
/// - ❌ 无实际 PDF 解析
///
/// 未来版本将添加：
/// - 基于页面的分割
/// - 元数据提取（作者、标题等）
/// - OCR 支持用于扫描文档
#[allow(dead_code)]
pub struct PdfLoader {
    #[allow(dead_code)]
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
        // 实验性实现：返回占位文本
        // 未来版本将使用 pdf-extract 或类似库解析 PDF
        Ok(
            Document::new("[PDF content extraction - experimental implementation]")
                .with_source(path.to_string_lossy().to_string()),
        )
    }

    async fn load_and_split(&self, path: PathBuf) -> Layer3Result<Vec<Document>> {
        // 实验性实现：按页分割（当前返回单个文档）
        Ok(vec![self.load(path).await?])
    }

    fn supports(&self, path: &std::path::Path) -> bool {
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

    #[test]
    fn test_pdf_loader_supports() {
        let loader = PdfLoader::new();
        assert!(loader.supports(std::path::Path::new("test.pdf")));
        assert!(!loader.supports(std::path::Path::new("test.txt")));
    }
}
