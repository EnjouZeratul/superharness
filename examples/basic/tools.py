"""
Custom Tools - 自定义工具示例

运行方式:
    python tools.py

预期输出:
    Agent 使用自定义工具完成任务
"""

import asyncio
from continuum import Agent


def basic_tools():
    """基础工具注册示例"""
    print("=== 基础工具示例 ===")

    agent = Agent()

    # 注册自定义工具 (使用 register_tool 方法)
    agent.register_tool(
        name="calculate",
        handler=lambda expression: f"计算结果: {eval(expression)}" if expression else "空表达式",
        description="计算数学表达式",
        parameters={
            "type": "object",
            "properties": {
                "expression": {"type": "string", "description": "数学表达式如 '2+3*4'"}
            },
            "required": ["expression"]
        }
    )

    agent.register_tool(
        name="get_time",
        handler=lambda: f"当前时间: {__import__('datetime').datetime.now().isoformat()}",
        description="获取当前时间",
        parameters={"type": "object", "properties": {}}
    )

    print(f"已注册工具: {list(agent._tools.keys())}")
    print(f"工具调用测试: {agent.call_tool('calculate', {'expression': '2*3+5'})}")
    print(f"工具调用测试: {agent.call_tool('get_time', {})}")


def tool_with_schema():
    """带 schema 的工具示例"""
    print("=== 带 Schema 的工具示例 ===")

    agent = Agent()

    # 定义完整的工具 schema
    agent.register_tool(
        name="search_files",
        handler=lambda pattern, path=".": f"搜索 '{pattern}' 找到 3 个文件",
        description="搜索匹配模式的文件",
        parameters={
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "文件名模式，如 '*.py'"
                },
                "path": {
                    "type": "string",
                    "description": "搜索路径，默认当前目录"
                }
            },
            "required": ["pattern"]
        }
    )

    result = agent.call_tool("search_files", {"pattern": "*.py"})
    print(f"搜索结果: {result}")


def tool_categories():
    """工具分类示例"""
    print("=== 工具分类示例 ===")

    agent = Agent()

    # 安全工具
    agent.register_tool(
        name="read_info",
        handler=lambda: f"系统: {__import__('platform').system()}",
        description="读取系统信息",
        parameters={"type": "object", "properties": {}}
    )

    print(f"系统信息: {agent.call_tool('read_info', {})}")


if __name__ == "__main__":
    basic_tools()
    print("\n" + "=" * 50 + "\n")
    tool_with_schema()
    print("\n" + "=" * 50 + "\n")
    tool_categories()