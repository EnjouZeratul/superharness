//! # Checksum Utilities
//!
//! 校验和计算和验证工具。

use sha2::{Digest, Sha256};

/// 检查点版本
const CHECKPOINT_VERSION: &str = "1.0";
const CHECKSUM_FIELD: &str = "_checksum";
const VERSION_FIELD: &str = "_version";

/// 校验和工具
pub struct ChecksumUtils;

impl ChecksumUtils {
    /// 计算数据的 SHA-256 校验和
    ///
    /// # Arguments
    /// * `data` - 要计算校验和的数据
    ///
    /// # Returns
    /// 十六进制格式的校验和字符串
    pub fn compute_checksum(data: &serde_json::Value) -> String {
        // 创建规范 JSON（排序键，无空白）
        let canonical = Self::canonicalize_json(data);

        // 计算 SHA-256
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 添加校验和到数据
    ///
    /// # Arguments
    /// * `data` - 检查点数据
    ///
    /// # Returns
    /// 带有校验和和版本的数据
    pub fn add_checksum(mut data: serde_json::Value) -> serde_json::Value {
        // 移除现有校验和
        if let Some(obj) = data.as_object_mut() {
            obj.remove(CHECKSUM_FIELD);
            obj.remove(VERSION_FIELD);
        }

        let checksum = Self::compute_checksum(&data);

        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                CHECKSUM_FIELD.to_string(),
                serde_json::Value::String(checksum),
            );
            obj.insert(
                VERSION_FIELD.to_string(),
                serde_json::Value::String(CHECKPOINT_VERSION.to_string()),
            );
        }

        data
    }

    /// 验证校验和
    ///
    /// # Arguments
    /// * `data` - 带有校验和的数据
    ///
    /// # Returns
    /// 元组：(是否有效, 错误信息)
    pub fn verify_checksum(data: &serde_json::Value) -> (bool, Option<String>) {
        let obj = match data.as_object() {
            Some(o) => o,
            None => return (false, Some("Data is not an object".to_string())),
        };

        // 检查校验和字段
        let expected_checksum = match obj.get(CHECKSUM_FIELD) {
            Some(v) => v.as_str().unwrap_or("").to_string(),
            None => return (false, Some("Missing checksum field".to_string())),
        };

        // 检查版本字段
        let version = match obj.get(VERSION_FIELD) {
            Some(v) => v.as_str().unwrap_or(""),
            None => return (false, Some("Missing version field".to_string())),
        };

        if version != CHECKPOINT_VERSION {
            return (
                false,
                Some(format!(
                    "Version mismatch: expected {}, got {}",
                    CHECKPOINT_VERSION, version
                )),
            );
        }

        // 计算实际校验和
        let mut data_copy = data.clone();
        if let Some(obj) = data_copy.as_object_mut() {
            obj.remove(CHECKSUM_FIELD);
            obj.remove(VERSION_FIELD);
        }

        let actual_checksum = Self::compute_checksum(&data_copy);

        if expected_checksum != actual_checksum {
            return (
                false,
                Some(format!(
                    "Checksum mismatch: expected {}..., got {}...",
                    &expected_checksum[..16.min(expected_checksum.len())],
                    &actual_checksum[..16.min(actual_checksum.len())]
                )),
            );
        }

        (true, None)
    }

    /// 规范化 JSON（排序键，紧凑格式）
    fn canonicalize_json(data: &serde_json::Value) -> String {
        // 递归排序对象键
        let sorted = Self::sort_json_keys(data);
        serde_json::to_string(&sorted).unwrap_or_default()
    }

    /// 递归排序 JSON 对象的键
    fn sort_json_keys(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut sorted_map = serde_json::Map::new();
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();

                for key in keys {
                    if let Some(val) = map.get(key) {
                        sorted_map.insert(key.clone(), Self::sort_json_keys(val));
                    }
                }

                serde_json::Value::Object(sorted_map)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(Self::sort_json_keys).collect())
            }
            other => other.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_checksum() {
        let data = serde_json::json!({"test": "value"});
        let checksum = ChecksumUtils::compute_checksum(&data);

        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 = 64 hex chars
    }

    #[test]
    fn test_add_and_verify_checksum() {
        let data = serde_json::json!({"session_id": "test123"});
        let data_with_checksum = ChecksumUtils::add_checksum(data);

        let (valid, error) = ChecksumUtils::verify_checksum(&data_with_checksum);
        assert!(valid);
        assert!(error.is_none());
    }

    #[test]
    fn test_verify_missing_checksum() {
        let data = serde_json::json!({"session_id": "test123"});

        let (valid, error) = ChecksumUtils::verify_checksum(&data);
        assert!(!valid);
        assert!(error.is_some());
    }

    #[test]
    fn test_deterministic_checksum() {
        let data1 = serde_json::json!({"b": 2, "a": 1});
        let data2 = serde_json::json!({"a": 1, "b": 2});

        let checksum1 = ChecksumUtils::compute_checksum(&data1);
        let checksum2 = ChecksumUtils::compute_checksum(&data2);

        // 键顺序不同，但校验和应该相同
        assert_eq!(checksum1, checksum2);
    }
}
