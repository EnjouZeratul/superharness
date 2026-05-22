"""Memory 单元测试"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from continuum_sdk.memory import Memory, MemoryTier, MemoryEntry, TierProxy


class TestMemoryTier:
    """MemoryTier 测试"""

    def test_tier_values(self):
        """测试层级值"""
        assert MemoryTier.WORKING.value == "working"
        assert MemoryTier.SESSION.value == "session"
        assert MemoryTier.PROJECT.value == "project"
        assert MemoryTier.LONG_TERM.value == "long_term"


class TestMemoryEntry:
    """MemoryEntry 测试"""

    def test_entry_creation(self):
        """测试条目创建"""
        entry = MemoryEntry(
            id="test-123",
            tier=MemoryTier.WORKING,
            content="Test content"
        )
        assert entry.id == "test-123"
        assert entry.tier == MemoryTier.WORKING
        assert entry.content == "Test content"
        assert entry.importance == 0.5

    def test_entry_touch(self):
        """测试访问更新"""
        entry = MemoryEntry(
            id="test",
            tier=MemoryTier.WORKING,
            content="content"
        )
        initial_count = entry.access_count
        entry.touch()
        assert entry.access_count == initial_count + 1

    def test_entry_with_metadata(self):
        """测试带元数据的条目"""
        entry = MemoryEntry(
            id="meta-test",
            tier=MemoryTier.PROJECT,
            content="Content",
            metadata={"key": "value"},
            importance=0.9
        )
        assert entry.metadata == {"key": "value"}
        assert entry.importance == 0.9


class TestMemory:
    """Memory 测试"""

    def test_memory_creation(self):
        """测试记忆系统创建"""
        memory = Memory(session_id="test-session")
        assert memory.session_id == "test-session"

    def test_remember_working(self):
        """测试工作记忆存储"""
        memory = Memory(session_id="test")
        entry_id = memory.remember("Test fact", tier=MemoryTier.WORKING)
        assert entry_id is not None

    def test_remember_session(self):
        """测试会话记忆存储"""
        memory = Memory(session_id="test")
        entry_id = memory.remember("Session fact", tier=MemoryTier.SESSION)
        assert entry_id is not None

    def test_recall(self):
        """测试记忆查询"""
        memory = Memory(session_id="test")
        memory.remember("Important keyword here", tier=MemoryTier.WORKING)
        results = memory.recall("keyword")
        assert len(results) > 0
        assert "keyword" in results[0].content.lower()

    def test_recall_empty(self):
        """测试空查询"""
        memory = Memory(session_id="test")
        results = memory.recall("nonexistent")
        assert len(results) == 0

    def test_recall_with_tier_limit(self):
        """测试限定层级的查询"""
        memory = Memory(session_id="test")
        memory.remember("Working info", tier=MemoryTier.WORKING)
        memory.remember("Project info", tier=MemoryTier.PROJECT)
        
        results = memory.recall("info", tier=MemoryTier.WORKING)
        assert len(results) == 1

    def test_get(self):
        """测试获取特定记忆"""
        memory = Memory(session_id="test")
        entry_id = memory.remember("Specific content", tier=MemoryTier.WORKING)
        entry = memory.get(MemoryTier.WORKING, entry_id)
        assert entry is not None
        assert entry.content == "Specific content"

    def test_get_nonexistent(self):
        """测试获取不存在的记忆"""
        memory = Memory(session_id="test")
        entry = memory.get(MemoryTier.WORKING, "nonexistent")
        assert entry is None

    def test_forget(self):
        """测试删除记忆"""
        memory = Memory(session_id="test")
        entry_id = memory.remember("To be deleted", tier=MemoryTier.WORKING)
        result = memory.forget(MemoryTier.WORKING, entry_id)
        assert result == True
        
        # 验证已删除
        entry = memory.get(MemoryTier.WORKING, entry_id)
        assert entry is None

    def test_forget_nonexistent(self):
        """测试删除不存在的记忆"""
        memory = Memory(session_id="test")
        result = memory.forget(MemoryTier.WORKING, "nonexistent")
        assert result == False

    def test_clear(self):
        """测试清空层级"""
        memory = Memory(session_id="test")
        memory.remember("Item 1", tier=MemoryTier.WORKING)
        memory.remember("Item 2", tier=MemoryTier.WORKING)
        
        count = memory.clear(MemoryTier.WORKING)
        assert count == 2
        assert len(memory.recall("")) == 0

    def test_stats(self):
        """测试统计"""
        memory = Memory(session_id="test")
        memory.remember("W1", tier=MemoryTier.WORKING)
        memory.remember("S1", tier=MemoryTier.SESSION)
        memory.remember("S2", tier=MemoryTier.SESSION)
        
        stats = memory.stats()
        assert stats[MemoryTier.WORKING] == 1
        assert stats[MemoryTier.SESSION] == 2

    def test_working_proxy(self):
        """测试工作记忆代理"""
        memory = Memory(session_id="test")
        proxy = memory.working()
        assert isinstance(proxy, TierProxy)

    def test_proxy_add(self):
        """测试代理添加"""
        memory = Memory(session_id="test")
        entry_id = memory.working().add("Proxy content")
        assert entry_id is not None

    def test_proxy_search(self):
        """测试代理搜索"""
        memory = Memory(session_id="test")
        memory.working().add("Find me")
        results = memory.working().search("Find")
        assert len(results) > 0

    def test_proxy_count(self):
        """测试代理计数"""
        memory = Memory(session_id="test")
        memory.working().add("Item 1")
        memory.working().add("Item 2")
        assert memory.working().count() == 2

    def test_to_dict(self):
        """测试导出"""
        memory = Memory(session_id="export-test")
        data = memory.to_dict()
        assert data["session_id"] == "export-test"
        assert "stats" in data

    def test_from_dict(self):
        """测试导入"""
        data = {"session_id": "imported"}
        memory = Memory.from_dict(data)
        assert memory.session_id == "imported"


class TestWorkingMemoryLimit:
    """工作记忆限制测试"""

    def test_working_memory_limit(self):
        """测试工作记忆大小限制"""
        memory = Memory(session_id="limit-test")
        
        # 添加超过限制的数量
        for i in range(150):
            memory.remember(f"Item {i}", tier=MemoryTier.WORKING)
        
        stats = memory.stats()
        # 工作记忆应被限制在100条
        assert stats[MemoryTier.WORKING] <= 100


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
