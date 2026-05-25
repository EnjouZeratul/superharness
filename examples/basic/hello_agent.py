"""
Hello Agent - 最简单的 Continuum Agent 示例

运行方式:
    python hello_agent.py

预期输出:
    Agent initialized...
    Response: [AI 响应内容]
"""

import asyncio
from continuum import Agent


def main():
    """基础 Agent 使用示例"""
    print("=== Hello Agent 示例 ===")

    # 1. 创建 Agent（自动从环境变量加载配置）
    # 需要设置: ANTHROPIC_API_KEY 或 CONTINUUM_API_KEY
    agent = Agent()
    print("Agent initialized...")

    # 2. 执行简单任务
    result = agent.run("你好，请用一句话介绍你自己")
    print(f"Response: {result}")


def main_async():
    """异步 Agent 示例"""
    print("=== 异步 Hello Agent 示例 ===")

    async def run():
        agent = Agent()

        # 流式响应
        async for chunk in agent.run_stream("你好"):
            print(chunk.content, end="", flush=True)
        print()  # newline

    asyncio.run(run())


if __name__ == "__main__":
    main()