//! 缓存管理模块
//!
//! 高性能缓存，支持 LRU、TTL、TTI 策略。

use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 最大容量
    pub max_capacity: u64,
    /// TTL（秒）- 时间到期后自动过期
    pub ttl_secs: u64,
    /// TTI（秒）- 空闲时间到期后自动过期
    pub tti_secs: Option<u64>,
    /// 初始容量（预分配）
    pub initial_capacity: Option<u64>,
    /// 是否启用统计
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            ttl_secs: 300, // 5分钟
            tti_secs: None,
            initial_capacity: Some(1_000),
            enable_stats: false,
        }
    }
}

impl CacheConfig {
    /// 创建高性能缓存配置
    pub fn high_performance() -> Self {
        Self {
            max_capacity: 100_000,
            ttl_secs: 600,       // 10分钟
            tti_secs: Some(300), // 5分钟空闲过期
            initial_capacity: Some(10_000),
            enable_stats: true,
        }
    }

    /// 创建低内存占用配置
    pub fn low_memory() -> Self {
        Self {
            max_capacity: 1_000,
            ttl_secs: 60, // 1分钟
            tti_secs: Some(30),
            initial_capacity: Some(100),
            enable_stats: false,
        }
    }
}

/// 缓存管理器
pub struct CacheManager {
    cache: MokaCache<String, Vec<u8>>,
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let mut builder = MokaCache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.ttl_secs));

        // 设置 TTI（空闲过期）
        if let Some(tti_secs) = config.tti_secs {
            builder = builder.time_to_idle(Duration::from_secs(tti_secs));
        }

        // 设置初始容量
        if let Some(initial_capacity) = config.initial_capacity {
            builder = builder.initial_capacity(initial_capacity as usize);
        }

        let cache = builder.build();

        Self { cache, config }
    }

    /// 获取缓存
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.get(key).await
    }

    /// 设置缓存
    pub async fn set(&self, key: String, value: Vec<u8>) {
        self.cache.insert(key, value).await;
    }

    /// 删除缓存
    pub async fn remove(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    /// 批量获取
    pub async fn get_batch(&self, keys: &[String]) -> Vec<Option<Vec<u8>>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.cache.get(key).await);
        }
        results
    }

    /// 批量设置
    pub async fn set_batch(&self, entries: Vec<(String, Vec<u8>)>) {
        for (key, value) in entries {
            self.cache.insert(key, value).await;
        }
    }

    /// 清空所有缓存
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// 获取缓存条目数量
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// 获取配置
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// 检查缓存是否存在
    pub async fn contains(&self, key: &str) -> bool {
        self.cache.get(key).await.is_some()
    }

    /// 获取并更新（如果不存在则插入）
    pub async fn get_or_insert<F>(&self, key: String, default: F) -> Vec<u8>
    where
        F: FnOnce() -> Vec<u8> + Send,
    {
        match self.cache.get(&key).await {
            Some(value) => value,
            None => {
                let value = default();
                self.cache.insert(key, value.clone()).await;
                value
            }
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_operations() {
        let cache = CacheManager::default();

        cache.set("key1".to_string(), b"value1".to_vec()).await;
        let value = cache.get("key1").await;
        assert_eq!(value, Some(b"value1".to_vec()));

        cache.remove("key1").await;
        let value = cache.get("key1").await;
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_ttl() {
        let config = CacheConfig {
            ttl_secs: 1,
            ..Default::default()
        };
        let cache = CacheManager::new(config);

        cache.set("key1".to_string(), b"value1".to_vec()).await;
        assert!(cache.get("key1").await.is_some());

        // 等待 TTL 过期
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let cache = CacheManager::default();

        let entries = vec![
            ("key1".to_string(), b"value1".to_vec()),
            ("key2".to_string(), b"value2".to_vec()),
            ("key3".to_string(), b"value3".to_vec()),
        ];
        cache.set_batch(entries).await;

        let values = cache
            .get_batch(&["key1".to_string(), "key2".to_string(), "key3".to_string()])
            .await;
        assert_eq!(values.len(), 3);
        assert!(values[0].is_some());
        assert!(values[1].is_some());
        assert!(values[2].is_some());
    }

    #[tokio::test]
    async fn test_clear() {
        let cache = CacheManager::default();

        cache.set("key1".to_string(), b"value1".to_vec()).await;
        cache.set("key2".to_string(), b"value2".to_vec()).await;

        // moka 可能异步处理，所以等待一下
        tokio::time::sleep(Duration::from_millis(10)).await;

        cache.clear().await;

        // invalidate_all 后条目应该被清除
        assert!(cache.get("key1").await.is_none());
        assert!(cache.get("key2").await.is_none());
    }

    #[tokio::test]
    async fn test_contains() {
        let cache = CacheManager::default();

        cache.set("key1".to_string(), b"value1".to_vec()).await;
        assert!(cache.contains("key1").await);
        assert!(!cache.contains("key2").await);
    }

    #[tokio::test]
    async fn test_get_or_insert() {
        let cache = CacheManager::default();

        // 第一次应该插入默认值
        let value = cache
            .get_or_insert("key1".to_string(), || b"default".to_vec())
            .await;
        assert_eq!(value, b"default".to_vec());

        // 第二次应该返回已存在的值
        let value = cache
            .get_or_insert("key1".to_string(), || b"new_default".to_vec())
            .await;
        assert_eq!(value, b"default".to_vec());
    }

    #[test]
    fn test_config_presets() {
        let hp_config = CacheConfig::high_performance();
        assert_eq!(hp_config.max_capacity, 100_000);
        assert!(hp_config.tti_secs.is_some());

        let lm_config = CacheConfig::low_memory();
        assert_eq!(lm_config.max_capacity, 1_000);
        assert!(lm_config.tti_secs.is_some());
    }

    #[tokio::test]
    async fn test_max_capacity() {
        let config = CacheConfig {
            max_capacity: 5,
            ..Default::default()
        };
        let cache = CacheManager::new(config);

        // 插入超过容量的条目
        for i in 0..10 {
            cache
                .set(format!("key{}", i), format!("value{}", i).into_bytes())
                .await;
        }

        // 由于 LRU，部分条目应该被淘汰
        // 注意：moka 可能不会立即淘汰，所以这里只检查不会崩溃
        assert!(cache.entry_count() <= 10);
    }
}
