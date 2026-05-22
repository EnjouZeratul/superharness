"""Hello Agent 示例

最简单的 Agent 使用示例。
"""

import asyncio
from continuum_sdk import Agent, Session


async def main():
    # 创建会话
    session = Session()
    session.add_user_message("你好！请介绍一下你自己。")

    # 创建 Agent
    agent = Agent()

    print("=== Hello Agent 示例 ===\n")

    # 发送消息
    response = agent.run("你好！请介绍一下你自己。")
    print(f"Agent: {response}\n")
    session.add_assistant_message(response)

    # 继续对话
    response = agent.run("你能做什么？")
    print(f"Agent: {response}\n")
    session.add_assistant_message(response)

    # 保存会话
    session.save_to_default()
    print(f"会话已保存，ID: {session.id}")


if __name__ == "__main__":
    asyncio.run(main())