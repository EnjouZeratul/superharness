"""
Agent Runtime

Main Agent class for SuperHarness SDK.

The Agent is the primary interface for running AI-powered tasks.
It provides a simple Quick Start API for common use cases while
supporting advanced configuration for complex workflows.

Key Features:
    - Quick Start: 3-step agent creation and execution
    - Session management for conversation persistence
    - Tool registration for custom functionality
    - State tracking (idle, running, paused, error)
    - Automatic configuration from environment
    - Real LLM API calls (Anthropic, OpenAI, Gemini)

Quick Start (3 steps):
    >>> from superharness import Agent
    >>> agent = Agent()  # Auto-configures from environment
    >>> result = agent.run("your task")

Advanced Usage:
    >>> agent = Agent(name="my-agent", model="claude-sonnet-4-6")
    >>> agent.start()
    >>> agent.register_tool("calc", lambda x: eval(x))
    >>> result = agent.execute("calculate 2+2")
    >>> session = agent.create_session()

Configuration:
    The Agent auto-loads configuration from:
    1. Environment variables (SUPERHARNESS_*)
    2. Config file (~/.superharness/config.toml)
    3. Default values

Environment Variables:
    - SUPERHARNESS_API_KEY: API key for LLM provider
    - SUPERHARNESS_PROVIDER: Provider name (anthropic, openai, google)
    - SUPERHARNESS_MODEL: Model name to use
"""

import asyncio
from typing import Optional, Dict, Any, Callable, Union, AsyncIterator, List
from datetime import datetime
from enum import Enum
import json

# Import Rust bindings (will be available after compilation)
try:
    from sh_core import Agent as RustAgent
    HAS_RUST_BINDINGS = True
except ImportError:
    HAS_RUST_BINDINGS = False

# Import config module for auto-configuration
from ..config import Config

# Import LLM client
from ..llm import (
    LlmClient,
    BaseLlmClient,
    Message,
    MessageRole as LlmMessageRole,
    ChatResponse,
    StreamChunk,
    ToolDefinition,
    LlmError,
    AuthenticationError,
)


class AgentState(Enum):
    """Agent state enumeration."""
    IDLE = "idle"
    RUNNING = "running"
    PAUSED = "paused"
    ERROR = "error"


class AgentConfig:
    """
    Agent configuration.

    Attributes:
        name: Agent identifier name
        model: LLM model to use
        provider: LLM provider (anthropic, openai, gemini)
        api_key: API key for authentication
        base_url: Optional custom API endpoint
        budget: Optional cost budget limit
        max_tokens: Maximum tokens per response
        temperature: Sampling temperature
        system_prompt: Optional system prompt
        timeout: Request timeout in seconds
    """

    def __init__(
        self,
        name: str = "default",
        model: str = "claude-sonnet-4-6",
        provider: str = "anthropic",
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        budget: Optional[float] = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        system_prompt: Optional[str] = None,
        timeout: float = 60.0,
        tools: Optional[list] = None,
    ):
        self.name = name
        self.model = model
        self.provider = provider
        self.api_key = api_key
        self.base_url = base_url
        self.budget = budget
        self.max_tokens = max_tokens
        self.temperature = temperature
        self.system_prompt = system_prompt
        self.timeout = timeout
        self.tools = tools or []

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "model": self.model,
            "provider": self.provider,
            "api_key": self.api_key,
            "base_url": self.base_url,
            "budget": self.budget,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
            "system_prompt": self.system_prompt,
            "timeout": self.timeout,
            "tools": self.tools,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "AgentConfig":
        return cls(
            name=data.get("name", "default"),
            model=data.get("model", "claude-sonnet-4-6"),
            provider=data.get("provider", "anthropic"),
            api_key=data.get("api_key"),
            base_url=data.get("base_url"),
            budget=data.get("budget"),
            max_tokens=data.get("max_tokens", 4096),
            temperature=data.get("temperature", 0.7),
            system_prompt=data.get("system_prompt"),
            timeout=data.get("timeout", 60.0),
            tools=data.get("tools"),
        )

    @classmethod
    def from_config(cls, config: Config) -> "AgentConfig":
        """Convert from SDK Config class."""
        return cls(
            name="default",
            model=config.model,
            provider=config.provider,
            api_key=config.api_key,
            base_url=config.base_url,
            budget=config.budget,
            max_tokens=config.max_tokens,
            temperature=config.temperature,
        )


class Agent:
    """
    SuperHarness Agent class.

    The primary interface for running AI-powered tasks with real LLM calls.

    Features:
        - Task execution with real LLM API calls
        - Streaming response support
        - Tool registration for function calling
        - Session management for conversation persistence
        - State tracking (idle, running, paused, error)
        - Automatic configuration from environment

    Quick Start:
        >>> from superharness import Agent
        >>> agent = Agent()  # Auto-configures
        >>> result = agent.run("your task")

    Advanced Usage:
        >>> # Custom configuration
        >>> agent = Agent(name="assistant", model="claude-sonnet-4-6")
        >>> agent.register_tool("search", my_search_function)
        >>> agent.start()
        >>> session = agent.create_session()
        >>> result = agent.execute("search for documents")

    Attributes:
        name: Agent name/identifier
        state: Current state (AgentState enum)
        config: Agent configuration (AgentConfig)
        created_at: Agent creation timestamp

    See Also:
        Session: For conversation management
        AgentConfig: For configuration details
        AgentState: For state values
    """

    def __init__(
        self,
        name: Optional[str] = None,
        config: Optional[Union[AgentConfig, Config]] = None,
        model: Optional[str] = None,
        api_key: Optional[str] = None,
        provider: Optional[str] = None,
    ):
        """
        Create a new Agent.

        Args:
            name: Agent name/identifier
            config: Configuration object (AgentConfig or Config)
            model: Model name (overrides config)
            api_key: API key (overrides config)
            provider: Provider name (overrides config)
        """
        self._name = name or "default"

        # Handle configuration
        if config is None:
            # Auto-load from environment
            sdk_config = Config.from_default()
            self._config = AgentConfig.from_config(sdk_config)
        elif isinstance(config, Config):
            self._config = AgentConfig.from_config(config)
        else:
            self._config = config

        # Override specific parameters
        if model:
            self._config.model = model
        if api_key:
            self._config.api_key = api_key
        if provider:
            self._config.provider = provider

        self._state = AgentState.IDLE
        self._current_session: Optional["Session"] = None
        self._sessions: Dict[str, "Session"] = {}
        self._tools: Dict[str, Callable] = {}
        self._tool_definitions: List[ToolDefinition] = []
        self._created_at = datetime.now()

        # LLM client (initialized on first use)
        self._llm_client: Optional[BaseLlmClient] = None

        # Rust bindings if available
        if HAS_RUST_BINDINGS:
            self._rust_agent = RustAgent(self._name)
        else:
            self._rust_agent = None

    def _get_llm_client(self) -> BaseLlmClient:
        """
        Get or create LLM client.

        Raises:
            ValueError: If API key is not configured
        """
        if self._llm_client is None:
            if not self._config.api_key:
                raise ValueError(
                    "API key is required. Set SUPERHARNESS_API_KEY environment variable "
                    "or pass api_key parameter."
                )

            self._llm_client = LlmClient.for_provider(
                provider=self._config.provider,
                api_key=self._config.api_key,
                base_url=self._config.base_url,
                model=self._config.model,
                timeout=self._config.timeout,
            )

        return self._llm_client

    @property
    def name(self) -> str:
        """Agent name."""
        return self._name

    @property
    def state(self) -> AgentState:
        """Agent state."""
        if self._rust_agent:
            rust_state = self._rust_agent.state()
            return AgentState(rust_state)
        return self._state

    @property
    def config(self) -> AgentConfig:
        """Agent configuration."""
        return self._config

    @property
    def created_at(self) -> datetime:
        """Creation timestamp."""
        return self._created_at

    def start(self) -> None:
        """Start the Agent."""
        if self._rust_agent:
            self._rust_agent.start()
        else:
            if self._state == AgentState.RUNNING:
                raise RuntimeError("Agent is already running")
            if self._state == AgentState.ERROR:
                raise RuntimeError("Agent is in error state")
            self._state = AgentState.RUNNING

    def pause(self) -> None:
        """Pause the Agent."""
        if self._rust_agent:
            self._rust_agent.pause()
        else:
            if self._state != AgentState.RUNNING:
                raise RuntimeError("Agent is not running")
            self._state = AgentState.PAUSED

    def stop(self) -> None:
        """Stop the Agent."""
        if self._rust_agent:
            self._rust_agent.stop()
        self._state = AgentState.IDLE

    def execute(self, task: str) -> str:
        """
        Execute a task using the LLM.

        This is a synchronous wrapper around the async execute_async method.
        Works both in sync and async contexts.

        Args:
            task: Task description or user message

        Returns:
            LLM response content

        Raises:
            RuntimeError: If agent is not running
            ValueError: If API key is not configured
            LlmError: If LLM API call fails
        """
        try:
            loop = asyncio.get_running_loop()
            # Already in async context, need to run in thread
            import concurrent.futures
            with concurrent.futures.ThreadPoolExecutor() as executor:
                future = executor.submit(asyncio.run, self.execute_async(task))
                return future.result()
        except RuntimeError:
            # No running loop, use asyncio.run
            return asyncio.run(self.execute_async(task))

    async def execute_async(self, task: str) -> str:
        """
        Execute a task asynchronously.

        Makes a real LLM API call and returns the response.

        Args:
            task: Task description or user message

        Returns:
            LLM response content
        """
        if self._rust_agent:
            return self._rust_agent.execute(task)

        if self._state != AgentState.RUNNING:
            raise RuntimeError("Agent is not running")

        # Build messages
        messages: List[Message] = []

        # Add conversation history from current session
        if self._current_session:
            for msg in self._current_session.get_messages():
                role = LlmMessageRole.USER if msg.role.value == "user" else LlmMessageRole.ASSISTANT
                if msg.role.value == "system":
                    role = LlmMessageRole.SYSTEM
                messages.append(Message(role=role, content=msg.content))

        # Add the new task
        messages.append(Message.user(task))

        # Get LLM client
        client = self._get_llm_client()

        # Make API call
        try:
            response: ChatResponse = await client.chat(
                messages=messages,
                model=self._config.model,
                max_tokens=self._config.max_tokens,
                temperature=self._config.temperature,
                system_prompt=self._config.system_prompt,
                tools=self._tool_definitions if self._tool_definitions else None,
            )

            # Track token usage
            if self._current_session:
                self._current_session.update_cost(
                    cost=0.0,  # Cost calculation would need pricing data
                    tokens=response.usage.total_tokens,
                )

            return response.content

        except AuthenticationError as e:
            self._state = AgentState.ERROR
            raise ValueError(
                f"Authentication failed: {e}. Check your API key."
            ) from e

        except LlmError as e:
            self._state = AgentState.ERROR
            raise RuntimeError(f"LLM error: {e}") from e

    async def execute_stream(self, task: str) -> AsyncIterator[StreamChunk]:
        """
        Execute a task with streaming response.

        Yields response chunks as they arrive from the LLM.

        Args:
            task: Task description or user message

        Yields:
            StreamChunk objects containing response content
        """
        if self._state != AgentState.RUNNING:
            raise RuntimeError("Agent is not running")

        # Build messages
        messages: List[Message] = []

        if self._current_session:
            for msg in self._current_session.get_messages():
                role = LlmMessageRole.USER if msg.role.value == "user" else LlmMessageRole.ASSISTANT
                if msg.role.value == "system":
                    role = LlmMessageRole.SYSTEM
                messages.append(Message(role=role, content=msg.content))

        messages.append(Message.user(task))

        client = self._get_llm_client()

        full_content = ""

        try:
            async for chunk in client.chat_stream(
                messages=messages,
                model=self._config.model,
                max_tokens=self._config.max_tokens,
                temperature=self._config.temperature,
                system_prompt=self._config.system_prompt,
            ):
                if chunk.content:
                    full_content += chunk.content
                yield chunk

            # Record complete response to session
            if self._current_session and full_content:
                self._current_session.add_assistant_message(full_content)

        except LlmError as e:
            self._state = AgentState.ERROR
            raise RuntimeError(f"LLM streaming error: {e}") from e

    def run(self, task: str, auto_start: bool = True) -> str:
        """
        One-shot task execution (Quick Start method).

        Automatically handles startup, session creation, and execution.

        Example:
            >>> from superharness import Agent
            >>> agent = Agent()
            >>> result = agent.run("hello")

        Args:
            task: Task description
            auto_start: Auto-start agent if not running (default True)

        Returns:
            LLM response content
        """
        if auto_start and self._state != AgentState.RUNNING:
            self.start()

        if not self._current_session:
            self._current_session = self.create_session()

        result = self.execute(task)

        self._current_session.add_user_message(task)
        self._current_session.add_assistant_message(result)

        return result

    async def run_stream(self, task: str, auto_start: bool = True) -> AsyncIterator[StreamChunk]:
        """
        Streaming task execution.

        Automatically handles startup and session creation.

        Args:
            task: Task description
            auto_start: Auto-start agent if not running

        Yields:
            StreamChunk objects
        """
        if auto_start and self._state != AgentState.RUNNING:
            self.start()

        if not self._current_session:
            self._current_session = self.create_session()

        self._current_session.add_user_message(task)

        async for chunk in self.execute_stream(task):
            yield chunk

    def chat(self, message: str) -> str:
        """
        Send a message and get response (Quick Start method).

        Simple alias for run().

        Args:
            message: User message

        Returns:
            Agent response
        """
        return self.run(message)

    async def chat_stream(self, message: str) -> AsyncIterator[StreamChunk]:
        """
        Streaming chat method.

        Args:
            message: User message

        Yields:
            StreamChunk objects
        """
        async for chunk in self.run_stream(message):
            yield chunk

    def create_session(self, session_id: Optional[str] = None) -> "Session":
        """
        Create a new conversation session.

        Args:
            session_id: Optional session identifier

        Returns:
            New Session instance
        """
        from .session import Session

        sid = session_id or f"{self._name}-session-{len(self._sessions)}"
        session = Session(id=sid)

        if self._rust_agent:
            session._rust_session = self._rust_agent.create_session()

        self._sessions[sid] = session
        return session

    def get_session(self, session_id: str) -> Optional["Session"]:
        """Get a specific session by ID."""
        return self._sessions.get(session_id)

    def set_session(self, session: "Session") -> None:
        """Set the current active session."""
        self._current_session = session
        if session.id not in self._sessions:
            self._sessions[session.id] = session

    def list_sessions(self) -> list:
        """List all sessions."""
        return list(self._sessions.values())

    def register_tool(
        self,
        name: str,
        handler: Callable,
        description: Optional[str] = None,
        parameters: Optional[Dict[str, Any]] = None,
    ) -> None:
        """
        Register a tool for function calling.

        Args:
            name: Tool name
            handler: Tool handler function
            description: Tool description (for LLM)
            parameters: JSON Schema for parameters (for LLM)
        """
        self._tools[name] = handler

        # Add tool definition for LLM function calling
        if description and parameters:
            self._tool_definitions.append(
                ToolDefinition(
                    name=name,
                    description=description,
                    parameters=parameters,
                )
            )

    def call_tool(self, name: str, args: Dict[str, Any]) -> Any:
        """
        Execute a registered tool.

        Args:
            name: Tool name
            args: Tool arguments

        Returns:
            Tool execution result

        Raises:
            ValueError: If tool is not registered
        """
        if name not in self._tools:
            raise ValueError(f"Tool not found: {name}")

        try:
            result = self._tools[name](**args)

            # Record tool usage in session
            if self._current_session:
                self._current_session.record_tool_use(name)

            return result

        except Exception as e:
            raise RuntimeError(f"Tool '{name}' execution failed: {e}") from e

    def list_tools(self) -> list:
        """List registered tool names."""
        return list(self._tools.keys())

    def clear_tools(self) -> None:
        """Clear all registered tools."""
        self._tools.clear()
        self._tool_definitions.clear()

    async def close(self) -> None:
        """Close the agent and release resources."""
        if self._llm_client:
            await self._llm_client.close()
            self._llm_client = None

    def __repr__(self) -> str:
        return f"Agent(name={self._name}, state={self._state.value}, model={self._config.model})"


def create_agent(name: str = "default", **kwargs) -> Agent:
    """
    Convenience function to create an Agent.

    Args:
        name: Agent name
        **kwargs: Configuration parameters

    Returns:
        Agent instance
    """
    config = AgentConfig(name=name, **kwargs)
    return Agent(name=name, config=config)