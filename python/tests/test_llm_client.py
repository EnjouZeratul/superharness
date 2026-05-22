"""
LLM Client Mock 测试

测试 LLM 客户端模块，使用 Mock 模拟 API 响应，不调用真实 API。

覆盖：
- 客户端创建 (LlmClient.for_provider)
- 请求构建 (Message, Tools, 参数)
- 响应处理 (ChatResponse, TokenUsage, StreamChunk)
- 错误处理 (认证、速率限制、超时、无效响应)
"""

import sys
import os
import pytest
import json
from unittest.mock import AsyncMock, MagicMock, patch, Mock
from typing import Dict, Any, List

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from continuum_sdk.llm.client import (
    LlmClient,
    AnthropicClient,
    OpenAIClient,
    GeminiClient,
    CustomClient,
    BaseLlmClient,
)
from continuum_sdk.llm.types import (
    Message,
    MessageRole,
    ChatResponse,
    TokenUsage,
    StreamChunk,
    ToolDefinition,
)
from continuum_sdk.llm.errors import (
    LlmError,
    AuthenticationError,
    RateLimitError,
    NetworkError,
    TimeoutError,
    InvalidResponseError,
    ModelNotFoundError,
    InsufficientQuotaError,
    ContentFilterError,
    classify_http_error,
)


# ==================== 客户端创建测试 ====================

class TestLlmClientFactory:
    """LlmClient.for_provider 工厂方法测试"""

    def test_create_anthropic_client(self):
        """测试创建 Anthropic 客户端"""
        client = LlmClient.for_provider("anthropic", api_key="test-key")
        assert isinstance(client, AnthropicClient)
        assert client.api_key == "test-key"
        assert client.provider == "anthropic"
        assert client.default_model == "claude-sonnet-4-6"

    def test_create_openai_client(self):
        """测试创建 OpenAI 客户端"""
        client = LlmClient.for_provider("openai", api_key="test-key")
        assert isinstance(client, OpenAIClient)
        assert client.api_key == "test-key"
        assert client.provider == "openai"
        assert client.default_model == "gpt-4"

    def test_create_gemini_client(self):
        """测试创建 Gemini 客户端"""
        client = LlmClient.for_provider("gemini", api_key="test-key")
        assert isinstance(client, GeminiClient)
        assert client.api_key == "test-key"
        assert client.provider == "gemini"

    def test_create_google_alias(self):
        """测试 google 作为 gemini 的别名"""
        client = LlmClient.for_provider("google", api_key="test-key")
        assert isinstance(client, GeminiClient)

    def test_create_custom_client(self):
        """测试创建自定义端点客户端"""
        client = LlmClient.for_provider(
            "custom",
            api_key="test-key",
            base_url="https://custom.api.com/v1"
        )
        assert isinstance(client, CustomClient)
        assert client.base_url == "https://custom.api.com/v1"

    def test_custom_client_requires_base_url(self):
        """测试自定义客户端必须提供 base_url"""
        with pytest.raises(ValueError, match="base_url is required"):
            LlmClient.for_provider("custom", api_key="test-key")

    def test_unknown_provider_raises_error(self):
        """测试未知提供商抛出错误"""
        with pytest.raises(ValueError, match="Unknown provider"):
            LlmClient.for_provider("unknown", api_key="test-key")

    def test_provider_name_case_insensitive(self):
        """测试提供商名称不区分大小写"""
        client1 = LlmClient.for_provider("ANTHROPIC", api_key="test-key")
        client2 = LlmClient.for_provider("Anthropic", api_key="test-key")
        assert isinstance(client1, AnthropicClient)
        assert isinstance(client2, AnthropicClient)

    def test_custom_model_override(self):
        """测试自定义默认模型"""
        client = LlmClient.for_provider(
            "anthropic",
            api_key="test-key",
            model="claude-opus-4"
        )
        assert client.default_model == "claude-opus-4"

    def test_custom_timeout_and_retries(self):
        """测试自定义超时和重试"""
        client = LlmClient.for_provider(
            "anthropic",
            api_key="test-key",
            timeout=120.0,
            max_retries=5
        )
        assert client.timeout == 120.0
        assert client.max_retries == 5


# ==================== 请求构建测试 ====================

class TestMessageFormatting:
    """消息格式化测试"""

    def test_message_user_factory(self):
        """测试用户消息工厂方法"""
        msg = Message.user("Hello")
        assert msg.role == MessageRole.USER
        assert msg.content == "Hello"

    def test_message_assistant_factory(self):
        """测试助手消息工厂方法"""
        msg = Message.assistant("Hi there!")
        assert msg.role == MessageRole.ASSISTANT
        assert msg.content == "Hi there!"

    def test_message_system_factory(self):
        """测试系统消息工厂方法"""
        msg = Message.system("You are helpful.")
        assert msg.role == MessageRole.SYSTEM
        assert msg.content == "You are helpful."

    def test_anthropic_format(self):
        """测试 Anthropic API 格式"""
        msg = Message.user("Test")
        formatted = msg.to_anthropic_format()
        assert formatted == {"role": "user", "content": "Test"}

    def test_openai_format(self):
        """测试 OpenAI API 格式"""
        msg = Message.user("Test")
        formatted = msg.to_openai_format()
        assert formatted == {"role": "user", "content": "Test"}

    def test_gemini_format(self):
        """测试 Gemini API 格式"""
        msg = Message.assistant("Test")
        formatted = msg.to_gemini_format()
        # Gemini uses "model" instead of "assistant"
        assert formatted["role"] == "model"
        assert formatted["parts"] == [{"text": "Test"}]


class TestToolDefinition:
    """工具定义测试"""

    def test_tool_definition_creation(self):
        """测试工具定义创建"""
        tool = ToolDefinition(
            name="calculator",
            description="Perform calculations",
            parameters={"type": "object", "properties": {"expr": {"type": "string"}}}
        )
        assert tool.name == "calculator"
        assert tool.description == "Perform calculations"

    def test_anthropic_tool_format(self):
        """测试 Anthropic 工具格式"""
        tool = ToolDefinition(
            name="test",
            description="Test tool",
            parameters={"type": "object"}
        )
        formatted = tool.to_anthropic_format()
        assert formatted["name"] == "test"
        assert "input_schema" in formatted

    def test_openai_tool_format(self):
        """测试 OpenAI 工具格式"""
        tool = ToolDefinition(
            name="test",
            description="Test tool",
            parameters={"type": "object"}
        )
        formatted = tool.to_openai_format()
        assert formatted["type"] == "function"
        assert formatted["function"]["name"] == "test"


# ==================== 响应处理测试 ====================

class TestChatResponse:
    """ChatResponse 解析测试"""

    def test_from_anthropic_response(self):
        """测试解析 Anthropic 响应"""
        data = {
            "id": "msg-123",
            "model": "claude-sonnet-4-6",
            "content": [{"type": "text", "text": "Hello!"}],
            "usage": {"input_tokens": 10, "output_tokens": 5},
            "stop_reason": "end_turn"
        }
        response = ChatResponse.from_anthropic(data)
        assert response.content == "Hello!"
        assert response.model == "claude-sonnet-4-6"
        assert response.usage.input_tokens == 10
        assert response.usage.output_tokens == 5

    def test_from_openai_response(self):
        """测试解析 OpenAI 响应"""
        data = {
            "id": "chatcmpl-123",
            "model": "gpt-4",
            "choices": [{
                "message": {"content": "Hi!"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 8, "completion_tokens": 4, "total_tokens": 12}
        }
        response = ChatResponse.from_openai(data)
        assert response.content == "Hi!"
        assert response.model == "gpt-4"
        assert response.usage.input_tokens == 8
        assert response.usage.output_tokens == 4
        assert response.usage.total_tokens == 12

    def test_from_gemini_response(self):
        """测试解析 Gemini 响应"""
        data = {
            "candidates": [{
                "content": {"parts": [{"text": "Greetings!"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 6,
                "candidatesTokenCount": 3,
                "totalTokenCount": 9
            }
        }
        response = ChatResponse.from_gemini(data, "gemini-1.5-pro")
        assert response.content == "Greetings!"
        assert response.model == "gemini-1.5-pro"
        assert response.usage.input_tokens == 6

    def test_token_usage_calculation(self):
        """测试 Token 使用统计自动计算"""
        usage = TokenUsage(input_tokens=100, output_tokens=50)
        assert usage.total_tokens == 150


class TestStreamChunk:
    """StreamChunk 测试"""

    def test_content_chunk(self):
        """测试内容块"""
        chunk = StreamChunk(content="Hello")
        assert chunk.content == "Hello"
        assert chunk.finish_reason is None

    def test_finish_chunk(self):
        """测试结束块"""
        chunk = StreamChunk(finish_reason="stop")
        assert chunk.content == ""
        assert chunk.finish_reason == "stop"


# ==================== Anthropic 客户端测试 ====================

class TestAnthropicClient:
    """Anthropic 客户端测试"""

    @pytest.fixture
    def mock_anthropic_response(self):
        """创建模拟 Anthropic 响应"""
        return {
            "id": "msg-test",
            "model": "claude-sonnet-4-6",
            "content": [{"type": "text", "text": "Test response"}],
            "usage": {"input_tokens": 10, "output_tokens": 5},
            "stop_reason": "end_turn"
        }

    @pytest.mark.asyncio
    async def test_chat_success(self, mock_anthropic_response):
        """测试成功的聊天请求"""
        client = AnthropicClient(api_key="test-key")

        # Mock httpx response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_anthropic_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            messages = [Message.user("Hello")]
            result = await client.chat(messages)

            assert result.content == "Test response"
            assert result.model == "claude-sonnet-4-6"
            mock_post.assert_called_once()

    @pytest.mark.asyncio
    async def test_chat_with_system_prompt(self, mock_anthropic_response):
        """测试带系统提示的请求"""
        client = AnthropicClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_anthropic_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            messages = [Message.user("Hello")]
            await client.chat(messages, system_prompt="Be helpful")

            # 验证请求体包含 system
            call_args = mock_post.call_args
            body = call_args.kwargs['json']
            assert 'system' in body
            assert body['system'] == "Be helpful"

    @pytest.mark.asyncio
    async def test_chat_with_tools(self, mock_anthropic_response):
        """测试带工具的请求"""
        client = AnthropicClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_anthropic_response

        tools = [ToolDefinition(
            name="test_tool",
            description="A test tool",
            parameters={"type": "object"}
        )]

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            messages = [Message.user("Hello")]
            await client.chat(messages, tools=tools)

            call_args = mock_post.call_args
            body = call_args.kwargs['json']
            assert 'tools' in body
            assert len(body['tools']) == 1

    @pytest.mark.asyncio
    async def test_chat_custom_base_url(self, mock_anthropic_response):
        """测试自定义 base_url"""
        client = AnthropicClient(
            api_key="test-key",
            base_url="https://custom.anthropic.com"
        )

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_anthropic_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            await client.chat([Message.user("Hi")])

            call_args = mock_post.call_args
            url = call_args.args[0]
            # 应包含 /v1/messages
            assert "/v1/messages" in url

    @pytest.mark.asyncio
    async def test_headers_correct(self):
        """测试请求头正确"""
        client = AnthropicClient(api_key="sk-test-123")
        headers = client._build_headers()

        assert headers["x-api-key"] == "sk-test-123"
        assert headers["anthropic-version"] == "2023-06-01"
        assert headers["content-type"] == "application/json"


# ==================== OpenAI 客户端测试 ====================

class TestOpenAIClient:
    """OpenAI 客户端测试"""

    @pytest.fixture
    def mock_openai_response(self):
        """创建模拟 OpenAI 响应"""
        return {
            "id": "chatcmpl-test",
            "model": "gpt-4",
            "choices": [{
                "message": {"content": "OpenAI response"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8}
        }

    @pytest.mark.asyncio
    async def test_chat_success(self, mock_openai_response):
        """测试成功的聊天请求"""
        client = OpenAIClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_openai_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            messages = [Message.user("Hello")]
            result = await client.chat(messages)

            assert result.content == "OpenAI response"
            assert result.model == "gpt-4"

    @pytest.mark.asyncio
    async def test_system_prompt_in_messages(self, mock_openai_response):
        """测试系统提示放入消息列表"""
        client = OpenAIClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_openai_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            await client.chat(
                [Message.user("Hi")],
                system_prompt="You are helpful"
            )

            call_args = mock_post.call_args
            body = call_args.kwargs['json']
            # OpenAI 风格：system 是第一条消息
            assert body['messages'][0]['role'] == 'system'

    @pytest.mark.asyncio
    async def test_headers_with_bearer_token(self):
        """测试 Bearer Token 认证头"""
        client = OpenAIClient(api_key="sk-openai")
        headers = client._build_headers()

        assert headers["Authorization"] == "Bearer sk-openai"
        assert headers["Content-Type"] == "application/json"


# ==================== Gemini 客户端测试 ====================

class TestGeminiClient:
    """Gemini 客户端测试"""

    @pytest.fixture
    def mock_gemini_response(self):
        """创建模拟 Gemini 响应"""
        return {
            "candidates": [{
                "content": {"parts": [{"text": "Gemini response"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 4,
                "candidatesTokenCount": 2,
                "totalTokenCount": 6
            }
        }

    @pytest.mark.asyncio
    async def test_chat_success(self, mock_gemini_response):
        """测试成功的聊天请求"""
        client = GeminiClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_gemini_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            messages = [Message.user("Hello")]
            result = await client.chat(messages)

            assert result.content == "Gemini response"

    @pytest.mark.asyncio
    async def test_api_key_in_url(self, mock_gemini_response):
        """测试 API Key 在 URL 中"""
        client = GeminiClient(api_key="gemini-key-123")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_gemini_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            await client.chat([Message.user("Hi")])

            call_args = mock_post.call_args
            url = call_args.args[0]
            assert "key=gemini-key-123" in url

    @pytest.mark.asyncio
    async def test_system_instruction_format(self, mock_gemini_response):
        """测试系统指令格式"""
        client = GeminiClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_gemini_response

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            await client.chat(
                [Message.user("Hi")],
                system_prompt="Be helpful"
            )

            call_args = mock_post.call_args
            body = call_args.kwargs['json']
            assert 'systemInstruction' in body
            assert body['systemInstruction']['parts'][0]['text'] == "Be helpful"


# ==================== Custom 客户端测试 ====================

class TestCustomClient:
    """自定义客户端测试"""

    @pytest.mark.asyncio
    async def test_uses_openai_format(self):
        """测试使用 OpenAI 兼容格式"""
        client = CustomClient(
            api_key="test-key",
            base_url="https://custom.api.com/v1"
        )

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "id": "custom-123",
            "model": "custom-model",
            "choices": [{
                "message": {"content": "Custom response"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        }

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            result = await client.chat([Message.user("Hi")])

            assert result.content == "Custom response"
            # 使用 OpenAI 解析器
            assert result.response_id == "custom-123"


# ==================== 错误处理测试 ====================

class TestErrorHandling:
    """错误处理测试"""

    @pytest.mark.asyncio
    async def test_authentication_error_401(self):
        """测试 401 认证失败"""
        client = AnthropicClient(api_key="invalid-key")

        mock_response = Mock()
        mock_response.status_code = 401
        mock_response.text = "Unauthorized"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(AuthenticationError):
                await client.chat([Message.user("Hi")])

    @pytest.mark.asyncio
    async def test_rate_limit_error_429(self):
        """测试 429 速率限制"""
        client = OpenAIClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 429
        mock_response.text = "Rate limit exceeded"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(RateLimitError):
                await client.chat([Message.user("Hi")])

    @pytest.mark.asyncio
    async def test_model_not_found_404(self):
        """测试 404 模型不存在"""
        client = AnthropicClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 404
        mock_response.text = "Model not found"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(ModelNotFoundError):
                await client.chat([Message.user("Hi")])

    @pytest.mark.asyncio
    async def test_network_error_502(self):
        """测试 502 网络错误"""
        client = GeminiClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 502
        mock_response.text = "Bad gateway"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(NetworkError):
                await client.chat([Message.user("Hi")])

    @pytest.mark.asyncio
    async def test_timeout_error_504(self):
        """测试 504 超时"""
        client = OpenAIClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 504
        mock_response.text = "Gateway timeout"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(TimeoutError):
                await client.chat([Message.user("Hi")])

    @pytest.mark.asyncio
    async def test_invalid_json_response(self):
        """测试无效 JSON 响应"""
        client = AnthropicClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.side_effect = json.JSONDecodeError("test", "test", 0)
        mock_response.text = "invalid json"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(InvalidResponseError):
                await client.chat([Message.user("Hi")])


class TestErrorClassification:
    """错误分类测试"""

    def test_classify_401(self):
        """测试分类 401"""
        error = classify_http_error(401, "test", "anthropic")
        assert isinstance(error, AuthenticationError)

    def test_classify_403(self):
        """测试分类 403"""
        error = classify_http_error(403, "test", "openai")
        assert isinstance(error, AuthenticationError)

    def test_classify_404(self):
        """测试分类 404"""
        error = classify_http_error(404, "test", "gemini")
        assert isinstance(error, ModelNotFoundError)

    def test_classify_429(self):
        """测试分类 429"""
        error = classify_http_error(429, "test", "anthropic")
        assert isinstance(error, RateLimitError)

    def test_classify_500(self):
        """测试分类 500"""
        error = classify_http_error(500, "Server error", "anthropic")
        assert isinstance(error, LlmError)

    def test_classify_502(self):
        """测试分类 502"""
        error = classify_http_error(502, "test", "openai")
        assert isinstance(error, NetworkError)

    def test_classify_503(self):
        """测试分类 503"""
        error = classify_http_error(503, "test", "gemini")
        assert isinstance(error, NetworkError)

    def test_classify_504(self):
        """测试分类 504"""
        error = classify_http_error(504, "test", "anthropic")
        assert isinstance(error, TimeoutError)

    def test_classify_unknown(self):
        """测试分类未知状态码"""
        error = classify_http_error(418, "I'm a teapot", "anthropic")
        assert isinstance(error, LlmError)


class TestErrorTypes:
    """错误类型测试"""

    def test_llm_error_with_provider(self):
        """测试带提供商的错误消息"""
        error = LlmError("Test error", provider="anthropic")
        assert "[anthropic]" in str(error)
        assert "Test error" in str(error)

    def test_rate_limit_retry_after(self):
        """测试速率限制重试时间"""
        error = RateLimitError("Too many requests", retry_after=30.0)
        assert error.retry_after == 30.0

    def test_timeout_with_duration(self):
        """测试超时错误包含时长"""
        error = TimeoutError("Request timed out", timeout=60.0)
        assert error.timeout == 60.0

    def test_invalid_response_with_data(self):
        """测试无效响应错误包含数据"""
        error = InvalidResponseError("Bad JSON", response_data={"raw": "data"})
        assert error.response_data == {"raw": "data"}

    def test_content_filter_with_reason(self):
        """测试内容过滤错误包含原因"""
        error = ContentFilterError("Blocked", filter_reason="violence")
        assert error.filter_reason == "violence"


# ==================== 资源清理测试 ====================

class TestClientLifecycle:
    """客户端生命周期测试"""

    @pytest.mark.asyncio
    async def test_close_releases_resources(self):
        """测试关闭释放资源"""
        client = AnthropicClient(api_key="test-key")

        with patch.object(client._client, 'aclose', new_callable=AsyncMock) as mock_close:
            await client.close()
            mock_close.assert_called_once()

    @pytest.mark.asyncio
    async def test_context_manager(self):
        """测试上下文管理器"""
        client = AnthropicClient(api_key="test-key")

        with patch.object(client._client, 'aclose', new_callable=AsyncMock) as mock_close:
            async with client as c:
                assert c is client
            mock_close.assert_called_once()


# ==================== 边界条件测试 ====================

class TestEdgeCases:
    """边界条件测试"""

    @pytest.mark.asyncio
    async def test_empty_messages(self):
        """测试空消息列表"""
        client = AnthropicClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "id": "msg-empty",
            "model": "claude-sonnet-4-6",
            "content": [{"type": "text", "text": ""}],
            "usage": {"input_tokens": 0, "output_tokens": 0}
        }

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            result = await client.chat([])
            assert result.content == ""

    @pytest.mark.asyncio
    async def test_special_characters_in_content(self):
        """测试特殊字符内容"""
        client = OpenAIClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "id": "chatcmpl-special",
            "model": "gpt-4",
            "choices": [{
                "message": {"content": "特殊字符: 你好世界! 🎉"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        }

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            result = await client.chat([Message.user("Test")])
            assert "你好世界" in result.content

    @pytest.mark.asyncio
    async def test_very_long_response(self):
        """测试超长响应"""
        client = GeminiClient(api_key="test-key")

        long_content = "A" * 10000

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "candidates": [{
                "content": {"parts": [{"text": long_content}]},
                "finishReason": "MAX_TOKENS"
            }],
            "usageMetadata": {
                "promptTokenCount": 1,
                "candidatesTokenCount": 10000,
                "totalTokenCount": 10001
            }
        }

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            result = await client.chat([Message.user("Write a lot")])
            assert len(result.content) == 10000


# ==================== 流式响应测试 ====================

class TestAnthropicStreaming:
    """Anthropic 流式响应测试"""

    @pytest.mark.asyncio
    async def test_stream_content_chunks(self):
        """测试流式内容块"""
        client = AnthropicClient(api_key="test-key")

        # 构造模拟流式响应
        stream_lines = [
            "data: {\"type\": \"content_block_delta\", \"delta\": {\"type\": \"text_delta\", \"text\": \"Hello\"}}",
            "data: {\"type\": \"content_block_delta\", \"delta\": {\"type\": \"text_delta\", \"text\": \" World\"}}",
            "data: {\"type\": \"message_stop\"}",
        ]

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.aiter_lines = Mock(return_value=self._async_iter(stream_lines))

        with patch.object(client._client, 'stream', return_value=self._async_context(mock_response)):
            chunks = []
            async for chunk in client.chat_stream([Message.user("Hi")]):
                chunks.append(chunk)

            content_chunks = [c for c in chunks if c.content]
            assert len(content_chunks) >= 2
            assert content_chunks[0].content == "Hello"
            assert content_chunks[1].content == " World"

    @pytest.mark.asyncio
    async def test_stream_finish_reason(self):
        """测试流式结束原因"""
        client = AnthropicClient(api_key="test-key")

        stream_lines = [
            "data: {\"type\": \"content_block_delta\", \"delta\": {\"type\": \"text_delta\", \"text\": \"Done\"}}",
            "data: {\"type\": \"message_stop\"}",
        ]

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.aiter_lines = Mock(return_value=self._async_iter(stream_lines))

        with patch.object(client._client, 'stream', return_value=self._async_context(mock_response)):
            chunks = []
            async for chunk in client.chat_stream([Message.user("Hi")]):
                chunks.append(chunk)

            finish_chunks = [c for c in chunks if c.finish_reason]
            assert len(finish_chunks) == 1
            assert finish_chunks[0].finish_reason == "stop"

    @pytest.mark.asyncio
    async def test_stream_error_handling(self):
        """测试流式错误处理"""
        client = AnthropicClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 429
        mock_response.aread = AsyncMock(return_value=b"Rate limit")

        with patch.object(client._client, 'stream', return_value=self._async_context(mock_response)):
            with pytest.raises(RateLimitError):
                async for _ in client.chat_stream([Message.user("Hi")]):
                    pass

    def _async_iter(self, items):
        """创建异步迭代器"""
        async def gen():
            for item in items:
                yield item
        return gen()

    def _async_context(self, response):
        """创建异步上下文管理器"""
        class AsyncCtx:
            async def __aenter__(self):
                return response
            async def __aexit__(self, *args):
                pass
        return AsyncCtx()


class TestOpenAIStreaming:
    """OpenAI 流式响应测试"""

    @pytest.mark.asyncio
    async def test_stream_content(self):
        """测试 OpenAI 流式内容"""
        client = OpenAIClient(api_key="test-key")

        stream_lines = [
            "data: {\"choices\": [{\"delta\": {\"content\": \"Hi\"}}]}",
            "data: {\"choices\": [{\"delta\": {\"content\": \" there\"}}]}",
            "data: [DONE]",
        ]

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.aiter_lines = Mock(return_value=self._async_iter(stream_lines))

        with patch.object(client._client, 'stream', return_value=self._async_context(mock_response)):
            chunks = []
            async for chunk in client.chat_stream([Message.user("Hello")]):
                chunks.append(chunk)

            content_chunks = [c for c in chunks if c.content]
            assert len(content_chunks) == 2

    @pytest.mark.asyncio
    async def test_stream_with_finish(self):
        """测试带结束原因的流"""
        client = OpenAIClient(api_key="test-key")

        stream_lines = [
            "data: {\"choices\": [{\"delta\": {\"content\": \"Done\"}, \"finish_reason\": \"stop\"}]}",
            "data: [DONE]",
        ]

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.aiter_lines = Mock(return_value=self._async_iter(stream_lines))

        with patch.object(client._client, 'stream', return_value=self._async_context(mock_response)):
            chunks = []
            async for chunk in client.chat_stream([Message.user("Hi")]):
                chunks.append(chunk)

            finish_chunks = [c for c in chunks if c.finish_reason]
            assert finish_chunks[0].finish_reason == "stop"

    def _async_iter(self, items):
        async def gen():
            for item in items:
                yield item
        return gen()

    def _async_context(self, response):
        class AsyncCtx:
            async def __aenter__(self):
                return response
            async def __aexit__(self, *args):
                pass
        return AsyncCtx()


class TestGeminiStreaming:
    """Gemini streaming tests""" 

    @pytest.mark.asyncio
    async def test_stream_invocation(self):
        """Test Gemini stream request is sent""" 
        client = GeminiClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.aiter_text = Mock(return_value=self._async_iter([""]))

        with patch.object(client._client, "stream", return_value=self._async_context(mock_response)):
            async for _ in client.chat_stream([Message.user("Hi")]):
                pass
            assert True

    @pytest.mark.asyncio
    async def test_stream_error(self):
        """Test Gemini stream error handling""" 
        client = GeminiClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 500
        mock_response.aread = AsyncMock(return_value=b"Error")

        with patch.object(client._client, "stream", return_value=self._async_context(mock_response)):
            with pytest.raises(LlmError):
                async for _ in client.chat_stream([Message.user("Hi")]):
                    pass

    def _async_iter(self, items):
        async def gen():
            for item in items:
                yield item
        return gen()

    def _async_context(self, response):
        class AsyncCtx:
            async def __aenter__(self):
                return response
            async def __aexit__(self, *args):
                pass
        return AsyncCtx()


class TestCustomStreaming:
    """Custom 流式响应测试"""

    @pytest.mark.asyncio
    async def test_stream_openai_compatible(self):
        """测试自定义端点使用 OpenAI 兼容流式"""
        client = CustomClient(api_key="test-key", base_url="https://custom.api.com/v1")

        stream_lines = [
            "data: {\"choices\": [{\"delta\": {\"content\": \"Custom\"}}]}",
            "data: [DONE]",
        ]

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.aiter_lines = Mock(return_value=self._async_iter(stream_lines))

        with patch.object(client._client, 'stream', return_value=self._async_context(mock_response)):
            chunks = []
            async for chunk in client.chat_stream([Message.user("Hi")]):
                chunks.append(chunk)

            content_chunks = [c for c in chunks if c.content]
            assert len(content_chunks) == 1
            assert content_chunks[0].content == "Custom"

    def _async_iter(self, items):
        async def gen():
            for item in items:
                yield item
        return gen()

    def _async_context(self, response):
        class AsyncCtx:
            async def __aenter__(self):
                return response
            async def __aexit__(self, *args):
                pass
        return AsyncCtx()


# ==================== 更多边界条件测试 ====================

class TestMoreEdgeCases:
    """更多边界条件测试"""

    def test_gemini_tool_format(self):
        """测试 Gemini 工具格式"""
        tool = ToolDefinition(
            name="test",
            description="Test",
            parameters={"type": "object"}
        )
        formatted = tool.to_gemini_format()
        assert formatted["name"] == "test"
        assert formatted["description"] == "Test"

    def test_message_with_tool_call_id(self):
        """测试带 tool_call_id 的消息"""
        msg = Message(
            role=MessageRole.TOOL,
            content="result",
            tool_call_id="call-123"
        )
        openai_format = msg.to_openai_format()
        assert openai_format["tool_call_id"] == "call-123"

    def test_message_with_name(self):
        """测试带 name 的消息"""
        msg = Message(
            role=MessageRole.USER,
            content="Hello",
            name="alice"
        )
        openai_format = msg.to_openai_format()
        assert openai_format["name"] == "alice"

    @pytest.mark.asyncio
    async def test_gemini_with_tools(self):
        """测试 Gemini 带工具"""
        client = GeminiClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "candidates": [{"content": {"parts": [{"text": "OK"}]}}],
            "usageMetadata": {"promptTokenCount": 1, "candidatesTokenCount": 1, "totalTokenCount": 2}
        }

        tools = [ToolDefinition(name="calc", description="Calculate", parameters={"type": "object"})]

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            await client.chat([Message.user("Hi")], tools=tools)

            call_args = mock_post.call_args
            body = call_args.kwargs['json']
            assert 'tools' in body
            assert 'functionDeclarations' in body['tools'][0]

    def test_openai_response_with_tool_calls(self):
        """测试 OpenAI 响应包含 tool calls"""
        data = {
            "id": "chatcmpl-123",
            "model": "gpt-4",
            "choices": [{
                "message": {
                    "content": "",
                    "tool_calls": [{"id": "call-1", "function": {"name": "test"}}]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        }
        response = ChatResponse.from_openai(data)
        assert len(response.tool_calls) == 1
        assert response.finish_reason == "tool_calls"

    @pytest.mark.asyncio
    async def test_anthropic_500_error(self):
        """测试 Anthropic 500 服务器错误"""
        client = AnthropicClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 500
        mock_response.text = "Internal Server Error"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(LlmError):
                await client.chat([Message.user("Hi")])

    @pytest.mark.asyncio
    async def test_openai_503_error(self):
        """测试 OpenAI 503 服务不可用"""
        client = OpenAIClient(api_key="test-key")

        mock_response = Mock()
        mock_response.status_code = 503
        mock_response.text = "Service Unavailable"

        with patch.object(client._client, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.return_value = mock_response

            with pytest.raises(NetworkError):
                await client.chat([Message.user("Hi")])


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
