"""
Custom LLM Integration - 自定义 LLM 集成示例

自定义 LLM 提供者支持:
- 多提供商配置
- 自定义端点配置
- 流式响应

运行方式:
    python custom_llm.py

预期输出:
- LLM 配置示例
- 自定义响应处理

注意: 运行此示例需要设置API密钥环境变量
"""

import os
import asyncio
from continuum_sdk.llm.client import BaseLlmClient, LlmClient
from continuum_sdk.llm.types import Message
from continuum_sdk.agent.runtime import Agent, AgentConfig


def basic_custom_llm():
    """基础自定义 LLM 示例"""
    print("=== 基础自定义 LLM 示例 ===")

    # 创建 LLM 配置
    config = AgentConfig(
        provider="openai",
        model="gpt-4",
        api_key=os.environ.get("OPENAI_API_KEY", "your-api-key"),
        temperature=0.7,
        max_tokens=1000,
    )

    print(f"LLM 配置:")
    print(f"  提供者: {config.provider}")
    print(f"  模型: {config.model}")
    print(f"  温度: {config.temperature}")
    print(f"  最大 tokens: {config.max_tokens}")

    # 使用配置创建 Agent
    agent = Agent(config=config)
    print(f"\nAgent 已配置自定义 LLM")


def local_model_integration():
    """本地模型集成示例"""
    print("=== 本地模型集成示例 ===")

    # 本地模型配置（如 llama.cpp, vLLM）
    config = AgentConfig(
        provider="openai",  # OpenAI兼容接口
        model="llama-3-70b",
        base_url="http://localhost:8000/v1",
        api_key="local",
        temperature=0.5,
        max_tokens=2048,
    )

    print(f"本地模型配置:")
    print(f"  端点: {config.base_url}")
    print(f"  模型: {config.model}")

    # 配置 Agent 使用本地模型
    agent = Agent(config=config)
    print(f"\nAgent 已连接本地模型")


def custom_endpoint():
    """自定义端点示例"""
    print("=== 自定义端点示例 ===")

    # 自定义端点配置
    config = AgentConfig(
        provider="openai",
        model="enterprise-model-v1",
        base_url="https://llm.internal.company.com/api",
        api_key="internal-token",
        timeout=60,
    )

    print(f"企业内部端点配置:")
    print(f"  URL: {config.base_url}")
    print(f"  超时: {config.timeout}s")


def model_parameter_tuning():
    """模型参数调优示例"""
    print("=== 模型参数调优示例 ===")

    # 代码生成（精确、低温度）
    code_config = AgentConfig(
        provider="openai",
        model="gpt-4",
        temperature=0.2,
        max_tokens=2000,
    )

    # 创意写作（高温度）
    creative_config = AgentConfig(
        provider="openai",
        model="gpt-4",
        temperature=0.9,
        max_tokens=3000,
    )

    # 分析任务（中等温度）
    analysis_config = AgentConfig(
        provider="openai",
        model="gpt-4",
        temperature=0.5,
        max_tokens=4000,
    )

    print("不同任务的参数配置:")
    print(f"  代码生成: temperature={code_config.temperature}")
    print(f"  创意写作: temperature={creative_config.temperature}")
    print(f"  分析任务: temperature={analysis_config.temperature}")


if __name__ == "__main__":
    basic_custom_llm()
    print("\n" + "=" * 50 + "\n")
    local_model_integration()
    print("\n" + "=" * 50 + "\n")
    custom_endpoint()
    print("\n" + "=" * 50 + "\n")
    model_parameter_tuning()
