"""
LLM Integration Tests - 真实 API 调用测试

运行方式：
    pytest python/tests/test_llm_integration.py -v -m integration

    # 或运行所有集成测试
    pytest -m integration --tb=short -v

环境变量（至少设置一个）：
    CONTINUUM_API_KEY      # 统一密钥
    ANTHROPIC_API_KEY      # Anthropic Claude
    OPENAI_API_KEY         # OpenAI GPT
    GOOGLE_API_KEY         # Google Gemini
    DEEPSEEK_API_KEY       # DeepSeek
    TOGETHER_API_KEY       # Together AI
    GROQ_API_KEY           # Groq
"""

import os
import pytest
import asyncio

# 跳过导入路径设置
import sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from continuum_sdk.llm import LlmClient, Message, ChatResponse
from continuum_sdk.agent import Agent


# ==================== Fixtures ====================

@pytest.fixture
def anthropic_key():
    """获取 Anthropic API Key"""
    key = os.environ.get("CONTINUUM_API_KEY") or os.environ.get("ANTHROPIC_API_KEY")
    if not key:
        pytest.skip("ANTHROPIC_API_KEY not set")
    return key


@pytest.fixture
def openai_key():
    """获取 OpenAI API Key"""
    key = os.environ.get("CONTINUUM_API_KEY") or os.environ.get("OPENAI_API_KEY")
    if not key:
        pytest.skip("OPENAI_API_KEY not set")
    return key


@pytest.fixture
def deepseek_key():
    """获取 DeepSeek API Key"""
    key = os.environ.get("DEEPSEEK_API_KEY")
    if not key:
        pytest.skip("DEEPSEEK_API_KEY not set")
    return key


# ==================== Anthropic Tests ====================

@pytest.mark.integration
@pytest.mark.asyncio
async def test_anthropic_chat_real(anthropic_key):
    """测试 Anthropic Claude 真实 API 调用"""
    # 使用腾讯云代理或原生 Anthropic API
    base_url = os.environ.get("ANTHROPIC_BASE_URL", "https://api.anthropic.com")

    # 腾讯云代理使用 hunyuan 模型，原生 API 使用 claude
    if "tencent" in base_url or "lkeap" in base_url:
        model = os.environ.get("CONTINUUM_MODEL", "hunyuan-turbos")
    else:
        model = os.environ.get("CONTINUUM_MODEL", "claude-sonnet-4-6")

    client = LlmClient.for_provider(
        provider="anthropic",
        api_key=anthropic_key,
        base_url=base_url,
        model=model
    )

    messages = [Message.user("Say 'Hello, Continuum!' and nothing else.")]

    response = await client.chat(messages)

    assert isinstance(response, ChatResponse)
    assert response.content
    assert len(response.content) > 0
    print(f"\n✓ Response: {response.content[:100]}...")
    print(f"  Model: {response.model}")
    print(f"  Tokens: {response.usage.input_tokens} in / {response.usage.output_tokens} out")


@pytest.mark.integration
@pytest.mark.asyncio
async def test_anthropic_chat_stream_real(anthropic_key):
    """测试 Anthropic 流式响应"""
    base_url = os.environ.get("ANTHROPIC_BASE_URL", "https://api.anthropic.com")
    model = os.environ.get("CONTINUUM_MODEL", "hunyuan-turbos" if "tencent" in base_url or "lkeap" in base_url else "claude-sonnet-4-6")

    client = LlmClient.for_provider(
        provider="anthropic",
        api_key=anthropic_key,
        base_url=base_url,
        model=model
    )

    messages = [Message.user("Count from 1 to 5, one number per line.")]

    chunks = []
    async for chunk in client.chat_stream(messages):
        if chunk.content:
            chunks.append(chunk.content)

    full_content = "".join(chunks)
    assert len(full_content) > 0
    print(f"\n✓ Stream response: {full_content[:50]}...")


# ==================== OpenAI Tests ====================

@pytest.mark.integration
@pytest.mark.asyncio
async def test_openai_chat_real(openai_key):
    """测试 OpenAI GPT 真实 API 调用"""
    client = LlmClient.for_provider(
        provider="openai",
        api_key=openai_key,
        model="gpt-4.1-mini"
    )

    messages = [Message.user("Say 'Hello from OpenAI!' and nothing else.")]

    response = await client.chat(messages)

    assert isinstance(response, ChatResponse)
    assert response.content
    print(f"\n✓ OpenAI response: {response.content[:100]}...")


# ==================== DeepSeek Tests ====================

@pytest.mark.integration
@pytest.mark.asyncio
async def test_deepseek_chat_real(deepseek_key):
    """测试 DeepSeek 真实 API 调用 (OpenAI 兼容格式)"""
    client = LlmClient.for_provider(
        provider="deepseek",
        api_key=deepseek_key,
        model="deepseek-chat"
    )

    messages = [Message.user("你好，请简短回复")]

    response = await client.chat(messages)

    assert isinstance(response, ChatResponse)
    assert response.content
    print(f"\n✓ DeepSeek response: {response.content[:100]}...")


# ==================== Agent Integration ====================

@pytest.mark.integration
@pytest.mark.asyncio
async def test_agent_fix_buggy_code(anthropic_key):
    """测试 Agent 修复 buggy_program.py"""
    from continuum_sdk.agent.runtime import AgentConfig

    base_url = os.environ.get("ANTHROPIC_BASE_URL", "https://api.anthropic.com")
    model = os.environ.get("CONTINUUM_MODEL", "hunyuan-turbos" if "tencent" in base_url or "lkeap" in base_url else "claude-sonnet-4-6")

    # 读取测试文件
    buggy_file = os.path.join(
        os.path.dirname(os.path.dirname(os.path.dirname(__file__))),
        "test",
        "buggy_program.py"
    )

    if not os.path.exists(buggy_file):
        pytest.skip("buggy_program.py not found")

    with open(buggy_file, 'r', encoding='utf-8') as f:
        buggy_code = f.read()

    config = AgentConfig(
        provider="anthropic",
        api_key=anthropic_key,
        base_url=base_url,
        model=model
    )

    agent = Agent(config=config)

    # 使用 Agent 分析代码
    task = f"""Analyze this Python code and list all bugs you find:

```python
{buggy_code}
```

List each bug with:
1. Function name
2. Bug description
3. How to fix it

Keep response under 200 words."""

    response = agent.run(task)

    assert response
    assert len(response) > 50
    print(f"\n✓ Agent analysis:\n{response[:500]}...")


@pytest.mark.integration
@pytest.mark.asyncio
async def test_custom_provider_openai_format(anthropic_key):
    """测试自定义提供商（使用 Anthropic 格式）"""
    base_url = os.environ.get("ANTHROPIC_BASE_URL", "https://api.anthropic.com")

    # 使用腾讯云代理作为自定义提供商示例
    client = LlmClient.for_provider(
        provider="my-custom-provider",
        api_key=anthropic_key,
        base_url=base_url,
        model="hunyuan-turbos",
        api_format="anthropic"
    )

    messages = [Message.user("Reply with just: 'Custom provider works!'")]

    response = await client.chat(messages)

    assert response.content
    print(f"\n✓ Custom provider response: {response.content}")


# ==================== Error Handling ====================

@pytest.mark.integration
@pytest.mark.asyncio
async def test_invalid_api_key():
    """测试无效 API Key 的错误处理"""
    from continuum_sdk.llm.errors import AuthenticationError

    client = LlmClient.for_provider(
        provider="anthropic",
        api_key="invalid-key-12345",
        model="claude-sonnet-4-6"
    )

    messages = [Message.user("Hello")]

    # 应该抛出认证错误
    with pytest.raises(AuthenticationError):
        await client.chat(messages)

    print("\n✓ Invalid key correctly raised AuthenticationError")


# ==================== Run Directly ====================

if __name__ == "__main__":
    """直接运行测试"""
    print("=" * 60)
    print("Continuum SDK - Real API Integration Tests")
    print("=" * 60)

    # 检查环境变量
    keys = {
        "ANTHROPIC_API_KEY": os.environ.get("ANTHROPIC_API_KEY"),
        "OPENAI_API_KEY": os.environ.get("OPENAI_API_KEY"),
        "DEEPSEEK_API_KEY": os.environ.get("DEEPSEEK_API_KEY"),
    }

    print("\n环境变量状态:")
    for name, value in keys.items():
        status = "✓ 已设置" if value else "✗ 未设置"
        print(f"  {name}: {status}")

    if not any(keys.values()):
        print("\n❌ 未找到任何 API Key，请设置环境变量后重试")
        print("\n示例:")
        print('  export ANTHROPIC_API_KEY="your-key-here"')
        print('  pytest python/tests/test_llm_integration.py -v -m integration')
        sys.exit(1)

    print("\n运行测试...")
    pytest.main([__file__, "-v", "-m", "integration", "--tb=short"])
