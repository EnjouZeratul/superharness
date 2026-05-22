//! 输入验证模块
//!
//! 验证所有外部输入的格式和安全性。

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 验证结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 是否有效
    pub valid: bool,
    /// 错误消息
    pub errors: Vec<String>,
    /// 验证后的数据（可选）
    pub sanitized: Option<String>,
}

/// 输入验证器
pub struct InputValidator {
    /// 最大输入长度
    max_length: usize,
    /// 禁止的模式
    forbidden_patterns: Vec<String>,
    /// 必须的字段
    #[allow(dead_code)]
    required_fields: HashMap<String, bool>,
}

impl InputValidator {
    pub fn new() -> Self {
        Self {
            max_length: 100_000, // 100KB 默认上限
            forbidden_patterns: vec![
                // 潜在危险模式
                "<script>".to_string(),
                "javascript:".to_string(),
                "data:".to_string(),
            ],
            required_fields: HashMap::new(),
        }
    }

    /// 验证输入
    pub fn validate(&self, input: &str) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        // 检查长度
        if input.len() > self.max_length {
            errors.push(format!(
                "Input too long: {} bytes (max {})",
                input.len(),
                self.max_length
            ));
        }

        // 检查禁止模式
        for pattern in &self.forbidden_patterns {
            if input.contains(pattern) {
                errors.push(format!("Forbidden pattern detected: {}", pattern));
            }
        }

        // 检查空输入
        if input.trim().is_empty() {
            errors.push("Input is empty".to_string());
        }

        let valid = errors.is_empty();
        let sanitized = if valid {
            Some(self.sanitize(input))
        } else {
            None
        };

        Ok(ValidationResult {
            valid,
            errors,
            sanitized,
        })
    }

    /// 清理输入
    fn sanitize(&self, input: &str) -> String {
        // 移除控制字符
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// 设置最大长度
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }

    /// 添加禁止模式
    pub fn add_forbidden_pattern(mut self, pattern: String) -> Self {
        self.forbidden_patterns.push(pattern);
        self
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_input() {
        let validator = InputValidator::new();
        let result = validator.validate("Hello, world!").unwrap();
        assert!(result.valid);
        assert!(result.sanitized.is_some());
    }

    #[test]
    fn test_empty_input() {
        let validator = InputValidator::new();
        let result = validator.validate("").unwrap();
        assert!(!result.valid);
        assert!(result.errors.contains(&"Input is empty".to_string()));
    }

    #[test]
    fn test_forbidden_pattern() {
        let validator = InputValidator::new();
        let result = validator.validate("<script>alert('xss')</script>").unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn test_max_length_boundary() {
        // 测试刚好在边界上
        let validator = InputValidator::new().with_max_length(100);

        // 刚好 100 字节 - 应该有效
        let input_at_limit = "a".repeat(100);
        let result = validator.validate(&input_at_limit).unwrap();
        assert!(result.valid, "Input at max length should be valid");

        // 超过 100 字节 - 应该无效
        let input_over_limit = "a".repeat(101);
        let result = validator.validate(&input_over_limit).unwrap();
        assert!(!result.valid, "Input over max length should be invalid");
        assert!(result.errors.iter().any(|e| e.contains("too long")));
    }

    #[test]
    fn test_unicode_handling() {
        let validator = InputValidator::new();

        // 中文输入
        let result = validator.validate("你好世界，这是一个测试").unwrap();
        assert!(result.valid);

        // Emoji 输入
        let result = validator.validate("Hello 🦀 Rust 🚀🎉").unwrap();
        assert!(result.valid);

        // 混合 Unicode
        let result = validator.validate("日本語テスト العربية עברית").unwrap();
        assert!(result.valid);

        // Unicode 控制字符应该被清理
        let result = validator.validate("Hello\x00World").unwrap();
        assert!(result.valid);
        assert!(result.sanitized.unwrap().contains("HelloWorld"));
    }

    #[test]
    fn test_concurrent_validation() {
        use std::sync::Arc;
        use std::thread;

        let validator = Arc::new(InputValidator::new());
        let mut handles = vec![];

        for i in 0..10 {
            let v = Arc::clone(&validator);
            handles.push(thread::spawn(move || {
                let input = format!("Test input {}", i);
                v.validate(&input).unwrap()
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // 所有验证都应该成功
        for result in results {
            assert!(result.valid);
        }
    }

    #[test]
    fn test_sanitize_removes_control_chars() {
        let validator = InputValidator::new();

        // 包含各种控制字符的输入
        let input = "Hello\x00\x01\x02World\nNewLine\tTab";
        let result = validator.validate(input).unwrap();

        assert!(result.valid);
        let sanitized = result.sanitized.unwrap();
        // 控制字符应该被移除（除了 \n 和 \t）
        assert!(!sanitized.contains('\x00'));
        assert!(!sanitized.contains('\x01'));
        assert!(!sanitized.contains('\x02'));
        // 换行和制表符应该保留
        assert!(sanitized.contains('\n'));
        assert!(sanitized.contains('\t'));
    }

    #[test]
    fn test_whitespace_only_input() {
        let validator = InputValidator::new();

        let result = validator.validate("   \t\n  ").unwrap();
        assert!(!result.valid);
        assert!(result.errors.contains(&"Input is empty".to_string()));
    }

    #[test]
    fn test_custom_forbidden_patterns() {
        let validator = InputValidator::new()
            .add_forbidden_pattern("SELECT * FROM".to_string())
            .add_forbidden_pattern("DROP TABLE".to_string());

        // SQL 注入尝试
        let result = validator.validate("SELECT * FROM users").unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("SELECT * FROM")));

        let result = validator.validate("DROP TABLE users").unwrap();
        assert!(!result.valid);

        // 正常输入
        let result = validator
            .validate("SELECT your option from the menu")
            .unwrap();
        assert!(result.valid);
    }
}
