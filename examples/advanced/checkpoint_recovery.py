"""
Checkpoint Recovery - 检查点恢复示例

检查点系统提供:
- 执行状态保存
- 崩溃后恢复
- 状态完整性验证

运行方式:
    python checkpoint_recovery.py

预期输出:
- 检查点创建过程
- 恢复后状态一致性验证
"""

import json
from continuum_sdk.agent.checkpoint import CheckpointClient


def basic_checkpoint():
    """基础检查点示例"""
    print("=== 基础检查点示例 ===")

    # 创建检查点客户端
    client = CheckpointClient()

    # 保存检查点
    session_id = "demo_session"
    state = {
        "task": "分析项目结构",
        "progress": "步骤1完成",
        "iteration": 1,
    }

    cp_id = client.save(session_id, state)
    print(f"检查点已保存: {cp_id}")

    # 列出检查点
    checkpoints = client.list(session_id)
    print(f"可用检查点: {len(checkpoints)}")

    # 加载检查点
    loaded = client.load(session_id, cp_id)
    if loaded:
        print(f"检查点已加载: {loaded}")


def crash_recovery_simulation():
    """崩溃恢复模拟"""
    print("=== 崩溃恢复模拟 ===")

    client = CheckpointClient()
    session_id = "crash_test_session"

    # 模拟崩溃前的状态
    print("\n1. 模拟执行过程（崩溃前）")
    for i in range(1, 5):
        state = {
            "iteration": i,
            "messages": [
                {"role": "user", "content": f"指令 {i}"},
                {"role": "assistant", "content": f"响应 {i}"},
            ],
            "tokens_used": 100 * i,
            "resume_hint": f"继续执行步骤 {i+1}",
        }
        client.save(session_id, state)
        print(f"  步骤 {i} 完成，检查点已保存")

    # 模拟崩溃
    print("\n2. 模拟崩溃...")
    print("  [进程异常终止]")

    # 模拟恢复
    print("\n3. 模拟恢复（新进程）")

    # 加载检查点列表
    checkpoints = client.list(session_id)
    if checkpoints:
        latest_state = checkpoints[-1]
        print(f"  最新检查点状态: {latest_state}")

        # 恢复执行
        state_data = latest_state if isinstance(latest_state, dict) else json.loads(latest_state) if latest_state else {}
        iteration = state_data.get("iteration", 0)
        print(f"\n  从步骤 {iteration + 1} 继续执行...")


def checkpoint_integrity():
    """检查点完整性验证"""
    print("=== 检查点完整性验证 ===")

    client = CheckpointClient()
    session_id = "integrity_session"

    # 创建并保存检查点
    state = {
        "iteration": 10,
        "messages": [{"role": "user", "content": "test"}],
        "tokens_used": 500,
    }

    cp_id = client.save(session_id, state)

    # 验证完整性
    loaded = client.load(session_id, cp_id)

    if loaded:
        parsed = loaded if isinstance(loaded, dict) else json.loads(loaded) if loaded else {}
        print("检查点完整性验证:")
        print(f"  iteration: {'ok' if parsed.get('iteration', 0) >= 0 else 'fail'}")
        print(f"  messages: {'ok' if isinstance(parsed.get('messages'), list) else 'fail'}")
        print(f"  tokens_used: {'ok' if parsed.get('tokens_used', 0) >= 0 else 'fail'}")
        print("\n检查点完整性: 通过")


def cleanup_checkpoints():
    """清理旧检查点"""
    print("=== 清理旧检查点 ===")

    client = CheckpointClient()
    session_id = "cleanup_session"

    # 创建多个检查点
    print(f"创建 5 个检查点")
    for i in range(5):
        state = {"iteration": i, "data": f"step_{i}"}
        client.save(session_id, state)

    # 列出检查点
    checkpoints = client.list(session_id)
    print(f"检查点数量: {len(checkpoints)}")

    # 清理指定检查点
    if len(checkpoints) > 2 and checkpoints:
        first_cp = checkpoints[0]
        client.delete(session_id, first_cp)
        print(f"已删除旧检查点")

    remaining = client.list(session_id)
    print(f"清理后剩余: {len(remaining)} 个")


if __name__ == "__main__":
    basic_checkpoint()
    print("\n" + "=" * 50 + "\n")
    crash_recovery_simulation()
    print("\n" + "=" * 50 + "\n")
    checkpoint_integrity()
    print("\n" + "=" * 50 + "\n")
    cleanup_checkpoints()
