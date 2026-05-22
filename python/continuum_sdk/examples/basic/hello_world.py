"""Hello World - 最简单的 Quick Start 示例

目标：3步启动 Agent
"""

import asyncio
from continuum_sdk import Agent

# Step 1: 导入 (已在上方)

# Step 2: 创建 Agent (auto-configures from environment)
agent = Agent()

# Step 3: 运行任务
result = agent.run("hello")
print(f"Result: {result}")

# 可选：继续对话
response = agent.chat("你好，请介绍一下你自己")
print(f"Agent: {response}")
