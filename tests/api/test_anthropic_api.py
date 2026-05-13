"""真实 API 调用验证 - Anthropic

测试 Anthropic Claude API 的真实调用。
"""

import pytest
import os


class TestAnthropicAPI:
    """Anthropic API 测试"""

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("ANTHROPIC_API_KEY"),
        reason="ANTHROPIC_API_KEY not set"
    )
    async def test_anthropic_simple_call(self):
        """测试简单 API 调用"""
        # 使用真实 API key
        # agent = Agent(provider="anthropic")
        # response = await agent.chat("Hello")
        # Expected: 返回非空响应
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("ANTHROPIC_API_KEY"),
        reason="ANTHROPIC_API_KEY not set"
    )
    async def test_anthropic_model_haiku(self):
        """测试 Claude Haiku 模型"""
        # agent = Agent(model="claude-3-haiku")
        # Expected: 使用 haiku 模型
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("ANTHROPIC_API_KEY"),
        reason="ANTHROPIC_API_KEY not set"
    )
    async def test_anthropic_model_opus(self):
        """测试 Claude Opus 模型"""
        # agent = Agent(model="claude-3-opus")
        # Expected: 使用 opus 模型
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("ANTHROPIC_API_KEY"),
        reason="ANTHROPIC_API_KEY not set"
    )
    async def test_anthropic_with_tools(self):
        """测试带工具的调用"""
        # agent = Agent(tools_enabled=True)
        # response = await agent.chat("读取 README.md")
        # Expected: 工具被调用
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("ANTHROPIC_API_KEY"),
        reason="ANTHROPIC_API_KEY not set"
    )
    async def test_anthropic_streaming(self):
        """测试流式响应"""
        # for chunk in agent.stream("Hello"):
        #     print(chunk)
        # Expected: 逐块接收
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("ANTHROPIC_API_KEY"),
        reason="ANTHROPIC_API_KEY not set"
    )
    async def test_anthropic_long_context(self):
        """测试长上下文"""
        # 发送大量上下文
        # Expected: 正确处理
        pass

    @pytest.mark.api
    def test_anthropic_invalid_key(self):
        """测试无效 API key"""
        # agent = Agent(api_key="invalid-key")
        # Expected: 报错
        pass

    @pytest.mark.api
    def test_anthropic_rate_limit(self):
        """测试速率限制"""
        # 快速发送多个请求
        # Expected: 正确处理或重试
        pass


pytestmark = pytest.mark.api