//! # CSV Document Loader
//!
//! CSV 文件加载器。

use crate::document_loaders::{DocumentLoader, LoadOptions};
use crate::retriever_engine::Document;
use crate::types::Layer3Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// CSV Loader 实现
pub struct CsvLoader {
    options: LoadOptions,
    /// 分隔符
    delimiter: char,
    /// 是否有表头
    has_header: bool,
}

impl CsvLoader {
    pub fn new() -> Self {
        Self {
            options: LoadOptions::default(),
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(delimiter: char) -> Self {
        Self {
            options: LoadOptions::default(),
            delimiter,
            has_header: true,
        }
    }
}

impl Default for CsvLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentLoader for CsvLoader {
    async fn load(&self, path: PathBuf) -> Layer3Result<Document> {
        let content = tokio::fs::read_to_string(&path).await?;
        Ok(Document::new(content).with_source(path.to_string_lossy().to_string()))
    }

    async fn load_and_split(&self, path: PathBuf) -> Layer3Result<Vec<Document>> {
        let content = tokio::fs::read_to_string(&path).await?;
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Ok(Vec::new());
        }

        // 解析表头（如果有）
        let header_line = if self.has_header { lines[0] } else { "" };
        let headers: Vec<&str> = header_line.split(self.delimiter).collect();

        let start_idx = if self.has_header { 1 } else { 0 };
        let mut documents = Vec::new();

        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            if line.trim().is_empty() {
                continue;
            }

            let values: Vec<&str> = line.split(self.delimiter).collect();
            let mut content_parts = Vec::new();

            // 如果有表头，使用键值对格式
            for (j, value) in values.iter().enumerate() {
                if j < headers.len() {
                    content_parts.push(format!("{}: {}", headers[j], value));
                } else {
                    content_parts.push(value.to_string());
                }
            }

            documents.push(
                Document::new(content_parts.join(", "))
                    .with_source(format!("{}#{}", path.to_string_lossy(), i))
            );
        }

        Ok(documents)
    }

    fn supports(&self, path: &PathBuf) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "csv" || e == "tsv")
            .unwrap_or(false)
    }

    fn extensions(&self) -> &[&str] {
        &["csv", "tsv"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_loader_extensions() {
        let loader = CsvLoader::new();
        assert!(loader.extensions().contains(&"csv"));
    }
}