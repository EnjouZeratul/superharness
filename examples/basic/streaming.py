"""
Streaming Response - 流式响应示例

运行方式:
    python streaming.py

预期输出:
    逐字打印的 AI 响应
"""

import asyncio
from continuum import Agent


async def stream_example():
    """流式响应示例"""
    print("=== 流式响应示例 ===")

    agent = Agent()
    print("输入: 请写一首关于程序员的短诗")
    print("响应: ", end="", flush=True)

    # 流式获取响应
    async for chunk in agent.run_stream("请写一首关于程序员的短诗"):
        print(chunk.content, end="", flush=True)

    print("\n")  # end with newline


async def stream_with_callback():
    """带回调的流式响应"""
    print("=== 带回调的流式响应 ===")

    tokens = []

    def on_token(token: str):
        tokens.append(token)
        print(token, end="", flush=True)

    agent = Agent()

    await agent.run_stream(
        "解释什么是递归，用简单的语言",
        on_token=on_token
    )

    print(f"\n总共收到 {len(tokens)} 个 token")


async def concurrent_streams():
    """并发流式请求"""
    print("=== 并发流式请求 ===")

    agent = Agent()

    # 并发执行多个流式任务
    tasks = [
        agent.run_stream("说一个数字"),
        agent.run_stream("说一个颜色"),
        agent.run_stream("说一个水果"),
    ]

    results = await asyncio.gather(*tasks, return_exceptions=True)

    for i, result in enumerate(results):
        if isinstance(result, Exception):
            print(f"任务 {i+1} 失败: {result}")
        else:
            print(f"任务 {i+1} 完成")


if __name__ == "__main__":
    asyncio.run(stream_example())