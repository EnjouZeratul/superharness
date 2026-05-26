//! 嵌入模型模块
//!
//! 文本嵌入、批量处理、缓存。
//!
//! 支持 OpenAI Embeddings API。需要设置 `OPENAI_API_KEY` 环境变量，
//! 或者通过 `EmbeddingsConfig` 提供配置。

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 默认嵌入模型
pub const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-ada-002";

/// 默认嵌入维度
pub const DEFAULT_EMBEDDING_DIMENSION: usize = 1536;

/// 嵌入模型配置
#[derive(Debug, Clone)]
pub struct EmbeddingsConfig {
    /// API 密钥
    pub api_key: String,
    /// API 基础 URL
    pub base_url: String,
    /// 模型名称
    pub model: String,
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
        }
    }
}

impl EmbeddingsConfig {
    /// 从环境变量创建配置
    ///
    /// 查找以下环境变量：
    /// - `OPENAI_API_KEY`: API 密钥（必需）
    /// - `OPENAI_BASE_URL`: API 基础 URL（可选）
    /// - `OPENAI_EMBEDDING_MODEL`: 嵌入模型名称（可选）
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            anyhow!("OPENAI_API_KEY environment variable not set. Please set it to use embeddings.")
        })?;

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let model = std::env::var("OPENAI_EMBEDDING_MODEL")
            .unwrap_or_else(|_| DEFAULT_EMBEDDING_MODEL.to_string());

        Ok(Self {
            api_key,
            base_url,
            model,
        })
    }

    /// 检查配置是否有效
    pub fn is_valid(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// 嵌入模型
#[derive(Debug)]
pub struct Embeddings {
    /// HTTP 客户端
    client: Client,
    /// 配置
    config: EmbeddingsConfig,
}

impl Embeddings {
    /// 创建新的嵌入模型实例
    ///
    /// # Errors
    ///
    /// 如果未配置 API 密钥，返回错误。
    pub fn new() -> Result<Self> {
        let config = EmbeddingsConfig::from_env()?;
        Self::with_config(config)
    }

    /// 使用自定义配置创建嵌入模型实例
    pub fn with_config(config: EmbeddingsConfig) -> Result<Self> {
        if !config.is_valid() {
            return Err(anyhow!(
                "Embeddings API not configured. Set OPENAI_API_KEY environment variable or provide a valid EmbeddingsConfig."
            ));
        }

        Ok(Self {
            client: Client::new(),
            config,
        })
    }

    /// 生成嵌入向量
    ///
    /// # Errors
    ///
    /// - 如果 API 调用失败
    /// - 如果响应解析失败
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No embedding returned"))
    }

    /// 批量生成嵌入向量
    ///
    /// # Errors
    ///
    /// - 如果 API 调用失败
    /// - 如果响应解析失败
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/embeddings", self.config.base_url);

        let request_body = OpenAiEmbeddingRequest {
            model: self.config.model.clone(),
            input: texts.to_vec(),
            encoding_format: Some("float".to_string()),
        };

        tracing::debug!("Sending embedding request for {} texts", texts.len());

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            tracing::error!("Embedding API error: {} - {}", status, response_text);
            return Err(anyhow!(
                "Embedding API request failed with status {}: {}",
                status,
                response_text
            ));
        }

        let response_body: OpenAiEmbeddingResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                anyhow!(
                    "Failed to parse embedding response: {} - {}",
                    e,
                    response_text
                )
            })?;

        // 按 index 排序并提取向量
        let mut embeddings: Vec<(usize, Vec<f32>)> = response_body
            .data
            .into_iter()
            .map(|item| (item.index, item.embedding))
            .collect();
        embeddings.sort_by_key(|(idx, _)| *idx);

        Ok(embeddings.into_iter().map(|(_, emb)| emb).collect())
    }

    /// 获取向量维度
    pub fn dimension(&self) -> usize {
        match self.config.model.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => DEFAULT_EMBEDDING_DIMENSION,
        }
    }

    /// 获取模型名称
    pub fn model_name(&self) -> &str {
        &self.config.model
    }
}

impl Default for Embeddings {
    fn default() -> Self {
        Self::new().expect("Failed to create Embeddings: OPENAI_API_KEY not set")
    }
}

// OpenAI Embeddings API 结构

#[derive(Serialize)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    usage: OpenAiEmbeddingUsage,
}

#[derive(Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    #[allow(dead_code)]
    object: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenAiEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 用于同步环境变量测试的锁
    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_from_env_missing_key() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        // 清除环境变量
        std::env::remove_var("OPENAI_API_KEY");
        let result = EmbeddingsConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OPENAI_API_KEY"));
    }

    #[test]
    fn test_config_from_env_with_key() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        // Clear any previously set model env var
        std::env::remove_var("OPENAI_EMBEDDING_MODEL");
        std::env::set_var("OPENAI_API_KEY", "test_key");
        let result = EmbeddingsConfig::from_env();
        std::env::remove_var("OPENAI_API_KEY");

        let config = result.expect("Config should be valid with OPENAI_API_KEY set");
        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.model, DEFAULT_EMBEDDING_MODEL);
    }

    #[test]
    fn test_config_from_env_custom_model() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        std::env::set_var("OPENAI_API_KEY", "test_key");
        std::env::set_var("OPENAI_EMBEDDING_MODEL", "text-embedding-3-small");
        let result = EmbeddingsConfig::from_env();
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("OPENAI_EMBEDDING_MODEL");

        let config = result.expect("Config should be valid with OPENAI_API_KEY set");
        assert_eq!(config.model, "text-embedding-3-small");
    }

    #[test]
    fn test_config_from_env_custom_base_url() {
        let _lock = ENV_TEST_LOCK.lock().unwrap();
        std::env::remove_var("OPENAI_EMBEDDING_MODEL");
        std::env::set_var("OPENAI_API_KEY", "test_key");
        std::env::set_var("OPENAI_BASE_URL", "https://custom.api.com/v1");
        let result = EmbeddingsConfig::from_env();
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("OPENAI_BASE_URL");

        let config = result.expect("Config should be valid with OPENAI_API_KEY set");
        assert_eq!(config.base_url, "https://custom.api.com/v1");
    }

    #[test]
    fn test_config_is_valid() {
        let config = EmbeddingsConfig {
            api_key: "key".to_string(),
            base_url: "url".to_string(),
            model: "model".to_string(),
        };
        assert!(config.is_valid());

        let invalid_config = EmbeddingsConfig::default();
        assert!(!invalid_config.is_valid());
    }

    #[test]
    fn test_embeddings_new_without_key() {
        std::env::remove_var("OPENAI_API_KEY");
        let result = Embeddings::new();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("OPENAI_API_KEY"));
    }

    #[test]
    fn test_embeddings_with_config_invalid() {
        let config = EmbeddingsConfig::default();
        let result = Embeddings::with_config(config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Embeddings API not configured"));
    }

    #[test]
    fn test_embeddings_with_config_valid() {
        let config = EmbeddingsConfig {
            api_key: "test_key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
        };
        let embeddings = Embeddings::with_config(config).unwrap();
        assert_eq!(embeddings.model_name(), DEFAULT_EMBEDDING_MODEL);
        assert_eq!(embeddings.dimension(), 1536);
    }

    #[test]
    fn test_dimension_for_models() {
        let config = EmbeddingsConfig {
            api_key: "test_key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "text-embedding-ada-002".to_string(),
        };
        let embeddings = Embeddings::with_config(config).unwrap();
        assert_eq!(embeddings.dimension(), 1536);

        let config = EmbeddingsConfig {
            api_key: "test_key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "text-embedding-3-large".to_string(),
        };
        let embeddings = Embeddings::with_config(config).unwrap();
        assert_eq!(embeddings.dimension(), 3072);
    }

    #[test]
    fn test_embed_batch_empty() {
        let config = EmbeddingsConfig {
            api_key: "test_key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
        };
        let embeddings = Embeddings::with_config(config).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(embeddings.embed_batch(&[]));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_request_serialization() {
        let request = OpenAiEmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: vec!["hello".to_string(), "world".to_string()],
            encoding_format: Some("float".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("text-embedding-ada-002"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{
            "data": [
                {"embedding": [0.1, 0.2, 0.3], "index": 0, "object": "embedding"},
                {"embedding": [0.4, 0.5, 0.6], "index": 1, "object": "embedding"}
            ],
            "model": "text-embedding-ada-002",
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;

        let response: OpenAiEmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(response.data[1].index, 1);
    }
}
