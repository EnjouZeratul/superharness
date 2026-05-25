"""
Intelligent Agent - 智能 Agent 示例

智能 Agent 提供:
- 任务规划 (Planner)
- 自校正机制 (SelfCorrection)
- 进度跟踪 (ProgressTracker)

运行方式:
    python intelligent_agent.py

预期输出:
- Agent 创建示例
- 模式切换演示
- 进度跟踪器使用

注意: 运行此示例需要设置API密钥环境变量才能进行真实LLM调用
"""

import asyncio
from continuum_sdk.agent import (
    IntelligentAgent,
    AgentMode,
    ExecutionResult,
)
from continuum_sdk.agent.planner import Planner, Plan, Step, StepType, StepStatus
from continuum_sdk.agent.progress import ProgressTracker


def basic_intelligent_agent():
    """基础智能 Agent 示例"""
    print("=== 基础智能 Agent 示例 ===")

    # 创建智能 Agent（交互模式）
    agent = IntelligentAgent(
        api_key="your-api-key",  # 或使用环境变量 ANTHROPIC_API_KEY
        mode=AgentMode.INTERACTIVE
    )

    print(f"Agent 创建成功")
    print(f"  模式: {agent.mode.value}")
    print(f"  提供者: {agent.provider}")


def autonomous_mode():
    """自主模式示例"""
    print("=== 自主模式示例 ===")

    agent = IntelligentAgent(mode=AgentMode.AUTONOMOUS)

    print(f"Agent 模式: {agent.mode.value}")
    print(f"自主模式特点: 自动规划和执行，无需人工干预")


def step_by_step_mode():
    """逐步模式示例"""
    print("=== 逐步模式示例 ===")

    agent = IntelligentAgent(mode=AgentMode.STEP_BY_STEP)

    print(f"Agent 模式: {agent.mode.value}")
    print(f"逐步模式特点: 每步执行前等待确认")


def progress_tracking():
    """进度跟踪示例"""
    print("=== 进度跟踪示例 ===")

    tracker = ProgressTracker()

    # 注册回调
    def on_event(event):
        print(f"  [{event.status}] {event.step_description}: {event.progress_percent:.0f}%")

    tracker.on_progress(on_event)

    # 开始跟踪
    tracker.start(total_steps=4)

    # 模拟步骤完成
    tracker.update_step("step_1", "completed", "步骤1完成")
    tracker.update_step("step_2", "completed", "步骤2完成")
    tracker.update_step("step_3", "completed", "步骤3完成")
    tracker.update_step("step_4", "completed", "步骤4完成")

    # 获取进度摘要
    print(f"\n进度摘要:")
    print(f"  当前状态: {tracker.state.value}")
    print(f"  完成步骤: {tracker.completed_steps}/{tracker.total_steps}")
    print(f"  已用时间: {tracker.get_elapsed_time():.1f}s")


def planner_demo():
    """规划器示例"""
    print("=== 规划器示例 ===")

    # 创建手动规划（无需LLM）
    plan = Plan(
        id="manual_plan",
        task="修复 login.py 中的空指针 bug",
        steps=[
            Step(
                id="step_1",
                type=StepType.READ,
                description="读取 login.py 文件内容",
                action="read login.py",
                status=StepStatus.PENDING
            ),
            Step(
                id="step_2",
                type=StepType.ANALYZE,
                description="分析空指针问题",
                action="analyze null pointer",
                status=StepStatus.PENDING
            ),
            Step(
                id="step_3",
                type=StepType.EDIT,
                description="修复空指针检查",
                action="edit login.py",
                status=StepStatus.PENDING
            ),
        ]
    )

    print(f"\n手动规划:")
    print(f"  任务: {plan.task}")
    print(f"  步骤数: {len(plan.steps)}")
    for step in plan.steps:
        print(f"    [{step.id}] {step.type.value}: {step.description}")


if __name__ == "__main__":
    basic_intelligent_agent()
    print("\n" + "=" * 50 + "\n")
    autonomous_mode()
    print("\n" + "=" * 50 + "\n")
    step_by_step_mode()
    print("\n" + "=" * 50 + "\n")
    progress_tracking()
    print("\n" + "=" * 50 + "\n")
    planner_demo()