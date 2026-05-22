"""会话管理示例

展示会话创建、保存、恢复等功能。
"""

import asyncio
from continuum_sdk import SessionManager


async def main():
    manager = SessionManager()

    print("=== 会话管理示例 ===\n")

    # 1. 创建新会话
    print("1. 创建新会话")
    session = await manager.create_session(
        name="demo-session",
        working_dir="."
    )
    print(f"   会话 ID: {session.id}")
    print(f"   会话名称: {session.name}")
    print(f"   状态: {session.state}\n")

    # 2. 获取历史记录
    print("2. 操作会话")
    # 模拟一些对话
    await session.add_message("user", "第一条消息")
    await session.add_message("assistant", "收到第一条消息")
    await session.add_message("user", "第二条消息")

    history = session.history()
    print(f"   消息数量: {len(history)}")
    for msg in history:
        print(f"   - [{msg['role']}]: {msg['content'][:30]}...")
    print()

    # 3. 保存检查点
    print("3. 保存检查点")
    checkpoint_id = await session.save_checkpoint(name="before-rollback")
    print(f"   检查点 ID: {checkpoint_id}\n")

    # 4. 添加更多消息
    await session.add_message("user", "第三条消息（将被回滚）")
    print("4. 添加了第三条消息")
    print(f"   当前消息数: {len(session.history())}\n")

    # 5. 回滚到检查点
    print("5. 回滚到检查点")
    await session.rollback(checkpoint_id)
    print(f"   回滚后消息数: {len(session.history())}\n")

    # 6. 列出所有会话
    print("6. 列出所有会话")
    sessions = await manager.list_sessions()
    for s in sessions:
        print(f"   - {s['id']}: {s['name']} ({s['state']})")
    print()

    # 7. 结束会话
    print("7. 结束会话")
    await session.end()
    print(f"   最终状态: {session.state}\n")


if __name__ == "__main__":
    asyncio.run(main())
