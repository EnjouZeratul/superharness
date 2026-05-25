"""
Intelligent Agent - 智能 Agent 示例

智能 Agent 提供:
- 任务规划 (Planner)
- 自校正机制 (SelfCorrection)
- 进度跟踪 (ProgressTracker)

运行方式:
    python intelligent_agent.py

预期输出:
- 任务规划步骤
- 执行进度
- 结果摘要
"""

import asyncio
from continuum_sdk.agent import (
    IntelligentAgent,
    AgentMode,
    ExecutionResult,
)


async def basic_intelligent_agent():
    """基础智能 Agent 示例"""
    print("=== 基础智能 Agent 示例 ===")

    # 创建智能 Agent（交互模式）
    agent = IntelligentAgent(
        api_key="your-api-key",  # 或使用环境变量
        mode=AgentMode.INTERACTIVE
    )

    # 规划任务
    print("\n规划任务: 修复 login.py 中的空指针 bug")
    plan = await agent.plan("修复 login.py 中的空指针 bug")

    # 显示规划
    print(f"\n任务规划:")
    for step in plan.steps:
        print(f"  [{step.id}] {step.type.value}: {step.description}")

    # 执行规划
    result = await agent.execute(plan)

    # 显示结果
    print(f"\n执行结果:")
    print(f"  状态: {result.status}")
    print(f"  完成步骤: {result.completed_steps}/{result.total_steps}")
    print(f"  持续时间: {result.duration_seconds:.2f}s")


async def autonomous_mode():
    """自主模式示例"""
    print("=== 自主模式示例 ===")

    agent = IntelligentAgent(mode=AgentMode.AUTONOMOUS)

    # 一键执行：规划 + 执行
    result = await agent.run("添加日志记录到 user_service.py")

    print(f"\n一键执行结果:")
    print(f"  任务: {result.task}")
    print(f"  状态: {result.status}")
    print(f"  校正次数: {result.corrections_applied}")


async def step_by_step_mode():
    """逐步模式示例"""
    print("=== 逐步模式示例 ===")

    agent = IntelligentAgent(mode=AgentMode.STEP_BY_STEP)

    plan = await agent.plan("重构 parser 模块")

    # 自定义回调控制每步
    def on_step_start(step):
        print(f"\n即将执行: {step.description}")
        print(f"操作: {step.action}")
        # 返回 True 继续执行，返回 False 跳过
        return True

    def on_step_complete(step):
        print(f"  完成: {step.id}")

    result = await agent.execute(
        plan,
        on_step_start=on_step_start,
        on_step_complete=on_step_complete
    )

    print(f"\n完成，共 {result.completed_steps} 步")


async def progress_tracking():
    """进度跟踪示例"""
    print("=== 进度跟踪示例 ===")

    agent = IntelligentAgent(mode=AgentMode.AUTONOMOUS)

    # 注册进度回调
    def on_progress(event):
        print(f"  进度: {event.progress_percent:.0f}% | {event.status}")

    agent.tracker.on_progress(on_progress)

    plan = await agent.plan("优化数据库查询性能")
    result = await agent.execute(plan)

    # 获取进度摘要
    print(agent.get_progress_text())
    print(agent.get_plan_summary())


async def error_recovery():
    """错误恢复示例"""
    print("=== 错误恢复示例 ===")

    agent = IntelligentAgent(mode=AgentMode.AUTONOMOUS)

    plan = await agent.plan("读取配置文件并验证")

    # 错误回调
    def on_error(step, error_ctx):
        print(f"  步骤 {step.id} 出错: {error_ctx.error_type.value}")
        print(f"  策略: {error_ctx.strategy if hasattr(error_ctx, 'strategy') else 'auto'}")
        # 返回 False 终止执行
        return True

    result = await agent.execute(plan, on_error=on_error)

    print(f"\n校正应用次数: {result.corrections_applied}")


async def main():
    """运行所有示例"""
    await basic_intelligent_agent()
    print("\n" + "=" * 50 + "\n")

    await autonomous_mode()
    print("\n" + "=" * 50 + "\n")

    await progress_tracking()


if __name__ == "__main__":
    asyncio.run(main())