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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_creation() {
        let obs = Observability::new();
        // 基础创建测试
        let _guard = obs.span("test_span");
    }

    #[test]
    fn test_observability_default() {
        let obs = Observability::default();
        let _guard = obs.span("default_span");
    }

    #[test]
    fn test_record_metric() {
        let obs = Observability::new();
        obs.record_metric("test_metric", 42.0);
        // 应该不崩溃
    }

    #[test]
    fn test_span_with_name() {
        let obs = Observability::new();
        let guard = obs.span("operation_name");
        // SpanGuard 存在即表示成功
        let _ = guard;
    }

    #[test]
    fn test_multiple_spans() {
        let obs = Observability::new();
        let _guard1 = obs.span("span1");
        let _guard2 = obs.span("span2");
        // 多个 span 应该可以共存
    }

    #[test]
    fn test_metric_zero_value() {
        let obs = Observability::new();
        obs.record_metric("zero_metric", 0.0);
    }

    #[test]
    fn test_metric_negative_value() {
        let obs = Observability::new();
        obs.record_metric("negative_metric", -1.0);
        // 应该接受负值
    }

    #[test]
    fn test_metric_large_value() {
        let obs = Observability::new();
        obs.record_metric("large_metric", 1_000_000.0);
    }

    #[test]
    fn test_span_names() {
        let obs = Observability::new();

        // 测试各种 span 名称
        let names = [
            "api_call",
            "database_query",
            "http_request",
            "processing",
            "validation",
        ];

        for name in names {
            let _guard = obs.span(name);
        }
    }

    #[test]
    fn test_observability_disabled() {
        let obs = Observability { enabled: false };
        let _guard = obs.span("disabled_span");
        // 即使禁用也应该能创建 span
    }
}
