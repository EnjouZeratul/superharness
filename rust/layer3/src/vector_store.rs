//! # Vector Store
//!
//! 向量存储：持久化向量索引。

use crate::types::{Layer3Result};
use crate::retriever_engine::{Chunk, RetrievalResult};
use async_trait::async_trait;
use std::collections::HashMap;

/// 向量存储 trait
///
/// 定义向量持久化存储接口。
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// 添加向量
    async fn add(&self, id: String, vector: Vec<f32>, metadata: HashMap<String, serde_json::Value>) -> Layer3Result<bool>;

    /// 批量添加向量
    async fn add_batch(&self, items: Vec<VectorItem>) -> Layer3Result<Vec<bool>>;

    /// 查询相似向量
    async fn query(&self, vector: Vec<f32>, top_k: usize) -> Layer3Result<Vec<RetrievalResult>>;

    /// 删除向量
    async fn delete(&self, id: &str) -> Layer3Result<bool>;

    /// 批量删除
    async fn delete_batch(&self, ids: &[String]) -> Layer3Result<usize>;

    /// 获取向量
    async fn get(&self, id: &str) -> Layer3Result<Option<VectorItem>>;

    /// 统计数量
    async fn count(&self) -> Layer3Result<usize>;

    /// 清空存储
    async fn clear(&self) -> Layer3Result<bool>;
}

/// 向量项
#[derive(Debug, Clone)]
pub struct VectorItem {
    /// 唯一 ID
    pub id: String,
    /// 向量数据
    pub vector: Vec<f32>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 关联内容（可选）
    pub content: Option<String>,
}

impl VectorItem {
    pub fn new(id: impl Into<String>, vector: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            vector,
            metadata: HashMap::new(),
            content: None,
        }
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// 向量存储配置
#[derive(Debug, Clone)]
pub struct VectorStoreConfig {
    /// 存储路径
    pub path: Option<String>,
    /// 向量维度
    pub dimension: usize,
    /// 距离度量
    pub metric: DistanceMetric,
    /// 索引类型
    pub index_type: IndexType,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            path: None,
            dimension: 1536,
            metric: DistanceMetric::Cosine,
            index_type: IndexType::Hnsw,
        }
    }
}

/// 距离度量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    /// 余弦相似度
    Cosine,
    /// 欧几里得距离
    Euclidean,
    /// 点积
    DotProduct,
    /// 曼哈顿距离
    Manhattan,
}

/// 索引类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// HNSW（高效近似最近邻）
    Hnsw,
    /// IVF（倒排文件索引）
    Ivf,
    /// Flat（暴力搜索）
    Flat,
    /// PQ（乘积量化）
    ProductQuantization,
}

/// 向量存储工厂 trait
pub trait VectorStoreFactory: Send + Sync {
    /// 创建向量存储
    fn create(&self, config: VectorStoreConfig) -> Layer3Result<Box<dyn VectorStore>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_item_builder() {
        let item = VectorItem::new("test", vec![1.0, 2.0, 3.0])
            .with_content("test content");
        assert_eq!(item.content, Some("test content".to_string()));
    }

    #[test]
    fn test_vector_store_config_default() {
        let config = VectorStoreConfig::default();
        assert_eq!(config.dimension, 1536);
        assert_eq!(config.metric, DistanceMetric::Cosine);
    }
}