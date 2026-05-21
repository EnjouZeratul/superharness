"""
LLM Client Module

Real LLM API client implementations for Continuum SDK.

Supports:
    - Anthropic Claude API
    - OpenAI GPT API
    - Google Gemini API
    - Custom endpoints

Usage:
    >>> from continuum_sdk.llm import LlmClient
    >>> client = LlmClient.for_provider("anthropic", api_key="...")
    >>> response = client.chat(messages=[{"role": "user", "content": "Hello"}])
"""

from .client import (
    LlmClient,
    BaseLlmClient,
    AnthropicClient,
    OpenAIClient,
    GeminiClient,
    CustomClient,
)
from .errors import (
    LlmError,
    AuthenticationError,
    RateLimitError,
    NetworkError,
    TimeoutError,
    InvalidResponseError,
)
from .types import (
    Message,
    MessageRole,
    ChatResponse,
    StreamChunk,
    TokenUsage,
    ToolDefinition,
)

__all__ = [
    # Client
    "LlmClient",
    "BaseLlmClient",
    "AnthropicClient",
    "OpenAIClient",
    "GeminiClient",
    "CustomClient",
    # Errors
    "LlmError",
    "AuthenticationError",
    "RateLimitError",
    "NetworkError",
    "TimeoutError",
    "InvalidResponseError",
    # Types
    "Message",
    "MessageRole",
    "ChatResponse",
    "StreamChunk",
    "TokenUsage",
    "ToolDefinition",
]