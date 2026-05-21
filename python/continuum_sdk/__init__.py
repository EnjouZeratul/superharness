"""
Continuum SDK

Python SDK for Continuum - A terminal agent framework with real LLM calls.

Features:
    - Real LLM API calls (Anthropic, OpenAI, Gemini)
    - Streaming response support
    - Tool registration and function calling
    - Session persistence and recovery
    - Multi-provider configuration

Quick Start (3 steps):
    >>> from continuum import Agent
    >>> agent = Agent()  # Auto-configures from environment
    >>> result = agent.run("hello")

With explicit configuration:
    >>> from continuum import Agent, Config
    >>> config = Config.from_env()
    >>> agent = Agent(config=config)

Streaming:
    >>> async for chunk in agent.run_stream("hello"):
    ...     print(chunk.content)

Tools:
    >>> agent.register_tool(
    ...     "calc",
    ...     lambda x: eval(x),
    ...     description="Evaluate math expressions",
    ...     parameters={"type": "object", "properties": {"expression": {"type": "string"}}}
    ... )
"""

__version__ = "1.0.0"

# Core classes
from .agent import Agent, Session
from .config import (
    Config,
    ConfigLoader,
    load_config,
    list_providers,
    get_default_model,
)

# LLM module (for advanced usage)
from .llm import (
    LlmClient,
    AnthropicClient,
    OpenAIClient,
    GeminiClient,
    Message,
    MessageRole,
    ChatResponse,
    StreamChunk,
    TokenUsage,
    LlmError,
)

__all__ = [
    # Core
    "Agent",
    "Session",
    # Config
    "Config",
    "ConfigLoader",
    "load_config",
    "list_providers",
    "get_default_model",
    # LLM (advanced)
    "LlmClient",
    "AnthropicClient",
    "OpenAIClient",
    "GeminiClient",
    "Message",
    "MessageRole",
    "ChatResponse",
    "StreamChunk",
    "TokenUsage",
    "LlmError",
]