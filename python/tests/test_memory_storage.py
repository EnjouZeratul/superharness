"""Memory Storage 持久化测试"""

import os
import sys
import tempfile
import shutil

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from pathlib import Path

from continuum_sdk.memory import Memory, MemoryEntry, MemoryTier, MemoryStorage, FileStorage


class TestMemoryStorage:
    """MemoryStorage 测试"""

    def test_memory_storage_creation(self):
        """测试内存存储创建"""
        storage = MemoryStorage()
        assert storage is not None

    def test_save_and_load(self):
        """测试保存和加载"""
        storage = MemoryStorage()
        entry = MemoryEntry(
            id="test-1",
            tier=MemoryTier.WORKING,
            content="Test content"
        )

        storage.save(MemoryTier.WORKING, entry)
        loaded = storage.load(MemoryTier.WORKING, "test-1")

        assert loaded is not None
        assert loaded.content == "Test content"

    def test_load_nonexistent(self):
        """测试加载不存在的条目"""
        storage = MemoryStorage()
        loaded = storage.load(MemoryTier.WORKING, "nonexistent")
        assert loaded is None

    def test_load_all(self):
        """测试加载所有条目"""
        storage = MemoryStorage()
        entry1 = MemoryEntry(id="e1", tier=MemoryTier.WORKING, content="content1")
        entry2 = MemoryEntry(id="e2", tier=MemoryTier.WORKING, content="content2")

        storage.save(MemoryTier.WORKING, entry1)
        storage.save(MemoryTier.WORKING, entry2)

        all_entries = storage.load_all(MemoryTier.WORKING)
        assert len(all_entries) == 2

    def test_delete(self):
        """测试删除"""
        storage = MemoryStorage()
        entry = MemoryEntry(id="del-1", tier=MemoryTier.WORKING, content="to delete")

        storage.save(MemoryTier.WORKING, entry)
        assert storage.count(MemoryTier.WORKING) == 1

        result = storage.delete(MemoryTier.WORKING, "del-1")
        assert result
        assert storage.count(MemoryTier.WORKING) == 0

    def test_delete_nonexistent(self):
        """测试删除不存在的条目"""
        storage = MemoryStorage()
        result = storage.delete(MemoryTier.WORKING, "nonexistent")
        assert not result

    def test_clear(self):
        """测试清空"""
        storage = MemoryStorage()
        for i in range(5):
            entry = MemoryEntry(id=f"c{i}", tier=MemoryTier.SESSION, content=f"content{i}")
            storage.save(MemoryTier.SESSION, entry)

        count = storage.clear(MemoryTier.SESSION)
        assert count == 5
        assert storage.count(MemoryTier.SESSION) == 0

    def test_search(self):
        """测试搜索"""
        storage = MemoryStorage()
        entry1 = MemoryEntry(id="s1", tier=MemoryTier.WORKING, content="Python is great")
        entry2 = MemoryEntry(id="s2", tier=MemoryTier.WORKING, content="Java is okay")
        entry3 = MemoryEntry(id="s3", tier=MemoryTier.WORKING, content="Python is popular")

        storage.save(MemoryTier.WORKING, entry1)
        storage.save(MemoryTier.WORKING, entry2)
        storage.save(MemoryTier.WORKING, entry3)

        results = storage.search(MemoryTier.WORKING, "Python", limit=10)
        assert len(results) == 2

    def test_count(self):
        """测试计数"""
        storage = MemoryStorage()
        assert storage.count(MemoryTier.WORKING) == 0

        entry = MemoryEntry(id="cnt1", tier=MemoryTier.WORKING, content="content")
        storage.save(MemoryTier.WORKING, entry)
        assert storage.count(MemoryTier.WORKING) == 1


class TestFileStorage:
    """FileStorage 测试"""

    @pytest.fixture
    def temp_dir(self):
        """创建临时目录"""
        dir_path = tempfile.mkdtemp()
        yield dir_path
        shutil.rmtree(dir_path)

    def test_file_storage_creation(self, temp_dir):
        """测试文件存储创建"""
        storage = FileStorage(temp_dir, session_id="test-session")
        assert storage is not None
        storage.close()

    def test_save_and_load(self, temp_dir):
        """测试保存和加载"""
        storage = FileStorage(temp_dir, session_id="test-save-load")
        entry = MemoryEntry(
            id="fs-1",
            tier=MemoryTier.WORKING,
            content="File storage test"
        )

        storage.save(MemoryTier.WORKING, entry)
        loaded = storage.load(MemoryTier.WORKING, "fs-1")

        assert loaded is not None
        assert loaded.content == "File storage test"
        storage.close()

    def test_persistence(self, temp_dir):
        """测试持久化（关闭后重新加载）"""
        session_id = "persist-test"

        # 创建并保存数据
        storage1 = FileStorage(temp_dir, session_id=session_id)
        entry = MemoryEntry(
            id="persist-1",
            tier=MemoryTier.PROJECT,
            content="Persistent content"
        )
        storage1.save(MemoryTier.PROJECT, entry)
        storage1.close()

        # 重新打开并验证
        storage2 = FileStorage(temp_dir, session_id=session_id)
        loaded = storage2.load(MemoryTier.PROJECT, "persist-1")

        assert loaded is not None
        assert loaded.content == "Persistent content"
        storage2.close()

    def test_file_structure(self, temp_dir):
        """测试文件结构"""
        storage = FileStorage(temp_dir, session_id="structure-test")

        entry = MemoryEntry(id="str-1", tier=MemoryTier.WORKING, content="working")
        storage.save(MemoryTier.WORKING, entry)

        entry = MemoryEntry(id="str-2", tier=MemoryTier.SESSION, content="session")
        storage.save(MemoryTier.SESSION, entry)

        storage.close()

        # 验证文件存在
        assert Path(temp_dir, "structure-test_working.json").exists()
        assert Path(temp_dir, "structure-test_session.json").exists()

    def test_auto_save_disabled(self, temp_dir):
        """测试禁用自动保存"""
        storage = FileStorage(
            temp_dir,
            session_id="no-auto-save",
            auto_save=False
        )

        entry = MemoryEntry(id="no-auto-1", tier=MemoryTier.WORKING, content="no auto")
        storage.save(MemoryTier.WORKING, entry)

        # 手动刷新
        storage.flush()
        storage.close()

        # 验证数据已保存
        storage2 = FileStorage(temp_dir, session_id="no-auto-save")
        loaded = storage2.load(MemoryTier.WORKING, "no-auto-1")
        assert loaded is not None
        storage2.close()

    def test_search(self, temp_dir):
        """测试搜索"""
        storage = FileStorage(temp_dir, session_id="search-test")

        entry1 = MemoryEntry(id="fs-s1", tier=MemoryTier.WORKING, content="Python programming")
        entry2 = MemoryEntry(id="fs-s2", tier=MemoryTier.WORKING, content="Java development")

        storage.save(MemoryTier.WORKING, entry1)
        storage.save(MemoryTier.WORKING, entry2)

        results = storage.search(MemoryTier.WORKING, "Python")
        assert len(results) == 1
        assert results[0].content == "Python programming"

        storage.close()

    def test_delete(self, temp_dir):
        """测试删除"""
        storage = FileStorage(temp_dir, session_id="delete-test")

        entry = MemoryEntry(id="del-test-1", tier=MemoryTier.WORKING, content="to delete")
        storage.save(MemoryTier.WORKING, entry)

        assert storage.count(MemoryTier.WORKING) == 1

        result = storage.delete(MemoryTier.WORKING, "del-test-1")
        assert result
        assert storage.count(MemoryTier.WORKING) == 0

        storage.close()

    def test_clear(self, temp_dir):
        """测试清空"""
        storage = FileStorage(temp_dir, session_id="clear-test")

        for i in range(3):
            entry = MemoryEntry(id=f"clear-{i}", tier=MemoryTier.SESSION, content=f"content {i}")
            storage.save(MemoryTier.SESSION, entry)

        count = storage.clear(MemoryTier.SESSION)
        assert count == 3
        assert storage.count(MemoryTier.SESSION) == 0

        storage.close()


class TestMemoryWithStorage:
    """Memory 与存储后端集成测试"""

    @pytest.fixture
    def temp_dir(self):
        """创建临时目录"""
        dir_path = tempfile.mkdtemp()
        yield dir_path
        shutil.rmtree(dir_path)

    def test_memory_with_memory_storage(self):
        """测试使用内存存储的 Memory"""
        memory = Memory(session_id="memory-test")

        entry_id = memory.remember("Test content", tier=MemoryTier.WORKING)
        assert entry_id is not None

        results = memory.recall("Test")
        assert len(results) == 1
        assert results[0].content == "Test content"

    def test_memory_with_file_storage(self, temp_dir):
        """测试使用文件存储的 Memory"""
        memory = Memory.create_with_file_storage(
            session_id="file-memory-test",
            storage_path=temp_dir
        )

        entry_id = memory.remember("Persistent content", tier=MemoryTier.PROJECT)
        assert entry_id is not None

        # 保存并关闭
        memory.save()

        # 重新加载
        memory2 = Memory.create_with_file_storage(
            session_id="file-memory-test",
            storage_path=temp_dir
        )

        results = memory2.recall("Persistent")
        assert len(results) == 1
        assert results[0].content == "Persistent content"

        memory2.close()

    def test_memory_stats(self):
        """测试统计"""
        memory = Memory(session_id="stats-test")

        memory.remember("W1", tier=MemoryTier.WORKING)
        memory.remember("W2", tier=MemoryTier.WORKING)
        memory.remember("S1", tier=MemoryTier.SESSION)

        stats = memory.stats()
        assert stats[MemoryTier.WORKING] == 2
        assert stats[MemoryTier.SESSION] == 1

    def test_memory_tier_proxy(self):
        """测试层级代理"""
        memory = Memory(session_id="proxy-test")

        memory.working().add("Proxy content")
        results = memory.working().search("Proxy")

        assert len(results) == 1
        assert results[0].content == "Proxy content"

    def test_memory_forget(self):
        """测试删除"""
        memory = Memory(session_id="forget-test")

        entry_id = memory.remember("To be forgotten", tier=MemoryTier.WORKING)
        assert memory.get(MemoryTier.WORKING, entry_id) is not None

        result = memory.forget(MemoryTier.WORKING, entry_id)
        assert result
        assert memory.get(MemoryTier.WORKING, entry_id) is None

    def test_memory_clear(self):
        """测试清空"""
        memory = Memory(session_id="clear-mem-test")

        memory.remember("Item 1", tier=MemoryTier.WORKING)
        memory.remember("Item 2", tier=MemoryTier.WORKING)

        count = memory.clear(MemoryTier.WORKING)
        assert count == 2
        assert len(memory.recall("")) == 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
