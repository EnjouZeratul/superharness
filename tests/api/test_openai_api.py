"""真实 API 调用验证 - OpenAI

测试 OpenAI GPT API 的真实调用。
需要配置 OPENAI_API_KEY。

运行: pytest tests/api/test_openai_api.py -v -s
"""

import pytest
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


def has_openai_key():
    """检查是否配置了 OpenAI API Key"""
    return bool(os.environ.get("OPENAI_API_KEY"))


skip_no_openai = pytest.mark.skipif(
    not has_openai_key(),
    reason="OPENAI_API_KEY not set"
)


@skip_no_openai
@pytest.mark.api
class TestOpenAIAPI:
    """OpenAI API 真实调用测试"""

    @pytest.mark.asyncio
    async def test_openai_simple_call(self):
        """测试简单 API 调用"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="openai",
            api_key=os.environ.get("OPENAI_API_KEY"),
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
    async def test_openai_model_gpt4(self):
        """测试 GPT-4 模型"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="openai",
            api_key=os.environ.get("OPENAI_API_KEY"),
            model="gpt-4",
        )

        messages = [Message.user("Reply with exactly: 'GPT4'")]
        response = await client.chat(messages=messages, max_tokens=20)

        assert response is not None
        assert "gpt" in response.content.lower() or "4" in response.content, \
            f"Expected GPT-4 response, got: {response.content}"
        print(f"\n[GPT-4 Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_openai_model_gpt35(self):
        """测试 GPT-3.5 模型"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="openai",
            api_key=os.environ.get("OPENAI_API_KEY"),
            model="gpt-3.5-turbo",
        )

        messages = [Message.user("Reply with exactly: 'GPT35'")]
        response = await client.chat(messages=messages, max_tokens=20)

        assert response is not None
        assert len(response.content) > 0, "Should have a response"
        print(f"\n[GPT-3.5 Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_openai_with_tools(self):
        """测试带工具定义的调用"""
        from continuum_sdk.llm import LlmClient, Message, ToolDefinition

        client = LlmClient.for_provider(
            provider="openai",
            api_key=os.environ.get("OPENAI_API_KEY"),
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

    @pytest.mark.asyncio
    async def test_openai_streaming(self):
        """测试流式响应"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="openai",
            api_key=os.environ.get("OPENAI_API_KEY"),
        )

        messages = [Message.user("Count from 1 to 5, one number per line.")]
        chunks = []
        async for chunk in client.chat_stream(messages=messages, max_tokens=50):
            if chunk.content:
                chunks.append(chunk.content)

        full_content = "".join(chunks)
        assert len(chunks) > 0, "Should receive multiple chunks"
        print(f"\n[Streamed Content]: {full_content}")

    @pytest.mark.asyncio
    async def test_openai_custom_base_url(self):
        """测试自定义 base URL（用于 Azure OpenAI 或代理）"""
        from continuum_sdk.llm import LlmClient, Message

        # 使用默认 OpenAI API，但指定自定义 URL 模式
        custom_url = os.environ.get("OPENAI_BASE_URL", "https://api.openai.com/v1")

        client = LlmClient.for_provider(
            provider="openai",
            api_key=os.environ.get("OPENAI_API_KEY"),
            base_url=custom_url,
        )

        messages = [Message.user("Say 'ok'")]
        response = await client.chat(messages=messages, max_tokens=10)

        assert response is not None
        assert len(response.content) > 0
        print(f"\n[Custom URL Response]: {response.content}")

    def test_openai_invalid_key(self):
        """测试无效 API key 错误处理"""
        from continuum_sdk.llm import LlmClient, Message, LlmError

        client = LlmClient.for_provider(
            provider="openai",
            api_key="invalid-test-key-12345",
        )

        messages = [Message.user("Hello")]
        try:
            import asyncio
            asyncio.run(client.chat(messages=messages, max_tokens=10))
            pytest.fail("Should have raised an error for invalid API key")
        except LlmError as e:
            print(f"\n[Expected Error]: {type(e).__name__}: {str(e)[:100]}")
            assert True
        except Exception as e:
            print(f"\n[Other Error]: {type(e).__name__}: {str(e)[:100]}")
            assert True


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
