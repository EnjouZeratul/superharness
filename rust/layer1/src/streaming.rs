//! 流式处理模块
//!
//! SSE、WebSocket 等流式响应处理。

use anyhow::Result;
use futures::Stream;

/// 流处理器
pub struct StreamHandler;

impl StreamHandler {
    /// 创建 SSE 流
    pub fn create_sse_stream(
        source: impl Stream<Item = Result<String>> + Send + 'static,
    ) -> impl Stream<Item = Result<String>> {
        use futures::StreamExt;

        source.map(|item| match item {
            Ok(data) => Ok(format!("data: {}\n\n", data)),
            Err(e) => Err(e),
        })
    }
}
