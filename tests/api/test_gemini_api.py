"""真实 API 调用验证 - Gemini

测试 Google Gemini API 的真实调用。
需要配置 GEMINI_API_KEY。

运行: pytest tests/api/test_gemini_api.py -v -s
"""

import pytest
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


def has_gemini_key():
    """检查是否配置了 Gemini API Key"""
    return bool(os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY"))


skip_no_gemini = pytest.mark.skipif(
    not has_gemini_key(),
    reason="GEMINI_API_KEY / GOOGLE_API_KEY not set"
)


@skip_no_gemini
@pytest.mark.api
class TestGeminiAPI:
    """Gemini API 真实调用测试"""

    @pytest.mark.asyncio
    async def test_gemini_simple_call(self):
        """测试简单 API 调用"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="gemini",
            api_key=os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY"),
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
    async def test_gemini_model_pro(self):
        """测试 Gemini Pro 模型"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="gemini",
            api_key=os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY"),
            model="gemini-1.5-pro",
        )

        messages = [Message.user("Reply with exactly: 'PRO'")]
        response = await client.chat(messages=messages, max_tokens=20)

        assert response is not None
        assert len(response.content) > 0, "Should have a response"
        print(f"\n[Gemini Pro Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_gemini_model_flash(self):
        """测试 Gemini Flash 模型"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="gemini",
            api_key=os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY"),
            model="gemini-1.5-flash",
        )

        messages = [Message.user("Reply with exactly: 'FLASH'")]
        response = await client.chat(messages=messages, max_tokens=20)

        assert response is not None
        assert len(response.content) > 0, "Should have a response"
        print(f"\n[Gemini Flash Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_gemini_streaming(self):
        """测试流式响应"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="gemini",
            api_key=os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY"),
        )

        messages = [Message.user("Count from 1 to 5, one number per line.")]
        chunks = []
        async for chunk in client.chat_stream(messages=messages, max_tokens=50):
            if chunk.content:
                chunks.append(chunk.content)

        full_content = "".join(chunks)
        assert len(chunks) > 0, "Should receive multiple chunks"
        print(f"\n[Streamed Content]: {full_content}")

    def test_gemini_invalid_key(self):
        """测试无效 API key 错误处理"""
        from continuum_sdk.llm import LlmClient, Message, LlmError

        client = LlmClient.for_provider(
            provider="gemini",
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
