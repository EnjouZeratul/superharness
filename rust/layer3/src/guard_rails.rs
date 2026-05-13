//! # Guard Rails
//!
//! 防护栏：输入输出安全检查。

use crate::types::{Layer3Result};
use async_trait::async_trait;

/// 防护栏 trait
///
/// 定义输入输出检查接口。
#[async_trait]
pub trait GuardRail: Send + Sync {
    /// 防护栏名称
    fn name(&self) -> &str;

    /// 检查输入
    async fn check_input(&self, input: &str) -> Layer3Result<GuardResult>;

    /// 检查输出
    async fn check_output(&self, output: &str) -> Layer3Result<GuardResult>;

    /// 修正输入（如果可能）
    async fn fix_input(&self, input: &str) -> Layer3Result<String>;

    /// 修正输出（如果可能）
    async fn fix_output(&self, output: &str) -> Layer3Result<String>;
}

/// 防护检查结果
#[derive(Debug, Clone)]
pub struct GuardResult {
    /// 是否通过
    pub passed: bool,
    /// 问题类型
    pub issue: Option<GuardIssue>,
    /// 建议修正
    pub suggestion: Option<String>,
}

/// 防护问题类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardIssue {
    /// 包含敏感信息
    SensitiveData,
    /// 格式错误
    FormatError,
    /// 内容过长
    TooLong,
    /// 内容过短
    TooShort,
    /// 包含危险指令
    DangerousInstruction,
    /// 偏离主题
    OffTopic,
    /// 自定义问题
    Custom(String),
}

/// 防护栏组合器
pub struct GuardRailsComposite {
    rails: Vec<Box<dyn GuardRail>>,
}

impl GuardRailsComposite {
    pub fn new() -> Self {
        Self { rails: Vec::new() }
    }

    pub fn add(&mut self, rail: Box<dyn GuardRail>) {
        self.rails.push(rail);
    }

    pub async fn check_input_all(&self, input: &str) -> Layer3Result<Vec<GuardResult>> {
        let mut results = Vec::new();
        for rail in &self.rails {
            results.push(rail.check_input(input).await?);
        }
        Ok(results)
    }

    pub async fn check_output_all(&self, output: &str) -> Layer3Result<Vec<GuardResult>> {
        let mut results = Vec::new();
        for rail in &self.rails {
            results.push(rail.check_output(output).await?);
        }
        Ok(results)
    }
}

impl Default for GuardRailsComposite {
    fn default() -> Self {
        Self::new()
    }
}

/// 长度防护栏
pub struct LengthGuard {
    min_length: usize,
    max_length: usize,
}

impl LengthGuard {
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self { min_length, max_length }
    }
}

impl Default for LengthGuard {
    fn default() -> Self {
        Self::new(1, 10000)
    }
}

#[async_trait]
impl GuardRail for LengthGuard {
    fn name(&self) -> &str {
        "length"
    }

    async fn check_input(&self, input: &str) -> Layer3Result<GuardResult> {
        let len = input.len();
        if len < self.min_length {
            return Ok(GuardResult {
                passed: false,
                issue: Some(GuardIssue::TooShort),
                suggestion: Some(format!("Minimum length: {}", self.min_length)),
            });
        }
        if len > self.max_length {
            return Ok(GuardResult {
                passed: false,
                issue: Some(GuardIssue::TooLong),
                suggestion: Some(format!("Maximum length: {}", self.max_length)),
            });
        }
        Ok(GuardResult {
            passed: true,
            issue: None,
            suggestion: None,
        })
    }

    async fn check_output(&self, output: &str) -> Layer3Result<GuardResult> {
        self.check_input(output).await
    }

    async fn fix_input(&self, input: &str) -> Layer3Result<String> {
        Ok(input.to_string())
    }

    async fn fix_output(&self, output: &str) -> Layer3Result<String> {
        if output.len() > self.max_length {
            Ok(output[..self.max_length].to_string())
        } else {
            Ok(output.to_string())
        }
    }
}

/// 正则防护栏
pub struct RegexGuard {
    pattern: regex::Regex,
    block_matches: bool,
    name: String,
}

impl RegexGuard {
    pub fn new(pattern: regex::Regex, block_matches: bool, name: impl Into<String>) -> Self {
        Self {
            pattern,
            block_matches,
            name: name.into(),
        }
    }
}

#[async_trait]
impl GuardRail for RegexGuard {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check_input(&self, input: &str) -> Layer3Result<GuardResult> {
        let matches = self.pattern.is_match(input);
        let passed = if self.block_matches { !matches } else { matches };
        Ok(GuardResult {
            passed,
            issue: if passed { None } else { Some(GuardIssue::FormatError) },
            suggestion: None,
        })
    }

    async fn check_output(&self, output: &str) -> Layer3Result<GuardResult> {
        self.check_input(output).await
    }

    async fn fix_input(&self, input: &str) -> Layer3Result<String> {
        Ok(self.pattern.replace_all(input, "").to_string())
    }

    async fn fix_output(&self, output: &str) -> Layer3Result<String> {
        self.fix_input(output).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_length_guard() {
        let guard = LengthGuard::new(5, 100);
        let result = guard.check_input("hello").await.unwrap();
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_length_guard_too_short() {
        let guard = LengthGuard::new(10, 100);
        let result = guard.check_input("hi").await.unwrap();
        assert!(!result.passed);
    }
}