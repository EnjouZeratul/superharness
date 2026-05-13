"""真实 API 调用验证 - Gemini

测试 Google Gemini API 的真实调用。
"""

import pytest
import os


class TestGeminiAPI:
    """Gemini API 测试"""

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("GEMINI_API_KEY"),
        reason="GEMINI_API_KEY not set"
    )
    async def test_gemini_simple_call(self):
        """测试简单 API 调用"""
        # agent = Agent(provider="gemini")
        # response = await agent.chat("Hello")
        # Expected: 返回非空响应
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("GEMINI_API_KEY"),
        reason="GEMINI_API_KEY not set"
    )
    async def test_gemini_model_pro(self):
        """测试 Gemini Pro 模型"""
        # agent = Agent(provider="gemini", model="gemini-pro")
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("GEMINI_API_KEY"),
        reason="GEMINI_API_KEY not set"
    )
    async def test_gemini_model_flash(self):
        """测试 Gemini Flash 模型"""
        # agent = Agent(provider="gemini", model="gemini-flash")
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("GEMINI_API_KEY"),
        reason="GEMINI_API_KEY not set"
    )
    async def test_gemini_streaming(self):
        """测试流式响应"""
        pass

    @pytest.mark.api
    def test_gemini_invalid_key(self):
        """测试无效 API key"""
        # Expected: 报错
        pass


pytestmark = pytest.mark.api