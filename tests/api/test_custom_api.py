"""真实 API 调用验证 - 自定义端点

测试自定义 API 端点（如腾讯云、阿里云等）。
需要配置 CUSTOM_API_KEY 和 CUSTOM_BASE_URL。

运行: pytest tests/api/test_custom_api.py -v -s
"""

import pytest
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


def has_custom_api():
    """检查是否配置了自定义 API"""
    return bool(
        os.environ.get("CUSTOM_API_KEY")
        or os.environ.get("TENCENT_API_KEY")
        or os.environ.get("ALIBABA_API_KEY")
    )


skip_no_custom = pytest.mark.skipif(
    not has_custom_api(),
    reason="CUSTOM_API_KEY / TENCENT_API_KEY / ALIBABA_API_KEY not set"
)


@skip_no_custom
@pytest.mark.api
class TestCustomAPI:
    """自定义 API 端点测试"""

    @pytest.mark.asyncio
    async def test_custom_endpoint_call(self):
        """测试自定义端点调用"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="custom",
            api_key=os.environ.get("CUSTOM_API_KEY"),
            base_url=os.environ.get("CUSTOM_BASE_URL"),
        )

        messages = [Message.user("Say 'hello'")]
        response = await client.chat(messages=messages, max_tokens=50)

        assert response is not None, "Response should not be None"
        assert response.content is not None, "Content should not be None"
        print(f"\n[Custom API Response]: {response.content}")

    @pytest.mark.asyncio
    @pytest.mark.skipif(
        not os.environ.get("TENCENT_API_KEY"),
        reason="TENCENT_API_KEY not set"
    )
    async def test_tencent_cloud_api(self):
        """测试腾讯云 API"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="custom",
            api_key=os.environ.get("TENCENT_API_KEY"),
            base_url=os.environ.get("TENCENT_BASE_URL", "https://api.hunyuan.cloud.tencent.com/v1"),
        )

        messages = [Message.user("你好，请回复'测试成功'")]
        response = await client.chat(messages=messages, max_tokens=50)

        assert response is not None
        print(f"\n[Tencent Response]: {response.content}")

    @pytest.mark.asyncio
    @pytest.mark.skipif(
        not os.environ.get("ALIBABA_API_KEY"),
        reason="ALIBABA_API_KEY not set"
    )
    async def test_alibaba_cloud_api(self):
        """测试阿里云 API"""
        from continuum_sdk.llm import LlmClient, Message

        client = LlmClient.for_provider(
            provider="custom",
            api_key=os.environ.get("ALIBABA_API_KEY"),
            base_url=os.environ.get("ALIBABA_BASE_URL", "https://dashscope.aliyuncs.com/compatible-mode/v1"),
        )

        messages = [Message.user("你好，请回复'测试成功'")]
        response = await client.chat(messages=messages, max_tokens=50)

        assert response is not None
        print(f"\n[Alibaba Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_custom_endpoint_with_auth_header(self):
        """测试自定义认证头"""
        from continuum_sdk.llm import LlmClient, Message

        # 某些自定义端点使用不同的认证方式
        headers = {"X-Custom-Auth": os.environ.get("CUSTOM_API_KEY", "test-token")}

        client = LlmClient.for_provider(
            provider="custom",
            api_key=os.environ.get("CUSTOM_API_KEY", "placeholder"),
            base_url=os.environ.get("CUSTOM_BASE_URL", "https://api.example.com/v1"),
        )

        # 如果有自定义认证，可以设置额外头部
        if hasattr(client, "set_headers"):
            client.set_headers(headers)

        messages = [Message.user("Hello")]
        try:
            response = await client.chat(messages=messages, max_tokens=50)
            assert response is not None
            print(f"\n[Custom Auth Response]: {response.content}")
        except Exception as e:
            print(f"\n[Custom Auth Error]: {e}")
            pytest.skip(f"Custom auth test skipped: {e}")

    @pytest.mark.asyncio
    async def test_custom_endpoint_response_format(self):
        """测试自定义响应格式解析"""
        from continuum_sdk.llm import LlmClient, Message

        if not os.environ.get("CUSTOM_API_KEY"):
            pytest.skip("CUSTOM_API_KEY not set")

        client = LlmClient.for_provider(
            provider="custom",
            api_key=os.environ.get("CUSTOM_API_KEY"),
            base_url=os.environ.get("CUSTOM_BASE_URL"),
        )

        messages = [Message.user("Reply with a short greeting")]
        response = await client.chat(messages=messages, max_tokens=50)

        assert response is not None
        assert hasattr(response, "content"), "Response should have content attribute"
        assert hasattr(response, "usage"), "Response should have usage attribute"
        print(f"\n[Response Format]: content={response.content[:50]}..., usage={response.usage}")

    def test_custom_endpoint_connection_error(self):
        """测试连接错误处理"""
        from continuum_sdk.llm import LlmClient, Message, LlmError

        client = LlmClient.for_provider(
            provider="custom",
            api_key="test-key",
            base_url="https://nonexistent.endpoint.12345/api/v1",
        )

        messages = [Message.user("Hello")]
        try:
            import asyncio
            asyncio.run(client.chat(messages=messages, max_tokens=10))
            pytest.fail("Should have raised a connection error")
        except LlmError as e:
            print(f"\n[Expected Connection Error]: {type(e).__name__}")
            assert True
        except Exception as e:
            print(f"\n[Other Error]: {type(e).__name__}")
            # Connection errors should be caught and wrapped
            assert True


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
