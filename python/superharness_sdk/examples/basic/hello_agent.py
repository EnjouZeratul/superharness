"""Hello Agent 示例

最简单的 Agent 使用示例。
"""

import asyncio
from superharness_sdk import Agent, SessionManager


async def main():
    # 创建会话
    session = await SessionManager().create_session(
        name="hello-session",
        working_dir="."
    )

    # 创建 Agent
    agent = Agent(session_id=session.id)

    print("=== Hello Agent 示例 ===\n")

    # 发送消息
    response = await agent.chat("你好！请介绍一下你自己。")
    print(f"Agent: {response}\n")

    # 继续对话
    response = await agent.chat("你能做什么？")
    print(f"Agent: {response}\n")

    # 结束会话
    await session.end()
    print("会话已结束")


if __name__ == "__main__":
    asyncio.run(main())
