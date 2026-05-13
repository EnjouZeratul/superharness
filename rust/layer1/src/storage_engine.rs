//! 存储引擎模块
//!
//! 统一的存储抽象，支持文件、对象存储等。

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储类型
    pub storage_type: StorageType,
    /// 基础路径
    pub base_path: String,
    /// 最大文件大小（字节）
    pub max_file_size: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: StorageType::FileSystem,
            base_path: "./data".to_string(),
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// 存储类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    FileSystem,
    Memory,
    S3,
}

/// 存储引擎
pub struct StorageEngine {
    config: StorageConfig,
}

impl StorageEngine {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// 读取数据
    pub async fn read(&self, key: &str) -> Result<Vec<u8>> {
        match self.config.storage_type {
            StorageType::FileSystem => {
                let path = PathBuf::from(&self.config.base_path).join(key);
                let data = tokio::fs::read(&path).await?;
                Ok(data)
            }
            StorageType::Memory => {
                // TODO: 实现内存存储
                Err(anyhow::anyhow!("Memory storage not implemented"))
            }
            StorageType::S3 => {
                // TODO: 实现 S3 存储
                Err(anyhow::anyhow!("S3 storage not implemented"))
            }
        }
    }

    /// 写入数据
    pub async fn write(&self, key: &str, data: &[u8]) -> Result<()> {
        match self.config.storage_type {
            StorageType::FileSystem => {
                let path = PathBuf::from(&self.config.base_path).join(key);

                // 确保目录存在
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                tokio::fs::write(&path, data).await?;
                Ok(())
            }
            StorageType::Memory => Err(anyhow::anyhow!("Memory storage not implemented")),
            StorageType::S3 => Err(anyhow::anyhow!("S3 storage not implemented")),
        }
    }

    /// 删除数据
    pub async fn delete(&self, key: &str) -> Result<()> {
        match self.config.storage_type {
            StorageType::FileSystem => {
                let path = PathBuf::from(&self.config.base_path).join(key);
                tokio::fs::remove_file(&path).await?;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Not implemented")),
        }
    }

    /// 检查是否存在
    pub async fn exists(&self, key: &str) -> Result<bool> {
        match self.config.storage_type {
            StorageType::FileSystem => {
                let path = PathBuf::from(&self.config.base_path).join(key);
                Ok(path.exists())
            }
            _ => Err(anyhow::anyhow!("Not implemented")),
        }
    }

    /// 列出所有键
    pub async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        match self.config.storage_type {
            StorageType::FileSystem => {
                let path = PathBuf::from(&self.config.base_path).join(prefix);
                let mut entries = Vec::new();

                if path.is_dir() {
                    let mut dir = tokio::fs::read_dir(&path).await?;
                    while let Some(entry) = dir.next_entry().await? {
                        if let Some(name) = entry.file_name().to_str() {
                            entries.push(name.to_string());
                        }
                    }
                }

                Ok(entries)
            }
            _ => Err(anyhow::anyhow!("Not implemented")),
        }
    }
}

impl Default for StorageEngine {
    fn default() -> Self {
        Self::new(StorageConfig::default())
    }
}
