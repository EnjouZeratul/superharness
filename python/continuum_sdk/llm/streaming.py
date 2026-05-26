"""SSE Parser

Server-Sent Events 解析器，支持跨 chunk 的帧边界处理。

参考 Rust 实现: layer1/src/streaming.rs

Features:
    - SSE 帧解析（跨 chunk）
    - Anthropic/OpenAI 流式格式支持
    - on_chunk 回调机制
    - abort 中断支持
"""

from __future__ import annotations

import json
from collections import deque
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable


@dataclass
class SseEvent:
    """SSE 事件"""

    event: str | None = None
    data: str = ""


class SseParser:
    """SSE 解析器

    解析 Server-Sent Events 格式的流式数据。
    支持跨 chunk 的帧边界处理。
    """

    def __init__(self, provider: str | None = None, model: str | None = None):
        """初始化 SSE 解析器

        Args:
            provider: 提供商名称（用于错误报告）
            model: 模型名称（用于错误报告）
        """
        self._buffer = ""
        self._provider = provider
        self._model = model

    def push(self, chunk: str) -> list[SseEvent]:
        """推送数据块并解析出完整的事件

        Args:
            chunk: 数据块字符串

        Returns:
            解析出的 SSE 事件列表
        """
        self._buffer += chunk
        events: list[SseEvent] = []

        while True:
            frame = self._next_frame()
            if frame is None:
                break

            event = self._parse_frame(frame)
            if event is not None:
                events.append(event)

        return events

    def finish(self) -> list[SseEvent]:
        """完成解析，处理缓冲区中剩余的数据

        Returns:
            剩余的 SSE 事件列表
        """
        if not self._buffer:
            return []

        trailing = self._buffer
        self._buffer = ""

        event = self._parse_frame(trailing)
        if event is not None:
            return [event]
        return []

    def _next_frame(self) -> str | None:
        """提取下一个帧"""
        # 查找 \n\n 或 \r\n\r\n 分隔符
        separator_pos = None
        separator_len = 0

        # 先查找 \n\n
        if "\n\n" in self._buffer:
            pos = self._buffer.index("\n\n")
            separator_pos = pos
            separator_len = 2
        # 再查找 \r\n\r\n
        elif "\r\n\r\n" in self._buffer:
            pos = self._buffer.index("\r\n\r\n")
            separator_pos = pos
            separator_len = 4

        if separator_pos is None:
            return None

        frame = self._buffer[:separator_pos]
        self._buffer = self._buffer[separator_pos + separator_len :]
        return frame

    def _parse_frame(self, frame: str) -> SseEvent | None:
        """解析帧"""
        trimmed = frame.strip()
        if not trimmed:
            return None

        data_lines: list[str] = []
        event_name: str | None = None

        for line in trimmed.split("\n"):
            # 跳过注释行
            if line.startswith(":"):
                continue

            # 解析 event 字段
            if line.startswith("event:"):
                event_name = line[6:].strip()
                continue

            # 解析 data 字段
            if line.startswith("data:"):
                data_lines.append(line[5:].lstrip())

        # 跳过 ping 事件
        if event_name == "ping":
            return None

        if not data_lines:
            return None

        payload = "\n".join(data_lines)

        # 处理 [DONE] 标记（OpenAI 格式）
        if payload == "[DONE]":
            return None

        return SseEvent(event=event_name, data=payload)


class ContentBlockType(Enum):
    """内容块类型"""

    TEXT = "text"
    THINKING = "thinking"
    TOOL_USE = "tool_use"


@dataclass
class ContentBlockStart:
    """内容块开始"""

    index: int
    block_type: ContentBlockType
    tool_id: str | None = None
    tool_name: str | None = None


@dataclass
class ContentBlockDelta:
    """内容块增量"""

    index: int
    delta_type: str  # "text", "thinking", "tool_input"
    content: str


@dataclass
class ContentBlockStop:
    """内容块结束"""

    index: int


@dataclass
class StreamUsage:
    """流式用量"""

    input_tokens: int = 0
    output_tokens: int = 0


@dataclass
class StreamEvent:
    """统一的流式事件"""

    event_type: str  # message_start, content_block_start, content_block_delta, content_block_stop, message_delta, message_stop
    data: dict[str, Any] = field(default_factory=dict)


class StreamState:
    """流式响应状态

    处理 Anthropic 和 OpenAI 的流式事件，转换为统一格式。
    """

    def __init__(self, model: str):
        """初始化流状态

        Args:
            model: 模型名称
        """
        self.model = model
        self.message_started = False
        self.text_started = False
        self.text_finished = False
        self.thinking_started = False
        self.thinking_finished = False
        self.finished = False
        self.stop_reason: str | None = None
        self.usage: StreamUsage | None = None
        self.tool_index_offset = 0

    def ingest_anthropic(self, event_data: dict[str, Any]) -> list[StreamEvent]:
        """处理 Anthropic 事件"""
        events: list[StreamEvent] = []
        event_type = event_data.get("type", "")

        if event_type == "message_start":
            message = event_data.get("message", {})
            if not self.message_started:
                self.message_started = True
                events.append(StreamEvent(
                    event_type="message_start",
                    data={"id": message.get("id", ""), "model": message.get("model", self.model)}
                ))

        elif event_type == "content_block_start":
            index = event_data.get("index", 0)
            content_block = event_data.get("content_block", {})
            block_type_str = content_block.get("type", "text")

            block_type = ContentBlockType.TEXT
            tool_id = None
            tool_name = None

            if block_type_str == "text":
                block_type = ContentBlockType.TEXT
            elif block_type_str == "thinking":
                block_type = ContentBlockType.THINKING
            elif block_type_str == "tool_use":
                block_type = ContentBlockType.TOOL_USE
                tool_id = content_block.get("id")
                tool_name = content_block.get("name")

            events.append(StreamEvent(
                event_type="content_block_start",
                data={"index": index, "block_type": block_type.value, "tool_id": tool_id, "tool_name": tool_name}
            ))

        elif event_type == "content_block_delta":
            index = event_data.get("index", 0)
            delta = event_data.get("delta", {})
            delta_type = delta.get("type", "text_delta")

            content = ""
            if delta_type == "text_delta":
                content = delta.get("text", "")
            elif delta_type == "thinking_delta":
                content = delta.get("thinking", "")
            elif delta_type == "input_json_delta":
                content = delta.get("partial_json", "")

            events.append(StreamEvent(
                event_type="content_block_delta",
                data={"index": index, "delta_type": delta_type, "content": content}
            ))

        elif event_type == "content_block_stop":
            index = event_data.get("index", 0)
            events.append(StreamEvent(
                event_type="content_block_stop",
                data={"index": index}
            ))

        elif event_type == "message_delta":
            delta = event_data.get("delta", {})
            usage_data = event_data.get("usage", {})

            self.stop_reason = delta.get("stop_reason")
            self.usage = StreamUsage(
                input_tokens=usage_data.get("input_tokens", 0),
                output_tokens=usage_data.get("output_tokens", 0)
            )

            events.append(StreamEvent(
                event_type="message_delta",
                data={"stop_reason": self.stop_reason, "usage": {"input_tokens": self.usage.input_tokens, "output_tokens": self.usage.output_tokens}}
            ))

        elif event_type == "message_stop":
            events.append(StreamEvent(event_type="message_stop"))

        return events

    def ingest_openai(self, chunk_data: dict[str, Any]) -> list[StreamEvent]:
        """处理 OpenAI 事件"""
        events: list[StreamEvent] = []

        if not self.message_started:
            self.message_started = True
            events.append(StreamEvent(
                event_type="message_start",
                data={"id": chunk_data.get("id", ""), "model": chunk_data.get("model", self.model)}
            ))

        # 处理 usage
        usage_data = chunk_data.get("usage")
        if usage_data:
            self.usage = StreamUsage(
                input_tokens=usage_data.get("prompt_tokens", 0),
                output_tokens=usage_data.get("completion_tokens", 0)
            )

        choices = chunk_data.get("choices", [])
        for choice in choices:
            delta = choice.get("delta", {})

            # 处理 reasoning_content（思考内容）
            reasoning = delta.get("reasoning_content")
            if reasoning:
                if not self.thinking_started:
                    self.thinking_started = True
                    events.append(StreamEvent(
                        event_type="content_block_start",
                        data={"index": 0, "block_type": "thinking"}
                    ))
                events.append(StreamEvent(
                    event_type="content_block_delta",
                    data={"index": 0, "delta_type": "thinking_delta", "content": reasoning}
                ))

            # 处理常规内容
            content = delta.get("content")
            if content:
                # 如果之前有思考块，先关闭它
                if self.thinking_started and not self.thinking_finished:
                    self.thinking_finished = True
                    events.append(StreamEvent(
                        event_type="content_block_stop",
                        data={"index": 0}
                    ))

                text_index = 1 if self.thinking_started else 0
                if not self.text_started:
                    self.text_started = True
                    events.append(StreamEvent(
                        event_type="content_block_start",
                        data={"index": text_index, "block_type": "text"}
                    ))
                events.append(StreamEvent(
                    event_type="content_block_delta",
                    data={"index": text_index, "delta_type": "text_delta", "content": content}
                ))

            # 处理结束原因
            finish_reason = choice.get("finish_reason")
            if finish_reason:
                self.stop_reason = self._normalize_finish_reason(finish_reason)

        return events

    def _normalize_finish_reason(self, reason: str) -> str:
        """规范化 OpenAI 结束原因"""
        if reason == "stop":
            return "end_turn"
        elif reason == "tool_calls":
            return "tool_use"
        return reason

    def finish(self) -> list[StreamEvent]:
        """完成流处理"""
        if self.finished:
            return []
        self.finished = True

        events: list[StreamEvent] = []

        # 关闭思考块
        if self.thinking_started and not self.thinking_finished:
            self.thinking_finished = True
            events.append(StreamEvent(event_type="content_block_stop", data={"index": 0}))

        # 关闭文本块
        if self.text_started and not self.text_finished:
            self.text_finished = True
            text_index = 1 if self.thinking_started else 0
            events.append(StreamEvent(event_type="content_block_stop", data={"index": text_index}))

        # 发送消息增量
        if self.message_started:
            events.append(StreamEvent(
                event_type="message_delta",
                data={
                    "stop_reason": self.stop_reason or "end_turn",
                    "usage": {"input_tokens": self.usage.input_tokens if self.usage else 0, "output_tokens": self.usage.output_tokens if self.usage else 0}
                }
            ))
            events.append(StreamEvent(event_type="message_stop"))

        return events


class CallbackStream:
    """带回调的流式响应

    支持 on_chunk 回调机制和 abort 中断。
    """

    def __init__(
        self,
        on_chunk: Callable[[str], None] | None = None,
        on_event: Callable[[StreamEvent], None] | None = None,
    ):
        """初始化回调流

        Args:
            on_chunk: 文本增量回调函数
            on_event: 事件回调函数
        """
        self._on_chunk = on_chunk
        self._on_event = on_event
        self._abort_flag = False
        self._pending: deque[StreamEvent] = deque()

    def abort(self) -> None:
        """请求中断"""
        self._abort_flag = True

    def is_aborted(self) -> bool:
        """检查是否已中断"""
        return self._abort_flag

    def push_sse_event(self, sse_event: SseEvent, state: StreamState, provider: str) -> list[StreamEvent]:
        """推送 SSE 事件并处理

        Args:
            sse_event: SSE 事件
            state: 流状态
            provider: 提供商类型

        Returns:
            处理后的流式事件列表
        """
        if self._abort_flag:
            return []

        try:
            data = json.loads(sse_event.data)
        except json.JSONDecodeError:
            return []

        events: list[StreamEvent] = []
        if provider.lower() == "anthropic":
            events = state.ingest_anthropic(data)
        else:
            events = state.ingest_openai(data)

        # 触发回调
        for event in events:
            if self._on_event:
                self._on_event(event)

            if event.event_type == "content_block_delta":
                content = event.data.get("content", "")
                if content and self._on_chunk:
                    self._on_chunk(content)

        return events

    def finish(self, state: StreamState) -> list[StreamEvent]:
        """完成流处理

        Args:
            state: 流状态

        Returns:
            完成事件列表
        """
        if self._abort_flag:
            return []

        events = state.finish()

        for event in events:
            if self._on_event:
                self._on_event(event)

            if event.event_type == "content_block_delta":
                content = event.data.get("content", "")
                if content and self._on_chunk:
                    self._on_chunk(content)

        return events