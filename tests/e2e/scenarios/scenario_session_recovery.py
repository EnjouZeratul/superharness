"""E2E 场景: 会话恢复流程

测试会话的保存、恢复、回滚能力。

场景描述:
1. 用户开始会话并进行对话
2. 会话意外中断
3. 用户恢复会话
4. Agent 应能继续之前的对话
5. 用户回滚到检查点
6. 会话状态应恢复到检查点时刻

预期行为:
- 会话正确保存
- 恢复后历史完整
- 回滚正确执行
"""

import pytest
from unittest.mock import Mock, patch
import asyncio
import json
from pathlib import Path

from continuum_sdk.agent.session import Session, Message, MessageRole
from continuum.checkpoint_writer import (
    CheckpointWriter,
    CrashRecovery,
    CheckpointData,
    ChecksumUtils,
)


class ScenarioSessionRecovery:
    """会话恢复场景"""

    # 场景配置
    SCENARIO_FLOW = {
        "initial_messages": [
            "你好，我是用户",
            "帮我记住：我喜欢 Python",
            "我正在学习 async/await",
        ],
        "checkpoint_name": "before-break",
        "additional_messages": [
            "我正在学习装饰器",
        ],
        "rollback_test": {
            "prompt": "我正在学什么？",
            "expected_before_rollback": "async/await, 装饰器",
            "expected_after_rollback": "async/await",
        },
    }

    async def run_initial(self, agent):
        """运行初始对话"""
        history = []
        for msg in self.SCENARIO_FLOW["initial_messages"]:
            response = await agent.chat(msg)
            history.append({"user": msg, "assistant": response})
        return history

    async def save_checkpoint(self, session):
        """保存检查点"""
        checkpoint_id = await session.save_checkpoint(
            name=self.SCENARIO_FLOW["checkpoint_name"]
        )
        return checkpoint_id

    async def run_after_checkpoint(self, agent):
        """检查点后对话"""
        history = []
        for msg in self.SCENARIO_FLOW["additional_messages"]:
            response = await agent.chat(msg)
            history.append({"user": msg, "assistant": response})
        return history

    async def test_rollback(self, agent, session, checkpoint_id):
        """测试回滚"""
        # 回滚前
        response_before = await agent.chat(
            self.SCENARIO_FLOW["rollback_test"]["prompt"]
        )

        # 回滚
        await session.rollback(checkpoint_id)

        # 回滚后
        response_after = await agent.chat(
            self.SCENARIO_FLOW["rollback_test"]["prompt"]
        )

        return {
            "before": response_before,
            "after": response_after,
        }


class TestScenarioSessionRecovery:
    """会话恢复场景测试"""

    @pytest.mark.e2e
    async def test_session_save_to_file(self, tmp_path):
        """测试会话保存到文件"""
        # 创建会话并添加消息
        session = Session(id="test-save-session")
        session.add_user_message("你好")
        session.add_assistant_message("你好！有什么可以帮你的？")
        session.add_user_message("请记住我喜欢 Python")
        session.add_assistant_message("好的，我记住了你喜欢 Python")

        # 保存到临时文件
        save_path = tmp_path / "test_session.json"
        session.save(save_path)

        # 验证文件已创建
        assert save_path.exists(), f"会话文件应该已创建: {save_path}"

        # 验证文件内容有效
        with open(save_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        assert data["id"] == "test-save-session"
        assert len(data["messages"]) == 4
        assert "Python" in data["messages"][2]["content"]

    @pytest.mark.e2e
    async def test_session_load_from_file(self, tmp_path):
        """测试从文件恢复会话"""
        # 创建并保存会话
        original_session = Session(id="test-load-session")
        original_session.add_user_message("测试问题")
        original_session.add_assistant_message("测试回答")
        original_session.set_metadata("custom_key", "custom_value")

        save_path = tmp_path / "loadable_session.json"
        original_session.save(save_path)

        # 从文件加载
        loaded_session = Session.load(save_path)

        # 验证会话状态恢复
        assert loaded_session.id == "test-load-session"
        assert loaded_session.message_count == 2
        assert loaded_session.get_metadata("custom_key") == "custom_value"

        # 验证消息内容恢复
        messages = loaded_session.get_messages()
        assert messages[0].content == "测试问题"
        assert messages[1].content == "测试回答"

    @pytest.mark.e2e
    async def test_checkpoint_create_with_writer(self, tmp_path):
        """测试检查点创建"""
        writer = CheckpointWriter(storage_path=str(tmp_path))

        # 创建检查点数据
        checkpoint_data = {
            "session_id": "test-checkpoint",
            "messages": [
                {"role": "user", "content": "你好"},
                {"role": "assistant", "content": "你好！"},
            ],
            "iteration": 1,
        }

        # 保存检查点
        success, error = writer.save_checkpoint(checkpoint_data, "test-checkpoint")

        assert success, f"检查点保存应该成功: {error}"

        # 验证检查点文件存在
        checkpoint_dir = tmp_path / "test-checkpoint" / "checkpoints"
        assert checkpoint_dir.exists()

    @pytest.mark.e2e
    async def test_checkpoint_load_and_verify(self, tmp_path):
        """测试检查点加载并验证"""
        writer = CheckpointWriter(storage_path=str(tmp_path))

        # 创建并保存检查点
        original_data = {
            "session_id": "test-verify",
            "messages": [
                {"role": "user", "content": "验证测试"},
                {"role": "assistant", "content": "验证通过"},
            ],
            "iteration": 2,
        }

        success, _ = writer.save_checkpoint(original_data, "test-verify")
        assert success

        # 加载检查点
        loaded_data, error = writer.load_checkpoint("test-verify")

        assert loaded_data is not None, f"检查点加载失败: {error}"
        assert loaded_data["session_id"] == "test-verify"
        assert len(loaded_data["messages"]) == 2
        assert loaded_data["iteration"] == 2

    @pytest.mark.e2e
    async def test_session_persistence_across_instances(self, tmp_path):
        """测试会话跨实例持久化"""
        # 第一个实例创建并保存会话
        session1 = Session(id="persistence-test")
        session1.add_user_message("第一个问题")
        session1.add_assistant_message("第一个回答")
        session1.set_metadata("user_id", "user123")
        session1.record_tool_use("read_file")

        save_path = tmp_path / "persistent_session.json"
        session1.save(save_path)

        # 模拟进程重启：创建新实例加载会话
        session2 = Session.load(save_path)

        # 验证状态完整恢复
        assert session2.id == "persistence-test"
        assert session2.message_count == 2
        assert session2.get_metadata("user_id") == "user123"
        assert session2.get_tools_used() == ["read_file"]

        # 继续对话
        session2.add_user_message("第二个问题")
        assert session2.message_count == 3

    @pytest.mark.e2e
    async def test_multiple_checkpoints_management(self, tmp_path):
        """测试多检查点管理"""
        writer = CheckpointWriter(storage_path=str(tmp_path))

        session_id = "multi-checkpoint-test"

        # 创建多个检查点
        for i in range(3):
            checkpoint_data = {
                "session_id": session_id,
                "messages": [{"role": "user", "content": f"检查点 {i+1}"}],
                "iteration": i + 1,
            }
            success, error = writer.save_checkpoint(checkpoint_data, session_id)
            assert success, f"检查点 {i+1} 保存失败: {error}"

        # 列出所有检查点
        checkpoints = writer.list_checkpoints(session_id)
        assert len(checkpoints) >= 3, "应该至少有 3 个检查点"

        # 验证检查点按时间排序
        timestamps = [cp[1] for cp in checkpoints]
        assert timestamps == sorted(timestamps, reverse=True), "检查点应按时间降序排列"

    @pytest.mark.e2e
    async def test_session_recovery_with_tools_recorded(self, tmp_path):
        """测试工具调用记录在恢复时保留"""
        session = Session(id="tool-recovery-test")

        # 模拟工具调用对话
        session.add_user_message("读取 main.py")
        session.record_tool_use("read_file")
        session.add_assistant_message("文件内容是...")
        session.record_tool_use("bash")
        session.add_user_message("运行测试")
        session.add_assistant_message("测试通过")

        # 保存并恢复
        save_path = tmp_path / "tool_session.json"
        session.save(save_path)
        restored = Session.load(save_path)

        # 验证工具调用记录保留
        tools_used = restored.get_tools_used()
        assert len(tools_used) == 2
        assert "read_file" in tools_used
        assert "bash" in tools_used


# ==================== 运行标记 ====================

pytestmark = pytest.mark.e2e
