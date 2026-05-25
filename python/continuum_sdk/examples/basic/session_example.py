"""会话管理示例

展示会话创建、保存、恢复等功能。
"""

import asyncio

from continuum_sdk import Session


async def main():
    print("=== 会话管理示例 ===\n")

    # 1. 创建新会话
    print("1. 创建新会话")
    session = Session()
    print(f"   会话 ID: {session.id}")
    print(f"   消息数: {session.message_count}\n")

    # 2. 添加消息
    print("2. 添加消息")
    session.add_user_message("第一条消息")
    session.add_assistant_message("收到第一条消息")
    session.add_user_message("第二条消息")

    messages = session.get_messages()
    print(f"   消息数量: {len(messages)}")
    for msg in messages:
        print(f"   - [{msg.role}]: {msg.content[:30]}...")
    print()

    # 3. 保存会话
    print("3. 保存会话")
    path = session.save_to_default()
    print(f"   保存路径: {path}\n")

    # 4. 会话信息
    print("4. 会话信息")
    print(f"   总成本: ${session.cost:.4f}")
    print(f"   Token 数: {session.tokens}")
    print(f"   使用工具: {session.get_tools_used()}\n")

    # 5. 列出所有会话
    print("5. 列出所有会话")
    sessions = Session.list_saved_sessions()
    for s in sessions[:5]:  # 只显示前5个
        print(f"   - {s}")
    print()

    # 6. 加载会话
    print("6. 加载会话")
    loaded = Session.load_from_default(session.id)
    print(f"   加载成功，消息数: {loaded.message_count}\n")

    # 7. 导出会话
    print("7. 导出会话")
    export_data = session.export()
    print(f"   导出数据长度: {len(export_data)} 字符\n")


if __name__ == "__main__":
    asyncio.run(main())