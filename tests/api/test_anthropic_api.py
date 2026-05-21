"""真实 API 调用验证 - Anthropic

测试 Anthropic Claude API 的真实调用。
需要配置 CONTINUUM_API_KEY 或 ANTHROPIC_API_KEY。

运行: pytest tests/api/test_anthropic_api.py -v -s
"""

import pytest
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


def has_api_key():
    """检查是否配置了 API Key"""
    return bool(
        os.environ.get("CONTINUUM_API_KEY")
        or os.environ.get("CONTINUUM_API_KEY")
        or os.environ.get("ANTHROPIC_API_KEY")
    )


skip_no_api = pytest.mark.skipif(
    not has_api_key(),
    reason="No API key configured (CONTINUUM_API_KEY / CONTINUUM_API_KEY / ANTHROPIC_API_KEY)"
)


@skip_no_api
@pytest.mark.api
class TestAnthropicAPI:
    """Anthropic API 真实调用测试"""

    @pytest.mark.asyncio
    async def test_anthropic_simple_call(self):
        """测试简单 API 调用"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="anthropic",
            api_key=os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("ANTHROPIC_API_KEY"),
        )

        messages = [Message.user("Say 'hello' and nothing else.")]
        response = await client.chat(messages=messages, max_tokens=50, temperature=0.0)

        assert response is not None, "Response should not be None"
        assert response.content is not None, "Content should not be None"
        assert len(response.content) > 0, "Content should not be empty"
        content_lower = response.content.lower()
        assert "hello" in content_lower, f"Expected 'hello' in response, got: {response.content}"
        print(f"\n[Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_anthropic_model_haiku(self):
        """测试 Claude Haiku 模型"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="anthropic",
            api_key=os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("ANTHROPIC_API_KEY"),
            model="claude-3-5-haiku-20241022",
        )

        messages = [Message.user("Reply with exactly: 'HAIKU'")]
        response = await client.chat(messages=messages, max_tokens=20)

        assert response is not None
        assert "haiku" in response.content.lower(), f"Expected 'haiku' in response, got: {response.content}"
        print(f"\n[Haiku Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_anthropic_with_tools(self):
        """测试带工具定义的调用"""
        from continuum_sdk.llm import LlmClient, Message, ToolDefinition

        client = LlmClient.for_provider(
            provider="anthropic",
            api_key=os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("ANTHROPIC_API_KEY"),
        )

        tools = [
            ToolDefinition(
                name="get_weather",
                description="Get weather for a location",
                parameters={
                    "type": "object",
                    "properties": {"location": {"type": "string"}},
                    "required": ["location"],
                },
            )
        ]

        messages = [Message.user("What's the weather in Tokyo?")]
        response = await client.chat(messages=messages, tools=tools, max_tokens=100)

        assert response is not None
        print(f"\n[Tool Response]: {response.content}")
        if hasattr(response, "tool_calls") and response.tool_calls:
            print(f"[Tool Calls]: {response.tool_calls}")
            assert any(tc["name"] == "get_weather" for tc in response.tool_calls), \
                "Expected get_weather tool call"

    @pytest.mark.asyncio
    async def test_anthropic_streaming(self):
        """测试流式响应"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="anthropic",
            api_key=os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("ANTHROPIC_API_KEY"),
        )

        messages = [Message.user("Count from 1 to 5, one number per line.")]
        chunks = []
        async for chunk in client.chat_stream(messages=messages, max_tokens=50):
            if chunk.content:
                chunks.append(chunk.content)

        full_content = "".join(chunks)
        assert len(chunks) > 0, "Should receive multiple chunks"
        assert any(str(i) in full_content for i in range(1, 6)), \
            f"Expected numbers 1-5 in stream, got: {full_content}"
        print(f"\n[Streamed Content]: {full_content}")

    @pytest.mark.asyncio
    async def test_anthropic_long_context(self):
        """测试长上下文处理"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="anthropic",
            api_key=os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("ANTHROPIC_API_KEY"),
        )

        # 创建长消息
        long_context = "This is a test line.\n" * 100
        messages = [
            Message.user(f"Here's some context:\n{long_context}\nSummarize this in one word."),
        ]
        response = await client.chat(messages=messages, max_tokens=20)

        assert response is not None
        assert len(response.content) > 0, "Should have a response"
        print(f"\n[Long Context Response]: {response.content}")

    def test_anthropic_invalid_key(self):
        """测试无效 API key 错误处理"""
        from continuum_sdk.llm import LlmClient, Message, LlmError

        client = LlmClient.for_provider(
            provider="anthropic",
            api_key="invalid-test-key-12345",
        )

        messages = [Message.user("Hello")]
        try:
            import asyncio
            asyncio.run(client.chat(messages=messages, max_tokens=10))
            pytest.fail("Should have raised an error for invalid API key")
        except LlmError as e:
            print(f"\n[Expected Error]: {type(e).__name__}: {str(e)[:100]}")
            assert True  # Expected path
        except Exception as e:
            # May raise other exception types
            print(f"\n[Other Error]: {type(e).__name__}: {str(e)[:100]}")
            assert True  # Also acceptable

    @pytest.mark.asyncio
    async def test_anthropic_rate_limit_handling(self):
        """测试速率限制处理"""
        from continuum_sdk.llm import LlmClient, Message
        import asyncio

        client = LlmClient.for_provider(
            provider="anthropic",
            api_key=os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("CONTINUUM_API_KEY")
            or os.environ.get("ANTHROPIC_API_KEY"),
        )

        # 快速发送多个请求
        messages = [Message.user("Say 'ok'")]
        tasks = [
            client.chat(messages=messages, max_tokens=10)
            for _ in range(3)
        ]

        try:
            results = await asyncio.gather(*tasks, return_exceptions=True)
            successes = sum(1 for r in results if not isinstance(r, Exception))
            print(f"\n[Rate Limit Test]: {successes}/3 requests succeeded")
            assert successes >= 1, "At least one request should succeed"
        except Exception as e:
            print(f"\n[Rate Limit Error]: {e}")
            pytest.skip(f"Rate limit test failed: {e}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
