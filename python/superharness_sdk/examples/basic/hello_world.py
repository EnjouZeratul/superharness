"""
Hello World - 最简单的 Quick Start 示例

目标：3步启动 Agent
"""

import sys
import os
# Add SDK to path (开发时使用，pip install 后不需要)
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(__file__)))))

from superharness_sdk import Agent

# Step 1: 导入 (已在上方)

# Step 2: 创建 Agent
agent = Agent()

# Step 3: 运行任务
result = agent.run("hello")
print(f"Result: {result}")

# 可选：继续对话
response = agent.chat("你好，请介绍一下你自己")
print(f"Agent: {response}")
