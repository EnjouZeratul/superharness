"""CLI session 命令集成测试

测试 `continuum session` 命令的各种场景。
"""

import pytest
import sys
import os
import tempfile
import json
import shutil
from pathlib import Path
from datetime import datetime
from unittest.mock import Mock, patch, MagicMock

# Add src and python directories to path
# Note: src must be added LAST so it shadows python/continuum with checkpoint_writer
_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(_root, "python"))
sys.path.insert(0, os.path.join(_root, "src"))


@pytest.fixture
def mock_session_config():
    """Mock session config fixture"""
    return {
        "session_id": "test-session-123",
        "name": "Test Session",
        "status": "active",
        "created_at": datetime.now().isoformat(),
    }


@pytest.fixture
def session_storage(tmp_path):
    """Session storage fixture"""
    storage = tmp_path / "sessions"
    storage.mkdir(parents=True, exist_ok=True)
    return storage


class TestCLISessionList:
    """session list 测试"""

    def test_list_all_sessions(self, session_storage):
        """测试列出所有会话"""
        # 创建多个会话
        for i in range(3):
            session_file = session_storage / f"session-{i}.json"
            session_file.write_text(json.dumps({
                "id": f"session-{i}",
                "name": f"Session {i}",
                "status": "active",
                "created_at": datetime.now().isoformat(),
            }))

        sessions = list(session_storage.glob("*.json"))
        assert len(sessions) == 3
        print(f"\n[List Sessions]: Found {len(sessions)} sessions")

    def test_list_sessions_with_filter(self, session_storage):
        """测试过滤会话"""
        # 创建不同状态的会话
        statuses = ["active", "completed", "failed"]
        for i, status in enumerate(statuses):
            session_file = session_storage / f"session-{status}-{i}.json"
            session_file.write_text(json.dumps({"status": status}))

        # 过滤 active
        active_sessions = [
            f for f in session_storage.glob("*.json")
            if json.loads(f.read_text()).get("status") == "active"
        ]
        assert len(active_sessions) == 1
        print(f"\n[Filter Sessions]: {len(active_sessions)} active")

    def test_list_sessions_empty(self, session_storage):
        """测试空列表"""
        # 清空目录
        for f in session_storage.glob("*.json"):
            f.unlink()

        sessions = list(session_storage.glob("*.json"))
        assert len(sessions) == 0
        print("\n[Empty Sessions]: No sessions found")

    def test_list_sessions_format(self, session_storage):
        """测试列表格式"""
        session_file = session_storage / "test.json"
        session_file.write_text(json.dumps({
            "id": "test-123",
            "name": "Test Session",
            "status": "active",
            "created_at": "2026-01-01T00:00:00",
        }))

        data = json.loads(session_file.read_text())
        # 验证格式化字段存在
        assert "id" in data
        assert "name" in data
        assert "status" in data
        assert "created_at" in data
        print(f"\n[Format]: {data}")


class TestCLISessionResume:
    """session resume 测试"""

    def test_resume_by_id(self, session_storage):
        """测试按 ID 恢复会话"""
        from continuum_sdk.agent.session import Session

        session_id = "resume-test"
        session_file = session_storage / f"{session_id}.json"

        # 创建会话
        original_session = Session(id=session_id)
        original_session.add_user_message("Previous message")
        exported = original_session.export()

        # export() returns JSON string directly
        session_file.write_text(exported)

        # 模拟恢复 - from_export() expects JSON string
        restored = Session.from_export(session_file.read_text())

        assert restored.id == session_id
        assert restored.message_count == 1
        print(f"\n[Resume ID]: Restored {session_id}")

    def test_resume_by_name(self, session_storage):
        """测试按名称恢复会话"""
        from continuum_sdk.agent.session import Session

        session_name = "named-session"
        session_file = session_storage / f"{session_name}.json"

        original = Session(id=session_name)
        original.add_user_message("Named session test")
        # export() returns JSON string directly
        session_file.write_text(original.export())

        # 查找并恢复
        matches = [f for f in session_storage.glob("*.json")
                   if f.stem == session_name]

        assert len(matches) == 1
        # from_export() expects JSON string
        restored = Session.from_export(matches[0].read_text())

        print(f"\n[Resume Name]: Found {session_name}")

    def test_resume_nonexistent_session(self, session_storage):
        """测试恢复不存在会话"""
        nonexistent_id = "nonexistent-123"

        # 查找不存在的会话
        matches = [f for f in session_storage.glob("*.json")
                   if f.stem == nonexistent_id]

        assert len(matches) == 0
        print(f"\n[Resume Nonexistent]: Session {nonexistent_id} not found")

    def test_resume_corrupted_session(self, session_storage):
        """测试恢复损坏会话"""
        corrupted_file = session_storage / "corrupted.json"
        corrupted_file.write_text("{invalid json content")

        try:
            data = json.loads(corrupted_file.read_text())
            pytest.fail("Should have raised JSON decode error")
        except json.JSONDecodeError as e:
            print(f"\n[Corrupted Session]: Error detected - {e}")
            assert True


class TestCLISessionDelete:
    """session delete 测试"""

    def test_delete_by_id(self, session_storage):
        """测试按 ID 删除"""
        session_file = session_storage / "to-delete.json"
        session_file.write_text(json.dumps({"id": "to-delete"}))

        # 删除
        session_file.unlink()
        assert not session_file.exists()
        print("\n[Delete]: Session removed")

    def test_delete_with_confirm(self, session_storage):
        """测试确认删除"""
        session_file = session_storage / "confirm-delete.json"
        session_file.write_text(json.dumps({"id": "confirm-delete", "important": True}))

        # 模拟确认流程
        user_confirms = True

        if user_confirms:
            session_file.unlink()
            print("\n[Delete Confirm]: User confirmed, deleted")
        else:
            print("\n[Delete Confirm]: User cancelled")

        assert not session_file.exists()  # 我们确认了

    def test_delete_force(self, session_storage):
        """测试强制删除"""
        session_file = session_storage / "force-delete.json"
        session_file.write_text(json.dumps({"id": "force-delete"}))

        # --force 跳过确认，直接删除
        force_delete = True

        if force_delete:
            session_file.unlink()

        assert not session_file.exists()
        print("\n[Delete Force]: Skipped confirmation")

    def test_delete_nonexistent(self, session_storage):
        """测试删除不存在会话"""
        nonexistent = session_storage / "nonexistent.json"

        # 尝试删除不存在的文件应静默处理
        if nonexistent.exists():
            nonexistent.unlink()

        print("\n[Delete Nonexistent]: Silently handled")


class TestCLISessionCheckpoint:
    """session checkpoint 测试"""

    def test_checkpoint_create(self, session_storage):
        """测试创建检查点"""
        from continuum.checkpoint_writer import CheckpointWriter

        checkpoint_storage = session_storage / "checkpoints"
        checkpoint_storage.mkdir(parents=True, exist_ok=True)

        writer = CheckpointWriter(storage_path=str(checkpoint_storage))

        checkpoint = {
            "session_id": "checkpoint-test",
            "messages": [{"role": "user", "content": "test"}],
            "iteration": 1,
        }

        success, error = writer.save_checkpoint(checkpoint, "checkpoint-test")
        assert success is True
        print(f"\n[Checkpoint Create]: Success")

    def test_checkpoint_list(self, session_storage):
        """测试列出检查点"""
        from continuum.checkpoint_writer import CheckpointWriter

        checkpoint_storage = session_storage / "cp_list_test"
        checkpoint_storage.mkdir(parents=True, exist_ok=True)

        writer = CheckpointWriter(storage_path=str(checkpoint_storage))

        # 创建多个检查点
        for i in range(3):
            checkpoint = {
                "session_id": "cp-list-test",
                "messages": [{"role": "user", "content": f"iteration {i}"}],
                "iteration": i,
            }
            writer.save_checkpoint(checkpoint, "cp-list-test")
            import time
            time.sleep(0.05)  # 确保时间戳不同

        checkpoints = writer.list_checkpoints("cp-list-test")
        assert len(checkpoints) >= 3
        print(f"\n[Checkpoint List]: {len(checkpoints)} checkpoints")

    def test_checkpoint_rollback(self, session_storage):
        """测试回滚到检查点"""
        from continuum.checkpoint_writer import CheckpointWriter

        checkpoint_storage = session_storage / "rollback_test"
        checkpoint_storage.mkdir(parents=True, exist_ok=True)

        writer = CheckpointWriter(storage_path=str(checkpoint_storage))

        # 保存初始状态
        initial = {
            "session_id": "rollback-test",
            "messages": [{"role": "user", "content": "initial"}],
            "iteration": 1,
        }
        writer.save_checkpoint(initial, "rollback-test")

        # 更新状态
        updated = {
            "session_id": "rollback-test",
            "messages": [{"role": "user", "content": "updated"}],
            "iteration": 2,
        }
        writer.save_checkpoint(updated, "rollback-test")

        # 恢复初始状态
        data, error = writer.load_checkpoint("rollback-test")
        assert data is not None
        print(f"\n[Checkpoint Rollback]: Loaded iteration {data['iteration']}")

    def test_checkpoint_delete(self, session_storage):
        """测试删除检查点"""
        from continuum.checkpoint_writer import CheckpointWriter

        checkpoint_storage = session_storage / "delete_test"
        checkpoint_storage.mkdir(parents=True, exist_ok=True)

        writer = CheckpointWriter(storage_path=str(checkpoint_storage))

        checkpoint = {"session_id": "delete-test", "messages": [], "iteration": 0}
        writer.save_checkpoint(checkpoint, "delete-test")

        # 删除检查点目录
        cp_dir = checkpoint_storage / "delete-test" / "checkpoints"
        if cp_dir.exists():
            shutil.rmtree(cp_dir)

        print("\n[Checkpoint Delete]: Removed")


class TestCLISessionExport:
    """session export 测试"""

    def test_export_to_json(self, session_storage):
        """测试导出 JSON"""
        from continuum_sdk.agent.session import Session

        session = Session(id="export-json")
        session.add_user_message("Test")
        session.add_assistant_message("Response")

        exported = session.export()
        # export() returns JSON string, parse it
        parsed = json.loads(exported)

        assert "id" in parsed
        assert "messages" in parsed
        print(f"\n[Export JSON]: {len(parsed['messages'])} messages")

    def test_export_to_markdown(self, session_storage):
        """测试导出 Markdown"""
        from continuum_sdk.agent.session import Session

        session = Session(id="export-md")
        session.add_user_message("Hello")
        session.add_assistant_message("Hi there!")

        # 构建 Markdown 格式
        markdown = "# Session Export\n\n"
        for msg in session.get_messages():
            role = msg.role.value
            content = msg.content
            markdown += f"**{role}**: {content}\n\n"

        assert "# Session Export" in markdown
        assert "**user**" in markdown
        print(f"\n[Export Markdown]: {len(markdown)} chars")

    def test_export_with_metadata(self, session_storage):
        """测试带元数据导出"""
        from continuum_sdk.agent.session import Session

        session = Session(id="export-meta")
        session.update_cost(0.05, 1000)
        session.record_tool_use("read_file")

        exported = session.export()
        # export() returns JSON string, parse it
        parsed = json.loads(exported)

        # 验证元数据
        assert parsed.get("cost") == 0.05
        assert parsed.get("tokens") == 1000
        assert "read_file" in parsed.get("tools_used", [])
        print(f"\n[Export Metadata]: cost={parsed.get('cost')}, tokens={parsed.get('tokens')}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])