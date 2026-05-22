//! 可观测性模块
//!
//! Tracing、Metrics、Logs 统一接口。

/// 可观测性管理器
pub struct Observability {
    #[allow(dead_code)]
    enabled: bool,
}

impl Observability {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// 记录 span
    pub fn span(&self, name: &str) -> SpanGuard {
        tracing::info_span!("span", name = name);
        SpanGuard
    }

    /// 记录指标
    pub fn record_metric(&self, name: &str, value: f64) {
        tracing::debug!(metric = name, value = value);
    }
}

impl Default for Observability {
    fn default() -> Self {
        Self::new()
    }
}

/// Span 守卫
pub struct SpanGuard;
