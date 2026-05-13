//! # Document Loaders
//!
//! 文档加载器：支持多种格式文档的加载。

pub mod csv;
pub mod json;
pub mod markdown;
pub mod pdf;
pub mod text;

use crate::retriever_engine::Document;
use crate::types::Layer3Result;
use async_trait::async_trait;
use std::path::PathBuf;

// Re-export loaders
pub use csv::CsvLoader;
pub use json::JsonLoader;
pub use markdown::MarkdownLoader;
pub use pdf::PdfLoader;
pub use text::TextLoader;

/// 文档加载器 trait
///
/// 定义文档加载的通用接口。
#[async_trait]
pub trait DocumentLoader: Send + Sync {
    /// 加载文档
    async fn load(&self, path: PathBuf) -> Layer3Result<Document>;

    /// 加载并分块
    async fn load_and_split(&self, path: PathBuf) -> Layer3Result<Vec<Document>>;

    /// 检查是否支持该文件类型
    fn supports(&self, path: &PathBuf) -> bool;

    /// 获取支持的扩展名列表
    fn extensions(&self) -> &[&str];
}

/// 批量加载器 trait
#[async_trait]
pub trait BatchLoader: DocumentLoader {
    /// 加载目录下所有文档
    async fn load_directory(&self, dir: PathBuf, recursive: bool) -> Layer3Result<Vec<Document>>;

    /// 批量加载文件
    async fn load_batch(&self, paths: Vec<PathBuf>) -> Layer3Result<Vec<(PathBuf, Document)>>;
}

/// 文档加载器注册表
pub struct LoaderRegistry {
    loaders: Vec<Box<dyn DocumentLoader>>,
}

impl LoaderRegistry {
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    pub fn register(&mut self, loader: Box<dyn DocumentLoader>) {
        self.loaders.push(loader);
    }

    pub fn get_loader(&self, path: &PathBuf) -> Option<&dyn DocumentLoader> {
        self.loaders
            .iter()
            .find(|l| l.supports(path))
            .map(|l| l.as_ref())
    }

    pub fn load(&self, path: PathBuf) -> Layer3Result<Document> {
        let loader = self
            .get_loader(&path)
            .ok_or_else(|| anyhow::anyhow!("No loader for: {:?}", path))?;

        // 需要异步调用，这里简化处理
        futures::executor::block_on(loader.load(path))
    }
}

impl Default for LoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 加载选项
#[derive(Debug, Clone, Default)]
pub struct LoadOptions {
    /// 编码
    pub encoding: Option<String>,
    /// 最大文件大小（字节）
    pub max_size: Option<u64>,
    /// 是否提取元数据
    pub extract_metadata: bool,
    /// 自定义解析选项
    pub parse_options: serde_json::Map<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_registry() {
        let registry = LoaderRegistry::new();
        assert!(registry.loaders.is_empty());
    }
}
