"""
LLM Client

Real LLM API client implementations.
"""

import asyncio
import json
from abc import ABC, abstractmethod
from typing import (
    Optional,
    List,
    Dict,
    Any,
    AsyncIterator,
    Union,
    Callable,
)

import httpx

from .types import (
    Message,
    MessageRole,
    ChatResponse,
    StreamChunk,
    TokenUsage,
    ToolDefinition,
)
from .errors import (
    LlmError,
    AuthenticationError,
    NetworkError,
    TimeoutError,
    InvalidResponseError,
    classify_http_error,
)


class BaseLlmClient(ABC):
    """
    Abstract base class for LLM clients.
    """

    def __init__(
        self,
        api_key: str,
        base_url: Optional[str] = None,
        timeout: float = 60.0,
        max_retries: int = 3,
        proxy: Optional[str] = None,
    ):
        self.api_key = api_key
        self.base_url = base_url
        self.timeout = timeout
        self.max_retries = max_retries
        self.proxy = proxy

        # Build httpx client
        limits = httpx.Limits(max_connections=10, max_keepalive_connections=5)
        transport = httpx.HTTPTransport(proxy=proxy) if proxy else None

        self._client = httpx.AsyncClient(
            timeout=httpx.Timeout(timeout),
            limits=limits,
            transport=transport,
        )

    @abstractmethod
    async def chat(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> ChatResponse:
        """
        Send chat completion request.

        Args:
            messages: List of conversation messages
            model: Model to use (provider-specific)
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            system_prompt: System prompt
            tools: Available tools for function calling
            **kwargs: Additional provider-specific options

        Returns:
            ChatResponse with the model's reply
        """
        pass

    @abstractmethod
    async def chat_stream(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> AsyncIterator[StreamChunk]:
        """
        Send streaming chat completion request.

        Yields:
            StreamChunk objects as they arrive
        """
        pass

    async def close(self):
        """Close the HTTP client."""
        await self._client.aclose()

    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.close()


class AnthropicClient(BaseLlmClient):
    """
    Anthropic Claude API client.

    API Reference: https://docs.anthropic.com/claude/reference
    """

    DEFAULT_BASE_URL = "https://api.anthropic.com/v1"

    def __init__(
        self,
        api_key: str,
        base_url: Optional[str] = None,
        default_model: str = "claude-sonnet-4-6",
        **kwargs,
    ):
        super().__init__(api_key, base_url or self.DEFAULT_BASE_URL, **kwargs)
        self.default_model = default_model
        self.provider = "anthropic"

    def _build_headers(self) -> Dict[str, str]:
        return {
            "x-api-key": self.api_key,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        }

    async def chat(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> ChatResponse:
        """Send chat request to Anthropic Claude API."""
        # 构建正确的 URL：如果 base_url 不包含 /v1，则添加
        if self.base_url.endswith("/v1") or self.base_url.endswith("/v1/"):
            url = f"{self.base_url.rstrip('/')}/messages"
        else:
            url = f"{self.base_url.rstrip('/')}/v1/messages"

        # Build request body
        body: Dict[str, Any] = {
            "model": model or self.default_model,
            "max_tokens": max_tokens,
            "messages": [m.to_anthropic_format() for m in messages],
            "temperature": temperature,
        }

        if system_prompt:
            body["system"] = system_prompt

        if tools:
            body["tools"] = [t.to_anthropic_format() for t in tools]

        # Add any extra kwargs
        body.update(kwargs)

        # Send request
        response = await self._client.post(
            url,
            headers=self._build_headers(),
            json=body,
        )

        if response.status_code != 200:
            raise classify_http_error(
                response.status_code,
                response.text,
                self.provider,
            )

        try:
            data = response.json()
            return ChatResponse.from_anthropic(data)
        except (json.JSONDecodeError, KeyError) as e:
            raise InvalidResponseError(
                f"Failed to parse Anthropic response: {e}",
                provider=self.provider,
                response_data=response.text,
            )

    async def chat_stream(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> AsyncIterator[StreamChunk]:
        """Send streaming chat request to Anthropic Claude API."""
        # 构建正确的 URL：如果 base_url 不包含 /v1，则添加
        if self.base_url.endswith("/v1") or self.base_url.endswith("/v1/"):
            url = f"{self.base_url.rstrip('/')}/messages"
        else:
            url = f"{self.base_url.rstrip('/')}/v1/messages"

        body: Dict[str, Any] = {
            "model": model or self.default_model,
            "max_tokens": max_tokens,
            "messages": [m.to_anthropic_format() for m in messages],
            "temperature": temperature,
            "stream": True,
        }

        if system_prompt:
            body["system"] = system_prompt

        if tools:
            body["tools"] = [t.to_anthropic_format() for t in tools]

        body.update(kwargs)

        async with self._client.stream(
            "POST",
            url,
            headers=self._build_headers(),
            json=body,
        ) as response:
            if response.status_code != 200:
                error_text = await response.aread()
                raise classify_http_error(
                    response.status_code,
                    error_text.decode(),
                    self.provider,
                )

            async for line in response.aiter_lines():
                if not line.startswith("data: "):
                    continue

                data_str = line[6:]  # Remove "data: " prefix
                if data_str.strip() == "[DONE]":
                    break

                try:
                    data = json.loads(data_str)
                except json.JSONDecodeError:
                    continue

                event_type = data.get("type", "")

                if event_type == "content_block_delta":
                    delta = data.get("delta", {})
                    if delta.get("type") == "text_delta":
                        yield StreamChunk(content=delta.get("text", ""))

                elif event_type == "message_stop":
                    yield StreamChunk(finish_reason="stop")
                    break


class OpenAIClient(BaseLlmClient):
    """
    OpenAI GPT API client.

    API Reference: https://platform.openai.com/docs/api-reference
    """

    DEFAULT_BASE_URL = "https://api.openai.com/v1"

    def __init__(
        self,
        api_key: str,
        base_url: Optional[str] = None,
        default_model: str = "gpt-4",
        **kwargs,
    ):
        super().__init__(api_key, base_url or self.DEFAULT_BASE_URL, **kwargs)
        self.default_model = default_model
        self.provider = "openai"

    def _build_headers(self) -> Dict[str, str]:
        return {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

    async def chat(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> ChatResponse:
        """Send chat request to OpenAI API."""
        url = f"{self.base_url}/chat/completions"

        # Build messages list
        api_messages = []
        if system_prompt:
            api_messages.append({"role": "system", "content": system_prompt})
        api_messages.extend([m.to_openai_format() for m in messages])

        body: Dict[str, Any] = {
            "model": model or self.default_model,
            "messages": api_messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
        }

        if tools:
            body["tools"] = [t.to_openai_format() for t in tools]

        body.update(kwargs)

        response = await self._client.post(
            url,
            headers=self._build_headers(),
            json=body,
        )

        if response.status_code != 200:
            raise classify_http_error(
                response.status_code,
                response.text,
                self.provider,
            )

        try:
            data = response.json()
            return ChatResponse.from_openai(data)
        except (json.JSONDecodeError, KeyError) as e:
            raise InvalidResponseError(
                f"Failed to parse OpenAI response: {e}",
                provider=self.provider,
                response_data=response.text,
            )

    async def chat_stream(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> AsyncIterator[StreamChunk]:
        """Send streaming chat request to OpenAI API."""
        url = f"{self.base_url}/chat/completions"

        api_messages = []
        if system_prompt:
            api_messages.append({"role": "system", "content": system_prompt})
        api_messages.extend([m.to_openai_format() for m in messages])

        body: Dict[str, Any] = {
            "model": model or self.default_model,
            "messages": api_messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "stream": True,
        }

        if tools:
            body["tools"] = [t.to_openai_format() for t in tools]

        body.update(kwargs)

        async with self._client.stream(
            "POST",
            url,
            headers=self._build_headers(),
            json=body,
        ) as response:
            if response.status_code != 200:
                error_text = await response.aread()
                raise classify_http_error(
                    response.status_code,
                    error_text.decode(),
                    self.provider,
                )

            async for line in response.aiter_lines():
                if not line.startswith("data: "):
                    continue

                data_str = line[6:]
                if data_str.strip() == "[DONE]":
                    yield StreamChunk(finish_reason="stop")
                    break

                try:
                    data = json.loads(data_str)
                except json.JSONDecodeError:
                    continue

                choices = data.get("choices", [])
                if choices:
                    delta = choices[0].get("delta", {})
                    content = delta.get("content", "")
                    finish_reason = choices[0].get("finish_reason")

                    if content:
                        yield StreamChunk(content=content)
                    if finish_reason:
                        yield StreamChunk(finish_reason=finish_reason)


class GeminiClient(BaseLlmClient):
    """
    Google Gemini API client.

    API Reference: https://ai.google.dev/tutorials/python_quickstart
    """

    DEFAULT_BASE_URL = "https://generativelanguage.googleapis.com/v1beta"

    def __init__(
        self,
        api_key: str,
        base_url: Optional[str] = None,
        default_model: str = "gemini-1.5-pro",
        **kwargs,
    ):
        super().__init__(api_key, base_url or self.DEFAULT_BASE_URL, **kwargs)
        self.default_model = default_model
        self.provider = "gemini"

    def _build_url(self, model: str, method: str = "generateContent") -> str:
        return f"{self.base_url}/models/{model}:{method}?key={self.api_key}"

    async def chat(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> ChatResponse:
        """Send chat request to Google Gemini API."""
        model_name = model or self.default_model
        url = self._build_url(model_name, "generateContent")

        # Build contents
        contents = [m.to_gemini_format() for m in messages]

        body: Dict[str, Any] = {
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": max_tokens,
                "temperature": temperature,
            },
        }

        if system_prompt:
            body["systemInstruction"] = {
                "parts": [{"text": system_prompt}]
            }

        if tools:
            body["tools"] = [{
                "functionDeclarations": [t.to_gemini_format() for t in tools]
            }]

        body.update(kwargs)

        response = await self._client.post(url, json=body)

        if response.status_code != 200:
            raise classify_http_error(
                response.status_code,
                response.text,
                self.provider,
            )

        try:
            data = response.json()
            return ChatResponse.from_gemini(data, model_name)
        except (json.JSONDecodeError, KeyError) as e:
            raise InvalidResponseError(
                f"Failed to parse Gemini response: {e}",
                provider=self.provider,
                response_data=response.text,
            )

    async def chat_stream(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> AsyncIterator[StreamChunk]:
        """Send streaming chat request to Google Gemini API."""
        model_name = model or self.default_model
        url = self._build_url(model_name, "streamGenerateContent")

        contents = [m.to_gemini_format() for m in messages]

        body: Dict[str, Any] = {
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": max_tokens,
                "temperature": temperature,
            },
        }

        if system_prompt:
            body["systemInstruction"] = {
                "parts": [{"text": system_prompt}]
            }

        body.update(kwargs)

        async with self._client.stream("POST", url, json=body) as response:
            if response.status_code != 200:
                error_text = await response.aread()
                raise classify_http_error(
                    response.status_code,
                    error_text.decode(),
                    self.provider,
                )

            buffer = ""
            async for chunk in response.aiter_text():
                buffer += chunk

                # Gemini returns JSON array elements
                while "},{" in buffer:
                    idx = buffer.index("},{") + 1
                    part = buffer[:idx]
                    buffer = buffer[idx:]

                    try:
                        # Remove array brackets if present
                        if part.startswith("["):
                            part = part[1:]
                        if part.endswith("]"):
                            part = part[:-1]

                        data = json.loads(part)

                        candidates = data.get("candidates", [])
                        if candidates:
                            parts = candidates[0].get("content", {}).get("parts", [])
                            if parts:
                                text = parts[0].get("text", "")
                                if text:
                                    yield StreamChunk(content=text)

                            finish = candidates[0].get("finishReason")
                            if finish:
                                yield StreamChunk(finish_reason=finish)
                    except json.JSONDecodeError:
                        continue


class CustomClient(BaseLlmClient):
    """
    Custom/OpenAI-compatible API client.

    For endpoints that follow OpenAI's API format but use different URLs.
    """

    def __init__(
        self,
        api_key: str,
        base_url: str,
        default_model: str = "default",
        **kwargs,
    ):
        super().__init__(api_key, base_url, **kwargs)
        self.default_model = default_model
        self.provider = "custom"

    def _build_headers(self) -> Dict[str, str]:
        return {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

    async def chat(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> ChatResponse:
        """Send chat request to custom OpenAI-compatible API."""
        url = f"{self.base_url}/chat/completions"

        api_messages = []
        if system_prompt:
            api_messages.append({"role": "system", "content": system_prompt})
        api_messages.extend([m.to_openai_format() for m in messages])

        body: Dict[str, Any] = {
            "model": model or self.default_model,
            "messages": api_messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
        }

        body.update(kwargs)

        response = await self._client.post(
            url,
            headers=self._build_headers(),
            json=body,
        )

        if response.status_code != 200:
            raise classify_http_error(
                response.status_code,
                response.text,
                self.provider,
            )

        try:
            data = response.json()
            return ChatResponse.from_openai(data)
        except (json.JSONDecodeError, KeyError) as e:
            raise InvalidResponseError(
                f"Failed to parse response: {e}",
                provider=self.provider,
                response_data=response.text,
            )

    async def chat_stream(
        self,
        messages: List[Message],
        model: Optional[str] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        tools: Optional[List[ToolDefinition]] = None,
        **kwargs,
    ) -> AsyncIterator[StreamChunk]:
        """Send streaming chat request to custom API."""
        url = f"{self.base_url}/chat/completions"

        api_messages = []
        if system_prompt:
            api_messages.append({"role": "system", "content": system_prompt})
        api_messages.extend([m.to_openai_format() for m in messages])

        body: Dict[str, Any] = {
            "model": model or self.default_model,
            "messages": api_messages,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "stream": True,
        }

        body.update(kwargs)

        async with self._client.stream(
            "POST",
            url,
            headers=self._build_headers(),
            json=body,
        ) as response:
            if response.status_code != 200:
                error_text = await response.aread()
                raise classify_http_error(
                    response.status_code,
                    error_text.decode(),
                    self.provider,
                )

            async for line in response.aiter_lines():
                if not line.startswith("data: "):
                    continue

                data_str = line[6:]
                if data_str.strip() == "[DONE]":
                    yield StreamChunk(finish_reason="stop")
                    break

                try:
                    data = json.loads(data_str)
                    choices = data.get("choices", [])
                    if choices:
                        delta = choices[0].get("delta", {})
                        content = delta.get("content", "")
                        if content:
                            yield StreamChunk(content=content)
                except json.JSONDecodeError:
                    continue


class LlmClient:
    """
    Unified LLM client factory.

    Creates the appropriate client based on provider.

    Usage:
        >>> client = LlmClient.for_provider("anthropic", api_key="...")
        >>> response = await client.chat([Message.user("Hello")])
    """

    @staticmethod
    def for_provider(
        provider: str,
        api_key: str,
        base_url: Optional[str] = None,
        model: Optional[str] = None,
        **kwargs,
    ) -> BaseLlmClient:
        """
        Create client for specified provider.

        Args:
            provider: Provider name (anthropic, openai, gemini, custom)
            api_key: API key for authentication
            base_url: Optional custom base URL
            model: Default model to use
            **kwargs: Additional client options

        Returns:
            Appropriate LLM client instance
        """
        provider = provider.lower()

        if provider == "anthropic":
            return AnthropicClient(
                api_key=api_key,
                base_url=base_url,
                default_model=model or "claude-sonnet-4-6",
                **kwargs,
            )
        elif provider == "openai":
            return OpenAIClient(
                api_key=api_key,
                base_url=base_url,
                default_model=model or "gpt-4",
                **kwargs,
            )
        elif provider in ("gemini", "google"):
            return GeminiClient(
                api_key=api_key,
                base_url=base_url,
                default_model=model or "gemini-1.5-pro",
                **kwargs,
            )
        elif provider == "custom":
            if not base_url:
                raise ValueError("base_url is required for custom provider")
            return CustomClient(
                api_key=api_key,
                base_url=base_url,
                default_model=model or "default",
                **kwargs,
            )
        else:
            raise ValueError(f"Unknown provider: {provider}")
