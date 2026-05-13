//! # Retriever Engine
//!
//! 检索引擎：向量相似度检索和 RAG 支持。

use crate::types::Layer3Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// 检索引擎 trait
///
/// 提供向量相似度检索能力。
#[async_trait]
pub trait RetrieverEngine: Send + Sync {
    /// 索引文档
    async fn index(&self, documents: Vec<Document>) -> Layer3Result<Vec<String>>;

    /// 检索相似文档
    async fn retrieve(&self, query: &str, top_k: usize) -> Layer3Result<Vec<RetrievalResult>>;

    /// 混合检索（向量 + 关键词）
    async fn hybrid_retrieve(
        &self,
        query: &str,
        top_k: usize,
    ) -> Layer3Result<Vec<RetrievalResult>>;

    /// 删除文档
    async fn delete(&self, doc_ids: &[String]) -> Layer3Result<bool>;

    /// 清空索引
    async fn clear(&self) -> Layer3Result<bool>;

    /// 获取文档数量
    async fn count(&self) -> Layer3Result<usize>;
}

/// 文档结构
#[derive(Debug, Clone)]
pub struct Document {
    /// 文档 ID（可选，自动生成）
    pub id: Option<String>,
    /// 文档内容
    pub content: String,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 来源（文件路径、URL 等）
    pub source: Option<String>,
}

impl Document {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: None,
            content: content.into(),
            metadata: HashMap::new(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// 检索结果
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    /// 文档 ID
    pub doc_id: String,
    /// 文档内容
    pub content: String,
    /// 相似度分数 (0.0-1.0)
    pub score: f32,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 来源
    pub source: Option<String>,
}

/// Embedding 模型 trait
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// 生成文本嵌入向量
    async fn embed(&self, text: &str) -> Layer3Result<Vec<f32>>;

    /// 批量生成嵌入向量
    async fn embed_batch(&self, texts: &[String]) -> Layer3Result<Vec<Vec<f32>>>;

    /// 获取向量维度
    fn dimension(&self) -> usize;

    /// 模型名称
    fn model_name(&self) -> &str;
}

/// 分块策略 trait
pub trait ChunkingStrategy: Send + Sync {
    /// 分块文档
    fn chunk(&self, document: &Document) -> Vec<Chunk>;
}

/// 文档分块
#[derive(Debug, Clone)]
pub struct Chunk {
    /// 分块 ID
    pub id: String,
    /// 文档 ID
    pub doc_id: String,
    /// 分块内容
    pub content: String,
    /// 在原文中的位置
    pub position: ChunkPosition,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 分块位置
#[derive(Debug, Clone, Copy)]
pub struct ChunkPosition {
    /// 起始字符位置
    pub start: usize,
    /// 结束字符位置
    pub end: usize,
    /// 分块索引
    pub index: usize,
    /// 总分块数
    pub total: usize,
}

/// 固定大小分块策略
#[derive(Debug, Clone)]
pub struct FixedSizeChunker {
    /// 分块大小（字符数）
    pub chunk_size: usize,
    /// 重叠大小
    pub overlap: usize,
}

impl FixedSizeChunker {
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
        }
    }
}

impl Default for FixedSizeChunker {
    fn default() -> Self {
        Self {
            chunk_size: 500,
            overlap: 50,
        }
    }
}

impl ChunkingStrategy for FixedSizeChunker {
    fn chunk(&self, document: &Document) -> Vec<Chunk> {
        let content = &document.content;
        if content.len() <= self.chunk_size {
            return vec![Chunk {
                id: format!("{}-0", document.id.as_deref().unwrap_or("doc")),
                doc_id: document.id.clone().unwrap_or_default(),
                content: content.clone(),
                position: ChunkPosition {
                    start: 0,
                    end: content.len(),
                    index: 0,
                    total: 1,
                },
                metadata: document.metadata.clone(),
            }];
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < content.len() {
            let end = (start + self.chunk_size).min(content.len());
            chunks.push(Chunk {
                id: format!("{}-{}", document.id.as_deref().unwrap_or("doc"), index),
                doc_id: document.id.clone().unwrap_or_default(),
                content: content[start..end].to_string(),
                position: ChunkPosition {
                    start,
                    end,
                    index,
                    total: 0, // 将在最后更新
                },
                metadata: document.metadata.clone(),
            });
            // 防止死循环：到达末尾时直接设置 start = end
            start = if end < content.len() {
                end.saturating_sub(self.overlap)
            } else {
                end
            };
            index += 1;
        }

        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.position.total = total;
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_builder() {
        let doc = Document::new("test content")
            .with_source("test.txt")
            .with_metadata("key", serde_json::json!("value"));
        assert_eq!(doc.source, Some("test.txt".to_string()));
    }

    #[test]
    fn test_fixed_size_chunker() {
        let chunker = FixedSizeChunker::new(100, 20);
        let doc = Document::new("a".repeat(250));
        let chunks = chunker.chunk(&doc);
        assert!(!chunks.is_empty());
    }
}
