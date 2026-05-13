"""
Agent Unit Tests

Tests for Agent and AgentConfig with mock LLM responses.
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from unittest.mock import AsyncMock, patch, MagicMock
import asyncio

from superharness_sdk.agent import Agent, AgentConfig, AgentState
from superharness_sdk.llm import ChatResponse, TokenUsage


class TestAgentConfig:
    """AgentConfig tests"""

    def test_default_config(self):
        """Test default configuration"""
        config = AgentConfig()
        assert config.name == "default"
        assert config.model == "claude-sonnet-4-6"
        assert config.provider == "anthropic"
        assert config.max_tokens == 4096
        assert config.temperature == 0.7

    def test_custom_config(self):
        """Test custom configuration"""
        config = AgentConfig(
            name="custom",
            model="gpt-4",
            provider="openai",
            max_tokens=8192,
            temperature=0.5
        )
        assert config.name == "custom"
        assert config.model == "gpt-4"
        assert config.provider == "openai"
        assert config.max_tokens == 8192
        assert config.temperature == 0.5

    def test_config_to_dict(self):
        """Test config serialization"""
        config = AgentConfig(name="test")
        data = config.to_dict()
        assert isinstance(data, dict)
        assert data["name"] == "test"

    def test_config_from_dict(self):
        """Test config deserialization"""
        data = {"name": "loaded", "model": "gpt-4", "provider": "openai"}
        config = AgentConfig.from_dict(data)
        assert config.name == "loaded"
        assert config.model == "gpt-4"
        assert config.provider == "openai"


class TestAgentState:
    """Agent state tests"""

    def test_agent_creation(self):
        """Test Agent creation"""
        agent = Agent(api_key="test-key")
        assert agent.name == "default"
        assert agent.state == AgentState.IDLE

    def test_agent_with_name(self):
        """Test named Agent"""
        agent = Agent(name="my-agent", api_key="test-key")
        assert agent.name == "my-agent"

    def test_agent_start(self):
        """Test Agent start"""
        agent = Agent(api_key="test-key")
        agent.start()
        assert agent.state == AgentState.RUNNING

    def test_agent_pause(self):
        """Test Agent pause"""
        agent = Agent(api_key="test-key")
        agent.start()
        agent.pause()
        assert agent.state == AgentState.PAUSED

    def test_agent_stop(self):
        """Test Agent stop"""
        agent = Agent(api_key="test-key")
        agent.start()
        agent.stop()
        assert agent.state == AgentState.IDLE

    def test_agent_double_start(self):
        """Test double start raises error"""
        agent = Agent(api_key="test-key")
        agent.start()
        with pytest.raises(RuntimeError):
            agent.start()

    def test_agent_pause_not_running(self):
        """Test pause when not running raises error"""
        agent = Agent(api_key="test-key")
        with pytest.raises(RuntimeError):
            agent.pause()


class TestAgentTools:
    """Agent tool tests"""

    def test_agent_register_tool(self):
        """Test tool registration"""
        agent = Agent(api_key="test-key")
        agent.register_tool("test_tool", lambda x: x)
        assert "test_tool" in agent.list_tools()

    def test_agent_call_tool(self):
        """Test tool execution"""
        agent = Agent(api_key="test-key")
        agent.register_tool("add", lambda a, b: a + b)
        result = agent.call_tool("add", {"a": 1, "b": 2})
        assert result == 3

    def test_agent_call_missing_tool(self):
        """Test calling missing tool raises error"""
        agent = Agent(api_key="test-key")
        with pytest.raises(ValueError):
            agent.call_tool("missing", {})

    def test_agent_register_tool_with_definition(self):
        """Test tool registration with LLM definition"""
        agent = Agent(api_key="test-key")
        agent.register_tool(
            "search",
            lambda query: f"results for {query}",
            description="Search for information",
            parameters={"type": "object", "properties": {"query": {"type": "string"}}}
        )
        assert "search" in agent.list_tools()
        assert len(agent._tool_definitions) == 1


class TestAgentSession:
    """Agent session tests"""

    def test_agent_create_session(self):
        """Test session creation"""
        agent = Agent(api_key="test-key")
        session = agent.create_session()
        assert session is not None
        assert session.id is not None

    def test_agent_get_session(self):
        """Test session retrieval"""
        agent = Agent(api_key="test-key")
        session = agent.create_session("test-session")
        retrieved = agent.get_session("test-session")
        assert retrieved is session

    def test_agent_list_sessions(self):
        """Test listing sessions"""
        agent = Agent(api_key="test-key")
        agent.create_session("s1")
        agent.create_session("s2")
        sessions = agent.list_sessions()
        assert len(sessions) == 2


class TestAgentExecute:
    """Agent execute tests with mocked LLM"""

    @pytest.mark.asyncio
    async def test_execute_async_with_mock(self):
        """Test async execution with mocked LLM"""
        # Create mock response
        mock_response = ChatResponse(
            content="Hello! How can I help you?",
            model="claude-sonnet-4-6",
            usage=TokenUsage(input_tokens=10, output_tokens=20),
        )

        # Create agent with mock
        agent = Agent(api_key="test-key", model="claude-sonnet-4-6")
        agent.start()

        # Mock the LLM client
        with patch.object(agent, '_get_llm_client') as mock_get_client:
            mock_client = MagicMock()
            mock_client.chat = AsyncMock(return_value=mock_response)
            mock_get_client.return_value = mock_client

            result = await agent.execute_async("Hello")

        assert result == "Hello! How can I help you?"

    @pytest.mark.asyncio
    async def test_execute_async_not_running(self):
        """Test execute when not running raises error"""
        agent = Agent(api_key="test-key")
        # Don't start the agent

        with pytest.raises(RuntimeError, match="not running"):
            await agent.execute_async("task")

    def test_execute_not_running(self):
        """Test execute when not running raises error"""
        agent = Agent(api_key="test-key")
        # Don't start the agent

        with pytest.raises(RuntimeError, match="not running"):
            agent.execute("task")

    def test_execute_no_api_key(self):
        """Test execute without API key raises error"""
        agent = Agent()  # No API key
        agent.start()

        with pytest.raises(ValueError, match="API key"):
            agent.execute("task")


class TestAgentQuickStart:
    """Quick Start tests"""

    @pytest.mark.asyncio
    async def test_three_step_start_with_mock(self):
        """Test 3-step start with mocked LLM"""
        mock_response = ChatResponse(
            content="Response from LLM",
            model="claude-sonnet-4-6",
            usage=TokenUsage(input_tokens=5, output_tokens=10),
        )

        agent = Agent(api_key="test-key")

        with patch.object(agent, '_get_llm_client') as mock_get_client:
            mock_client = MagicMock()
            mock_client.chat = AsyncMock(return_value=mock_response)
            mock_get_client.return_value = mock_client

            result = agent.run("hello")

        assert result == "Response from LLM"
        assert agent.state == AgentState.RUNNING

    @pytest.mark.asyncio
    async def test_agent_sequential_calls_with_mock(self):
        """Test sequential calls with mocked LLM"""
        mock_response = ChatResponse(
            content="Task completed",
            model="claude-sonnet-4-6",
            usage=TokenUsage(input_tokens=5, output_tokens=10),
        )

        agent = Agent(api_key="test-key")
        agent.start()

        with patch.object(agent, '_get_llm_client') as mock_get_client:
            mock_client = MagicMock()
            mock_client.chat = AsyncMock(return_value=mock_response)
            mock_get_client.return_value = mock_client

            result1 = agent.run("task1", auto_start=False)
            result2 = agent.chat("task2")

        assert result1 == "Task completed"
        assert result2 == "Task completed"


class TestAgentStreaming:
    """Agent streaming tests"""

    @pytest.mark.asyncio
    async def test_run_stream_with_mock(self):
        """Test streaming with mocked LLM"""
        from superharness_sdk.llm import StreamChunk

        async def mock_stream(*args, **kwargs):
            chunks = [
                StreamChunk(content="Hello"),
                StreamChunk(content=" "),
                StreamChunk(content="world"),
                StreamChunk(finish_reason="stop"),
            ]
            for chunk in chunks:
                yield chunk

        agent = Agent(api_key="test-key")

        with patch.object(agent, '_get_llm_client') as mock_get_client:
            mock_client = MagicMock()
            mock_client.chat_stream = mock_stream
            mock_get_client.return_value = mock_client

            chunks = []
            async for chunk in agent.run_stream("hello"):
                chunks.append(chunk)

        assert len(chunks) == 4
        full_content = "".join(c.content for c in chunks if c.content)
        assert full_content == "Hello world"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])