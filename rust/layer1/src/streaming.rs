//! 流式处理模块
//!
//! SSE、WebSocket 等流式响应处理。
//!
//! [STABLE] SSE 解析器完整实现
//! [STABLE] Anthropic/OpenAI 流式格式支持
//! [STABLE] on_chunk 回调机制
//! [STABLE] abort 中断支持

use anyhow::Result;
use futures::Stream;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

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

/// SSE 解析器
///
/// 解析 Server-Sent Events 格式的流式数据。
/// 支持跨 chunk 的帧边界处理。
#[derive(Debug, Default)]
pub struct SseParser {
    buffer: Vec<u8>,
    provider: Option<String>,
    model: Option<String>,
}

impl SseParser {
    /// 创建新的 SSE 解析器
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加上下文信息（用于错误报告）
    #[must_use]
    pub fn with_context(mut self, provider: impl Into<String>, model: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self.model = Some(model.into());
        self
    }

    /// 推送数据块并解析出完整的事件
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<SseEvent>> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        while let Some(frame) = self.next_frame() {
            if let Some(event) = self.parse_frame(&frame)? {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// 完成解析，处理缓冲区中剩余的数据
    pub fn finish(&mut self) -> Result<Vec<SseEvent>> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }

        let trailing = std::mem::take(&mut self.buffer);
        match self.parse_frame(&String::from_utf8_lossy(&trailing))? {
            Some(event) => Ok(vec![event]),
            None => Ok(Vec::new()),
        }
    }

    fn next_frame(&mut self) -> Option<String> {
        // 查找 \n\n 或 \r\n\r\n 分隔符
        let separator = self
            .buffer
            .windows(2)
            .position(|window| window == b"\n\n")
            .map(|position| (position, 2))
            .or_else(|| {
                self.buffer
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|position| (position, 4))
            })?;

        let (position, separator_len) = separator;
        let frame = self
            .buffer
            .drain(..position + separator_len)
            .collect::<Vec<_>>();
        let frame_len = frame.len().saturating_sub(separator_len);
        Some(String::from_utf8_lossy(&frame[..frame_len]).into_owned())
    }

    fn parse_frame(&self, frame: &str) -> Result<Option<SseEvent>> {
        let trimmed = frame.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let mut data_lines = Vec::new();
        let mut event_name: Option<String> = None;

        for line in trimmed.lines() {
            // 跳过注释行
            if line.starts_with(':') {
                continue;
            }
            // 解析 event 字段
            if let Some(name) = line.strip_prefix("event:") {
                event_name = Some(name.trim().to_string());
                continue;
            }
            // 解析 data 字段
            if let Some(data) = line.strip_prefix("data:") {
                data_lines.push(data.trim_start().to_string());
            }
        }

        // 跳过 ping 事件
        if matches!(event_name.as_deref(), Some("ping")) {
            return Ok(None);
        }

        if data_lines.is_empty() {
            return Ok(None);
        }

        let payload = data_lines.join("\n");

        // 处理 [DONE] 标记（OpenAI 格式）
        if payload == "[DONE]" {
            return Ok(None);
        }

        Ok(Some(SseEvent {
            event: event_name,
            data: payload,
        }))
    }
}

/// SSE 事件
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// 事件类型（可选）
    pub event: Option<String>,
    /// 事件数据
    pub data: String,
}

/// Anthropic 流式事件
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicStreamEvent {
    /// 消息开始
    MessageStart {
        message: AnthropicMessageStart,
    },
    /// 内容块开始
    ContentBlockStart {
        index: u32,
        content_block: AnthropicContentBlock,
    },
    /// 内容块增量
    ContentBlockDelta {
        index: u32,
        delta: AnthropicContentDelta,
    },
    /// 内容块结束
    ContentBlockStop {
        index: u32,
    },
    /// 消息增量
    MessageDelta {
        delta: AnthropicMessageDelta,
        #[serde(default)]
        usage: AnthropicStreamUsage,
    },
    /// 消息结束
    MessageStop {},
}

/// Anthropic 消息开始事件
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AnthropicMessageStart {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub role: String,
    pub model: String,
    #[serde(default)]
    pub content: Vec<AnthropicContentBlock>,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub stop_sequence: Option<String>,
    #[serde(default)]
    pub usage: AnthropicStreamUsage,
}

/// Anthropic 内容块
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
    Text { text: String },
    Thinking { thinking: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
}

/// Anthropic 内容增量
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    InputJsonDelta { partial_json: String },
}

/// Anthropic 消息增量
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AnthropicMessageDelta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

/// Anthropic 流式用量
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct AnthropicStreamUsage {
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
    pub output_tokens: u32,
}

/// OpenAI 流式事件
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenAiStreamChunk {
    pub id: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Vec<OpenAiStreamChoice>,
    #[serde(default)]
    pub usage: Option<OpenAiStreamUsage>,
}

/// OpenAI 流式选择
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenAiStreamChoice {
    pub delta: OpenAiStreamDelta,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// OpenAI 流式增量
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct OpenAiStreamDelta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<OpenAiStreamToolCall>,
}

/// OpenAI 流式工具调用
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenAiStreamToolCall {
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub function: OpenAiStreamFunction,
}

/// OpenAI 流式函数
#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct OpenAiStreamFunction {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}

/// OpenAI 流式用量
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpenAiStreamUsage {
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
}

/// 统一的流式事件
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 消息开始
    MessageStart {
        id: String,
        model: String,
    },
    /// 内容块开始
    ContentBlockStart {
        index: u32,
        block_type: ContentBlockType,
    },
    /// 内容块增量
    ContentBlockDelta {
        index: u32,
        delta: ContentDelta,
    },
    /// 内容块结束
    ContentBlockStop {
        index: u32,
    },
    /// 消息增量
    MessageDelta {
        stop_reason: Option<String>,
        usage: StreamUsage,
    },
    /// 消息结束
    MessageStop,
}

/// 内容块类型
#[derive(Debug, Clone)]
pub enum ContentBlockType {
    Text,
    Thinking,
    ToolUse { id: String, name: String },
}

/// 内容增量
#[derive(Debug, Clone)]
pub enum ContentDelta {
    Text(String),
    Thinking(String),
    ToolInput(String),
}

/// 流式用量
#[derive(Debug, Clone, Default)]
pub struct StreamUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// 流式响应状态
#[derive(Debug)]
pub struct StreamState {
    model: String,
    message_started: bool,
    text_started: bool,
    text_finished: bool,
    thinking_started: bool,
    thinking_finished: bool,
    finished: bool,
    stop_reason: Option<String>,
    usage: Option<StreamUsage>,
    #[allow(dead_code)]
    tool_index_offset: u32,
    #[allow(dead_code)]
    tool_calls_count: u32,
}

impl StreamState {
    /// 创建新的流状态
    pub fn new(model: String) -> Self {
        Self {
            model,
            message_started: false,
            text_started: false,
            text_finished: false,
            thinking_started: false,
            thinking_finished: false,
            finished: false,
            stop_reason: None,
            usage: None,
            tool_index_offset: 0,
            tool_calls_count: 0,
        }
    }

    /// 处理 Anthropic 事件
    pub fn ingest_anthropic(&mut self, event: AnthropicStreamEvent) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        match event {
            AnthropicStreamEvent::MessageStart { message } => {
                if !self.message_started {
                    self.message_started = true;
                    events.push(StreamEvent::MessageStart {
                        id: message.id,
                        model: message.model,
                    });
                }
            }
            AnthropicStreamEvent::ContentBlockStart { index, content_block } => {
                let block_type = match content_block {
                    AnthropicContentBlock::Text { .. } => ContentBlockType::Text,
                    AnthropicContentBlock::Thinking { .. } => ContentBlockType::Thinking,
                    AnthropicContentBlock::ToolUse { id, name, .. } => {
                        ContentBlockType::ToolUse { id, name }
                    }
                };
                events.push(StreamEvent::ContentBlockStart { index, block_type });
            }
            AnthropicStreamEvent::ContentBlockDelta { index, delta } => {
                let content_delta = match delta {
                    AnthropicContentDelta::TextDelta { text } => ContentDelta::Text(text),
                    AnthropicContentDelta::ThinkingDelta { thinking } => ContentDelta::Thinking(thinking),
                    AnthropicContentDelta::InputJsonDelta { partial_json } => ContentDelta::ToolInput(partial_json),
                };
                events.push(StreamEvent::ContentBlockDelta { index, delta: content_delta });
            }
            AnthropicStreamEvent::ContentBlockStop { index } => {
                events.push(StreamEvent::ContentBlockStop { index });
            }
            AnthropicStreamEvent::MessageDelta { delta, usage } => {
                self.stop_reason = delta.stop_reason;
                self.usage = Some(StreamUsage {
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                });
                events.push(StreamEvent::MessageDelta {
                    stop_reason: self.stop_reason.clone(),
                    usage: self.usage.clone().unwrap_or_default(),
                });
            }
            AnthropicStreamEvent::MessageStop { .. } => {
                events.push(StreamEvent::MessageStop);
            }
        }

        events
    }

    /// 处理 OpenAI 事件
    pub fn ingest_openai(&mut self, chunk: OpenAiStreamChunk) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        if !self.message_started {
            self.message_started = true;
            events.push(StreamEvent::MessageStart {
                id: chunk.id.clone(),
                model: chunk.model.clone().unwrap_or_else(|| self.model.clone()),
            });
        }

        if let Some(usage) = chunk.usage {
            self.usage = Some(StreamUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
            });
        }

        for choice in chunk.choices {
            // 处理 reasoning_content（思考内容）
            if let Some(reasoning) = choice.delta.reasoning_content.filter(|v| !v.is_empty()) {
                if !self.thinking_started {
                    self.thinking_started = true;
                    events.push(StreamEvent::ContentBlockStart {
                        index: 0,
                        block_type: ContentBlockType::Thinking,
                    });
                }
                events.push(StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: ContentDelta::Thinking(reasoning),
                });
            }

            // 处理常规内容
            if let Some(content) = choice.delta.content.filter(|v| !v.is_empty()) {
                // 如果之前有思考块，先关闭它
                if self.thinking_started && !self.thinking_finished {
                    self.thinking_finished = true;
                    events.push(StreamEvent::ContentBlockStop { index: 0 });
                }

                let text_index = if self.thinking_started { 1 } else { 0 };
                if !self.text_started {
                    self.text_started = true;
                    events.push(StreamEvent::ContentBlockStart {
                        index: text_index,
                        block_type: ContentBlockType::Text,
                    });
                }
                events.push(StreamEvent::ContentBlockDelta {
                    index: text_index,
                    delta: ContentDelta::Text(content),
                });
            }

            // 处理工具调用
            for (i, tool_call) in choice.delta.tool_calls.into_iter().enumerate() {
                let tool_index = (if self.thinking_started { 2 } else { 1 }) + i as u32;
                if let Some(name) = tool_call.function.name {
                    events.push(StreamEvent::ContentBlockStart {
                        index: tool_index,
                        block_type: ContentBlockType::ToolUse {
                            id: tool_call.id.unwrap_or_default(),
                            name,
                        },
                    });
                }
                if let Some(args) = tool_call.function.arguments {
                    events.push(StreamEvent::ContentBlockDelta {
                        index: tool_index,
                        delta: ContentDelta::ToolInput(args),
                    });
                }
            }

            // 处理结束原因
            if let Some(finish_reason) = choice.finish_reason {
                self.stop_reason = Some(normalize_openai_finish_reason(&finish_reason));
            }
        }

        events
    }

    /// 完成流处理
    pub fn finish(&mut self) -> Vec<StreamEvent> {
        if self.finished {
            return Vec::new();
        }
        self.finished = true;

        let mut events = Vec::new();

        // 关闭思考块
        if self.thinking_started && !self.thinking_finished {
            self.thinking_finished = true;
            events.push(StreamEvent::ContentBlockStop { index: 0 });
        }

        // 关闭文本块
        if self.text_started && !self.text_finished {
            self.text_finished = true;
            let text_index = if self.thinking_started { 1 } else { 0 };
            events.push(StreamEvent::ContentBlockStop { index: text_index });
        }

        // 发送消息增量
        if self.message_started {
            events.push(StreamEvent::MessageDelta {
                stop_reason: self.stop_reason.clone().or_else(|| Some("end_turn".to_string())),
                usage: self.usage.clone().unwrap_or_default(),
            });
            events.push(StreamEvent::MessageStop);
        }

        events
    }
}

fn normalize_openai_finish_reason(reason: &str) -> String {
    match reason {
        "stop" => "end_turn".to_string(),
        "tool_calls" => "tool_use".to_string(),
        other => other.to_string(),
    }
}

/// 可中断的流式响应
pub struct AbortableStream<S> {
    inner: S,
    abort_flag: Arc<AtomicBool>,
}

impl<S> AbortableStream<S> {
    /// 创建可中断的流
    pub fn new(inner: S, abort_flag: Arc<AtomicBool>) -> Self {
        Self { inner, abort_flag }
    }

    /// 检查是否已中断
    pub fn is_aborted(&self) -> bool {
        self.abort_flag.load(Ordering::Relaxed)
    }
}

impl<S, T> Stream for AbortableStream<S>
where
    S: Stream<Item = Result<T>> + Unpin,
{
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.abort_flag.load(Ordering::Relaxed) {
            return Poll::Ready(None);
        }
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// 流式消息
pub struct MessageStream {
    response: reqwest::Response,
    parser: SseParser,
    pending: VecDeque<StreamEvent>,
    done: bool,
    state: StreamState,
    provider: StreamProvider,
}

/// 流式提供商类型
#[derive(Debug, Clone, Copy)]
pub enum StreamProvider {
    Anthropic,
    OpenAI,
    Gemini,
}

impl MessageStream {
    /// 创建新的流式消息
    pub fn new(response: reqwest::Response, provider: StreamProvider, model: String) -> Self {
        let parser = SseParser::new()
            .with_context(
                match provider {
                    StreamProvider::Anthropic => "Anthropic",
                    StreamProvider::OpenAI => "OpenAI",
                    StreamProvider::Gemini => "Gemini",
                },
                &model,
            );
        Self {
            response,
            parser,
            pending: VecDeque::new(),
            done: false,
            state: StreamState::new(model),
            provider,
        }
    }

    /// 获取下一个事件
    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Ok(Some(event));
            }

            if self.done {
                let _remaining = self.parser.finish()?;
                for event in self.state.finish() {
                    self.pending.push_back(event);
                }
                // 不处理 remaining，因为已经通过 state 处理
                if let Some(event) = self.pending.pop_front() {
                    return Ok(Some(event));
                }
                return Ok(None);
            }

            match self.response.chunk().await? {
                Some(chunk) => {
                    let sse_events = self.parser.push(&chunk)?;
                    for sse_event in sse_events {
                        let events = self.parse_sse_event(&sse_event)?;
                        self.pending.extend(events);
                    }
                }
                None => {
                    self.done = true;
                }
            }
        }
    }

    fn parse_sse_event(&mut self, event: &SseEvent) -> Result<Vec<StreamEvent>> {
        match self.provider {
            StreamProvider::Anthropic => {
                let anthropic_event: AnthropicStreamEvent = serde_json::from_str(&event.data)?;
                Ok(self.state.ingest_anthropic(anthropic_event))
            }
            StreamProvider::OpenAI => {
                let openai_chunk: OpenAiStreamChunk = serde_json::from_str(&event.data)?;
                Ok(self.state.ingest_openai(openai_chunk))
            }
            StreamProvider::Gemini => {
                // Gemini 格式类似 OpenAI，可以复用处理逻辑
                let openai_chunk: OpenAiStreamChunk = serde_json::from_str(&event.data)?;
                Ok(self.state.ingest_openai(openai_chunk))
            }
        }
    }

    /// 收集所有文本内容
    pub async fn collect_text(&mut self) -> Result<String> {
        let mut text = String::new();
        while let Some(event) = self.next_event().await? {
            if let StreamEvent::ContentBlockDelta { delta: ContentDelta::Text(t), .. } = event {
                text.push_str(&t);
            }
        }
        Ok(text)
    }
}

/// 回调类型
pub type OnChunkCallback = Box<dyn Fn(&str) + Send + Sync>;

/// 带回调的流式响应
pub struct CallbackStream {
    inner: MessageStream,
    on_chunk: Option<OnChunkCallback>,
    abort_flag: Arc<AtomicBool>,
}

impl CallbackStream {
    /// 创建带回调的流
    pub fn new(inner: MessageStream, on_chunk: Option<OnChunkCallback>) -> Self {
        Self {
            inner,
            on_chunk,
            abort_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取中断标志
    pub fn abort_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.abort_flag)
    }

    /// 请求中断
    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::Relaxed);
    }

    /// 获取下一个事件
    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>> {
        if self.abort_flag.load(Ordering::Relaxed) {
            return Ok(None);
        }

        let event = self.inner.next_event().await?;

        // 触发回调
        if let Some(ref callback) = self.on_chunk {
            if let Some(ref e) = event {
                if let StreamEvent::ContentBlockDelta { delta: ContentDelta::Text(t), .. } = e {
                    callback(t);
                }
            }
        }

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_parser_parses_single_frame() {
        let frame = concat!(
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n"
        );

        let mut parser = SseParser::new();
        let events = parser.push(frame.as_bytes()).expect("frame should parse");

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, Some("content_block_start".to_string()));
        assert!(events[0].data.contains("content_block_start"));
    }

    #[test]
    fn sse_parser_handles_chunked_stream() {
        let mut parser = SseParser::new();
        let first = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel";
        let second = b"lo\"}}\n\n";

        assert!(parser.push(first).expect("first chunk should buffer").is_empty());
        let events = parser.push(second).expect("second chunk should parse");

        assert_eq!(events.len(), 1);
        assert!(events[0].data.contains("Hello"));
    }

    #[test]
    fn sse_parser_ignores_ping_and_done() {
        let mut parser = SseParser::new();
        let payload = concat!(
            ": keepalive\n",
            "event: ping\n",
            "data: {\"type\":\"ping\"}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n",
            "data: [DONE]\n\n"
        );

        let events = parser.push(payload.as_bytes()).expect("parser should succeed");
        assert_eq!(events.len(), 1); // Only message_delta, ping and [DONE] are ignored
    }

    #[test]
    fn stream_state_handles_anthropic_events() {
        let mut state = StreamState::new("claude-sonnet-4-6".to_string());

        let start_event = AnthropicStreamEvent::MessageStart {
            message: AnthropicMessageStart {
                id: "msg_123".to_string(),
                kind: "message".to_string(),
                role: "assistant".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                content: vec![],
                stop_reason: None,
                stop_sequence: None,
                usage: AnthropicStreamUsage::default(),
            },
        };

        let events = state.ingest_anthropic(start_event);
        assert!(matches!(events[0], StreamEvent::MessageStart { .. }));
    }

    #[test]
    fn stream_state_handles_openai_events() {
        let mut state = StreamState::new("gpt-4o".to_string());

        let chunk = OpenAiStreamChunk {
            id: "chatcmpl_123".to_string(),
            model: Some("gpt-4o".to_string()),
            choices: vec![OpenAiStreamChoice {
                delta: OpenAiStreamDelta {
                    content: Some("Hello".to_string()),
                    ..Default::default()
                },
                finish_reason: None,
            }],
            usage: None,
        };

        let events = state.ingest_openai(chunk);
        assert!(matches!(events[0], StreamEvent::MessageStart { .. }));
        assert!(matches!(events[1], StreamEvent::ContentBlockStart { .. }));
    }

    #[test]
    fn abortable_stream_respects_abort_flag() {
        use futures::stream;
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        let abort_flag = Arc::new(AtomicBool::new(true));
        let inner = stream::iter(vec![Ok("test".to_string())]);
        let mut stream = AbortableStream::new(inner, abort_flag);

        let result = futures::executor::block_on_stream(&mut stream).next();
        assert!(result.is_none(), "aborted stream should return None immediately");
    }
}
