"""Streaming Tests

测试 SSE 解析器和流式状态处理。
"""

import pytest
from continuum_sdk.llm.streaming import (
    SseParser,
    SseEvent,
    StreamState,
    StreamEvent,
    CallbackStream,
    ContentBlockType,
)


class TestSseParser:
    """测试 SSE 解析器"""

    def test_parse_single_frame(self):
        """解析单个帧"""
        frame = "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n"

        parser = SseParser()
        events = parser.push(frame)

        assert len(events) == 1
        assert events[0].event == "content_block_start"
        assert "content_block_start" in events[0].data

    def test_parse_chunked_stream(self):
        """解析跨 chunk 的流"""
        parser = SseParser()

        # 第一块（不完整）
        first = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel"
        events = parser.push(first)
        assert len(events) == 0  # 还没有完整帧

        # 第二块（完成帧）
        second = "lo\"}}\n\n"
        events = parser.push(second)
        assert len(events) == 1
        assert "Hello" in events[0].data

    def test_ignore_ping_and_done(self):
        """忽略 ping 事件和 [DONE] 标记"""
        parser = SseParser()

        payload = ": keepalive\nevent: ping\ndata: {\"type\":\"ping\"}\n\nevent: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\ndata: [DONE]\n\n"
        events = parser.push(payload)

        assert len(events) == 1  # 只有 message_delta，ping 和 [DONE] 被忽略
        assert events[0].event == "message_delta"

    def test_multiple_frames(self):
        """解析多个帧"""
        parser = SseParser()

        payload = "data: frame1\n\ndata: frame2\n\ndata: frame3\n\n"
        events = parser.push(payload)

        assert len(events) == 3
        assert events[0].data == "frame1"
        assert events[1].data == "frame2"
        assert events[2].data == "frame3"

    def test_finish_remaining_buffer(self):
        """完成时处理剩余缓冲区"""
        parser = SseParser()

        # 没有分隔符的不完整帧
        parser.push("data: incomplete")
        events = parser.finish()

        # finish 会尝试解析剩余缓冲区（即使没有分隔符）
        assert len(events) == 1
        assert events[0].data == "incomplete"

        # 完整帧应该在 push 时就被解析
        parser2 = SseParser()
        events = parser2.push("data: final\n\n")
        assert len(events) == 1
        events = parser2.finish()
        assert len(events) == 0


class TestStreamState:
    """测试流状态"""

    def test_anthropic_message_start(self):
        """处理 Anthropic message_start 事件"""
        state = StreamState("claude-sonnet-4-6")

        event_data = {
            "type": "message_start",
            "message": {
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "model": "claude-sonnet-4-6",
            }
        }

        events = state.ingest_anthropic(event_data)
        assert len(events) == 1
        assert events[0].event_type == "message_start"
        assert events[0].data["id"] == "msg_123"

    def test_anthropic_content_block_delta(self):
        """处理 Anthropic content_block_delta 事件"""
        state = StreamState("claude-sonnet-4-6")
        state.message_started = True

        event_data = {
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "text_delta",
                "text": "Hello"
            }
        }

        events = state.ingest_anthropic(event_data)
        assert len(events) == 1
        assert events[0].event_type == "content_block_delta"
        assert events[0].data["content"] == "Hello"

    def test_openai_chunk(self):
        """处理 OpenAI 流式块"""
        state = StreamState("gpt-4o")

        chunk_data = {
            "id": "chatcmpl_123",
            "model": "gpt-4o",
            "choices": [{
                "delta": {"content": "Hello"},
                "finish_reason": None
            }]
        }

        events = state.ingest_openai(chunk_data)
        assert len(events) >= 2  # message_start + content_block_start + delta
        assert events[0].event_type == "message_start"

    def test_openai_with_reasoning(self):
        """处理 OpenAI reasoning_content"""
        state = StreamState("gpt-4o")

        # 先发送 reasoning
        chunk1 = {
            "id": "chatcmpl_123",
            "model": "gpt-4o",
            "choices": [{
                "delta": {"reasoning_content": "Thinking..."},
                "finish_reason": None
            }]
        }
        events = state.ingest_openai(chunk1)

        # 应该有 thinking block
        assert any(e.data.get("block_type") == "thinking" for e in events)

        # 然后发送内容
        chunk2 = {
            "id": "chatcmpl_123",
            "model": "gpt-4o",
            "choices": [{
                "delta": {"content": "Answer"},
                "finish_reason": None
            }]
        }
        events = state.ingest_openai(chunk2)

        # 应该关闭 thinking block 并开始 text block
        assert any(e.event_type == "content_block_stop" for e in events)

    def test_finish(self):
        """完成流处理"""
        state = StreamState("gpt-4o")
        state.message_started = True
        state.text_started = True

        events = state.finish()
        assert len(events) >= 2  # message_delta + message_stop
        assert events[-1].event_type == "message_stop"


class TestCallbackStream:
    """测试回调流"""

    def test_on_chunk_callback(self):
        """测试 on_chunk 回调"""
        chunks_received: list[str] = []

        stream = CallbackStream(on_chunk=lambda c: chunks_received.append(c))
        state = StreamState("claude-sonnet-4-6")

        sse_event = SseEvent(
            event="content_block_delta",
            data='{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}'
        )

        stream.push_sse_event(sse_event, state, "anthropic")

        assert len(chunks_received) == 1
        assert chunks_received[0] == "Hello"

    def test_on_event_callback(self):
        """测试 on_event 回调"""
        events_received: list[StreamEvent] = []

        stream = CallbackStream(on_event=lambda e: events_received.append(e))
        state = StreamState("claude-sonnet-4-6")

        sse_event = SseEvent(
            event="message_start",
            data='{"type":"message_start","message":{"id":"msg_123","model":"claude-sonnet-4-6"}}'
        )

        stream.push_sse_event(sse_event, state, "anthropic")

        assert len(events_received) == 1
        assert events_received[0].event_type == "message_start"

    def test_abort(self):
        """测试中断"""
        chunks_received: list[str] = []

        stream = CallbackStream(on_chunk=lambda c: chunks_received.append(c))
        state = StreamState("claude-sonnet-4-6")

        # 中断
        stream.abort()

        sse_event = SseEvent(
            event="content_block_delta",
            data='{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}'
        )

        events = stream.push_sse_event(sse_event, state, "anthropic")

        # 中断后不应该有事件
        assert len(events) == 0
        assert len(chunks_received) == 0

    def test_is_aborted(self):
        """测试中断检查"""
        stream = CallbackStream()
        assert not stream.is_aborted()

        stream.abort()
        assert stream.is_aborted()


class TestIntegration:
    """集成测试"""

    def test_full_anthropic_stream(self):
        """完整的 Anthropic 流处理"""
        parser = SseParser()
        state = StreamState("claude-sonnet-4-6")
        chunks: list[str] = []

        stream = CallbackStream(on_chunk=lambda c: chunks.append(c))

        # 模拟 Anthropic 流式响应
        frames = [
            "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"model\":\"claude-sonnet-4-6\"}}\n\n",
            "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\"}}\n\n",
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n",
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" world\"}}\n\n",
            "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
            "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n",
            "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
        ]

        for frame in frames:
            sse_events = parser.push(frame)
            for sse_event in sse_events:
                stream.push_sse_event(sse_event, state, "anthropic")

        # 验证收集的文本
        assert chunks == ["Hello", " world"]

    def test_full_openai_stream(self):
        """完整的 OpenAI 流处理"""
        parser = SseParser()
        state = StreamState("gpt-4o")
        chunks: list[str] = []

        stream = CallbackStream(on_chunk=lambda c: chunks.append(c))

        # 模拟 OpenAI 流式响应
        frames = [
            "data: {\"id\":\"chatcmpl_1\",\"model\":\"gpt-4o\",\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            "data: {\"id\":\"chatcmpl_1\",\"model\":\"gpt-4o\",\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\n",
            "data: {\"id\":\"chatcmpl_1\",\"model\":\"gpt-4o\",\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n",
        ]

        for frame in frames:
            sse_events = parser.push(frame)
            for sse_event in sse_events:
                stream.push_sse_event(sse_event, state, "openai")

        # 验证收集的文本
        assert chunks == ["Hello", " world"]