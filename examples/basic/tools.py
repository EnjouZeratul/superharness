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

    # 注册自定义工具
    @agent.tool
    def calculate(expression: str) -> str:
        """计算数学表达式"""
        try:
            result = eval(expression)
            return f"计算结果: {result}"
        except Exception as e:
            return f"计算错误: {e}"

    @agent.tool
    def get_time() -> str:
        """获取当前时间"""
        from datetime import datetime
        return f"当前时间: {datetime.now().isoformat()}"

    # 使用工具
    result = agent.run("计算 2 * 3 + 5，并告诉我当前时间")
    print(f"Response: {result}")


async def async_tools():
    """异步工具示例"""
    print("=== 异步工具示例 ===")

    agent = Agent()

    @agent.tool
    async def fetch_url(url: str) -> str:
        """异步获取 URL 内容"""
        import httpx
        async with httpx.AsyncClient() as client:
            response = await client.get(url, timeout=10)
            return f"获取到 {len(response.content)} 字节"

    result = await agent.run_async("获取 https://example.com 的内容")
    print(f"Response: {result}")


def tool_with_schema():
    """带 schema 的工具示例"""
    print("=== 带 Schema 的工具示例 ===")

    agent = Agent()

    # 定义完整的工具 schema
    agent.register_tool(
        name="search_files",
        handler=lambda pattern: f"搜索 '{pattern}' 找到 3 个文件",
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

    result = agent.run("搜索所有 Python 文件")
    print(f"Response: {result}")


def tool_categories():
    """工具分类示例"""
    print("=== 工具分类示例 ===")

    agent = Agent()

    # 安全工具
    @agent.tool(category="safe")
    def read_info() -> str:
        """读取系统信息"""
        import platform
        return f"系统: {platform.system()}, Python: {platform.python_version()}"

    # 需确认工具
    @agent.tool(category="confirm")
    def delete_file(path: str) -> str:
        """删除文件（需要确认）"""
        import os
        if os.path.exists(path):
            os.remove(path)
            return f"已删除: {path}"
        return f"文件不存在: {path}"

    # 危险工具
    @agent.tool(category="dangerous")
    def run_command(cmd: str) -> str:
        """执行 shell 命令（危险操作）"""
        import subprocess
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        return result.stdout or result.stderr

    # Agent 会根据工具分类决定是否需要确认
    result = agent.run("告诉我系统信息")
    print(f"Response: {result}")


if __name__ == "__main__":
    basic_tools()