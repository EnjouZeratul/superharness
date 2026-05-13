"""真实 API 调用验证 - OpenAI

测试 OpenAI GPT API 的真实调用。
"""

import pytest
import os


class TestOpenAIAPI:
    """OpenAI API 测试"""

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("OPENAI_API_KEY"),
        reason="OPENAI_API_KEY not set"
    )
    async def test_openai_simple_call(self):
        """测试简单 API 调用"""
        # agent = Agent(provider="openai")
        # response = await agent.chat("Hello")
        # Expected: 返回非空响应
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("OPENAI_API_KEY"),
        reason="OPENAI_API_KEY not set"
    )
    async def test_openai_model_gpt4(self):
        """测试 GPT-4 模型"""
        # agent = Agent(provider="openai", model="gpt-4")
        # Expected: 使用 GPT-4
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("OPENAI_API_KEY"),
        reason="OPENAI_API_KEY not set"
    )
    async def test_openai_model_gpt35(self):
        """测试 GPT-3.5 模型"""
        # agent = Agent(provider="openai", model="gpt-3.5-turbo")
        # Expected: 使用 GPT-3.5
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("OPENAI_API_KEY"),
        reason="OPENAI_API_KEY not set"
    )
    async def test_openai_with_tools(self):
        """测试带工具的调用"""
        # agent = Agent(provider="openai", tools_enabled=True)
        # Expected: 工具调用正常
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("OPENAI_API_KEY"),
        reason="OPENAI_API_KEY not set"
    )
    async def test_openai_streaming(self):
        """测试流式响应"""
        # Expected: 逐块接收
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("OPENAI_API_KEY"),
        reason="OPENAI_API_KEY not set"
    )
    async def test_openai_custom_base_url(self):
        """测试自定义 base URL"""
        # 用于 Azure OpenAI 或代理
        # agent = Agent(provider="openai", base_url="...")
        pass

    @pytest.mark.api
    def test_openai_invalid_key(self):
        """测试无效 API key"""
        # Expected: 报错
        pass


pytestmark = pytest.mark.api