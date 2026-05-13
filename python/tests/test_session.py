"""Session 单元测试"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from datetime import datetime
from superharness_sdk.agent.session import Session, Message, MessageRole


class TestMessage:
    """Message 测试"""

    def test_message_creation(self):
        """测试消息创建"""
        msg = Message(role=MessageRole.USER, content="Hello")
        assert msg.role == MessageRole.USER
        assert msg.content == "Hello"
        assert msg.timestamp is not None

    def test_message_to_dict(self):
        """测试消息序列化"""
        msg = Message(role=MessageRole.USER, content="Test")
        data = msg.to_dict()
        assert data["role"] == "user"
        assert data["content"] == "Test"

    def test_message_from_dict(self):
        """测试消息反序列化"""
        data = {"role": "assistant", "content": "Response", "timestamp": datetime.now().isoformat()}
        msg = Message.from_dict(data)
        assert msg.role == MessageRole.ASSISTANT
        assert msg.content == "Response"


class TestSession:
    """Session 测试"""

    def test_session_creation(self):
        """测试会话创建"""
        session = Session()
        assert session.id is not None
        assert session.message_count == 0

    def test_session_with_id(self):
        """测试指定 ID 的会话"""
        session = Session(id="custom-id")
        assert session.id == "custom-id"

    def test_add_user_message(self):
        """测试添加用户消息"""
        session = Session()
        session.add_user_message("Hello")
        assert session.message_count == 1

    def test_add_assistant_message(self):
        """测试添加助手消息"""
        session = Session()
        session.add_assistant_message("Hi there")
        assert session.message_count == 1

    def test_add_system_message(self):
        """测试添加系统消息"""
        session = Session()
        session.add_system_message("System prompt")
        assert session.message_count == 1

    def test_get_messages(self):
        """测试获取消息列表"""
        session = Session()
        session.add_user_message("Q1")
        session.add_assistant_message("A1")
        messages = session.get_messages()
        assert len(messages) == 2

    def test_clear_messages(self):
        """测试清空消息"""
        session = Session()
        session.add_user_message("Test")
        session.clear_messages()
        assert session.message_count == 0

    def test_get_last_message(self):
        """测试获取最后一条消息"""
        session = Session()
        session.add_user_message("First")
        session.add_user_message("Last")
        last = session.get_last_message()
        assert last.content == "Last"

    def test_get_last_message_empty(self):
        """测试空会话获取最后消息"""
        session = Session()
        assert session.get_last_message() is None

    def test_metadata(self):
        """测试元数据"""
        session = Session()
        session.set_metadata("key", "value")
        assert session.get_metadata("key") == "value"
        assert session.get_metadata("missing") is None

    def test_tool_recording(self):
        """测试工具使用记录"""
        session = Session()
        session.record_tool_use("read_file")
        session.record_tool_use("write_file")
        tools = session.get_tools_used()
        assert "read_file" in tools
        assert "write_file" in tools

    def test_cost_tracking(self):
        """测试成本追踪"""
        session = Session()
        session.update_cost(0.05, 1000)
        assert session.cost == 0.05
        assert session.tokens == 1000
        session.update_cost(0.03, 500)
        assert session.cost == 0.08
        assert session.tokens == 1500

    def test_export_import(self):
        """测试导出导入"""
        session = Session(id="export-test")
        session.add_user_message("Test message")
        exported = session.export()
        
        restored = Session.from_export(exported)
        assert restored.id == "export-test"
        assert restored.message_count == 1

    def test_created_at(self):
        """测试创建时间"""
        session = Session()
        assert isinstance(session.created_at, datetime)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
