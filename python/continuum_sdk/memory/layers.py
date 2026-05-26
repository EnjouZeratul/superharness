"""分层记忆 API

提供 Working -> Session -> Project -> LongTerm 四层记忆。

[STABILITY: STABLE] Core API 稳定
支持多种存储后端：MemoryStorage、FileStorage、SQLiteStorage
"""

import uuid
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any

from .storage import FileStorage, MemoryStorage, StorageBackend, MemoryEntry, MemoryTier


@dataclass
class MemoryQuery:
    """记忆查询"""

    query: str
    tier: MemoryTier | None = None
    limit: int = 10
    time_range: tuple | None = None


class TierProxy:
    """层级代理

    Usage:
        memory.working().add("内容")
        memory.working().search("查询")
        memory.working().clear()
    """

    def __init__(self, memory: "Memory", tier: MemoryTier):
        self._memory = memory
        self._tier = tier

    def add(
        self,
        content: str,
        metadata: dict[str, Any] | None = None,
        importance: float = 0.5,
    ) -> str:
        """添加记忆"""
        return self._memory.remember(
            content, tier=self._tier, metadata=metadata, importance=importance
        )

    def search(self, query: str, limit: int = 10) -> list[MemoryEntry]:
        """搜索记忆"""
        return self._memory.recall(query, tier=self._tier, limit=limit)

    def get(self, memory_id: str) -> MemoryEntry | None:
        """获取记忆"""
        return self._memory.get(self._tier, memory_id)

    def remove(self, memory_id: str) -> bool:
        """删除记忆"""
        return self._memory.forget(self._tier, memory_id)

    def clear(self) -> int:
        """清空层级"""
        return self._memory.clear(self._tier)

    def count(self) -> int:
        """获取数量"""
        stats = self._memory.stats()
        return stats.get(self._tier, 0)


class Memory:
    """分层记忆系统

    Usage:
        from continuum_sdk.memory import Memory, MemoryTier

        # 创建内存存储的记忆系统（默认）
        memory = Memory(session_id="session-123")

        # 创建文件持久化的记忆系统
        memory = Memory(
            session_id="session-123",
            storage=FileStorage("~/.continuum/memory", session_id="session-123")
        )

        # 存储记忆
        memory.remember("Important fact", tier=MemoryTier.WORKING)
        memory.remember("Project config", tier=MemoryTier.PROJECT)

        # 查询记忆
        results = memory.query("fact")

        # 获取特定层级
        working = memory.get_tier(MemoryTier.WORKING)

        # 统计
        stats = memory.stats()

        # 便捷访问
        memory.working().add("临时信息")
        results = memory.working().search("关键词")

        # 持久化操作
        memory.save()  # 保存到文件
        memory.close()  # 关闭并保存
    """

    def __init__(
        self,
        session_id: str,
        storage: StorageBackend | None = None,
        auto_persist: bool = False,
    ):
        """初始化记忆系统

        Args:
            session_id: 会话 ID
            storage: 存储后端（None 表示使用内存存储）
            auto_persist: 是否自动持久化（仅对 FileStorage 有效）
        """
        self._session_id = session_id
        self._auto_persist = auto_persist

        # 使用提供的存储后端，或默认使用内存存储
        if storage is not None:
            self._backend = storage
        else:
            self._backend = MemoryStorage()

        # 工作记忆大小限制
        self._working_limit = 100

    @property
    def session_id(self) -> str:
        """获取会话 ID"""
        return self._session_id

    def remember(
        self,
        content: str,
        tier: MemoryTier = MemoryTier.WORKING,
        metadata: dict[str, Any] | None = None,
        importance: float = 0.5,
    ) -> str:
        """存储记忆

        Args:
            content: 记忆内容
            tier: 记忆层级
            metadata: 元数据（可选）
            importance: 重要性分数 (0.0-1.0)

        Returns:
            记忆 ID
        """
        entry = MemoryEntry(
            id=str(uuid.uuid4())[:8],
            tier=tier,
            content=content,
            metadata=metadata or {},
            importance=importance,
        )

        # 使用存储后端保存
        self._backend.save(tier, entry)

        # 工作记忆限制大小
        if tier == MemoryTier.WORKING:
            count = self._backend.count(tier)
            if count > self._working_limit:
                entries = self._backend.load_all(tier)
                if entries:
                    oldest = min(entries, key=lambda e: e.created_at)
                    self._backend.delete(tier, oldest.id)

        return entry.id

    def recall(
        self, query: str, tier: MemoryTier | None = None, limit: int = 10
    ) -> list[MemoryEntry]:
        """查询记忆

        Args:
            query: 查询文本
            tier: 限制层级（可选）
            limit: 结果数量限制

        Returns:
            匹配的记忆条目列表
        """
        results = []

        # 搜索顺序：Working -> Session -> Project -> LongTerm
        tiers = (
            [tier]
            if tier
            else [
                MemoryTier.WORKING,
                MemoryTier.SESSION,
                MemoryTier.PROJECT,
                MemoryTier.LONG_TERM,
            ]
        )

        for t in tiers:
            entries = self._backend.search(t, query, limit - len(results))
            results.extend(entries)
            if len(results) >= limit:
                break

        return results

    def get(self, tier: MemoryTier, memory_id: str) -> MemoryEntry | None:
        """获取特定记忆

        Args:
            tier: 记忆层级
            memory_id: 记忆 ID

        Returns:
            记忆条目（如果存在）
        """
        return self._backend.load(tier, memory_id)

    def forget(self, tier: MemoryTier, memory_id: str) -> bool:
        """删除记忆

        Args:
            tier: 记忆层级
            memory_id: 记忆 ID

        Returns:
            是否成功删除
        """
        return self._backend.delete(tier, memory_id)

    def clear(self, tier: MemoryTier) -> int:
        """清空指定层级

        Args:
            tier: 记忆层级

        Returns:
            删除的记忆数量
        """
        return self._backend.clear(tier)

    def stats(self) -> dict[MemoryTier, int]:
        """获取各层级统计

        Returns:
            各层级记忆数量
        """
        return {tier: self._backend.count(tier) for tier in MemoryTier}

    # ==================== 持久化方法 ====================

    def save(self) -> None:
        """保存所有记忆到存储"""
        self._backend.flush()

    def close(self) -> None:
        """关闭记忆系统，保存所有数据"""
        self._backend.close()

    @staticmethod
    def get_default_storage_path() -> Path:
        """获取默认存储路径"""
        return FileStorage.get_default_storage_path()

    @classmethod
    def create_with_file_storage(
        cls,
        session_id: str,
        storage_path: str | Path | None = None,
        auto_persist: bool = True,
    ) -> "Memory":
        """创建使用文件存储的记忆系统

        Args:
            session_id: 会话 ID
            storage_path: 存储路径（默认 ~/.continuum/memory）
            auto_persist: 是否自动持久化

        Returns:
            Memory 实例
        """
        path = Path(storage_path) if storage_path else cls.get_default_storage_path()
        storage = FileStorage(path, auto_save=auto_persist, session_id=session_id)
        return cls(session_id=session_id, storage=storage, auto_persist=auto_persist)

    # ==================== 便捷方法 ====================

    def working(self) -> TierProxy:
        """获取工作记忆代理"""
        return TierProxy(self, MemoryTier.WORKING)

    def session(self) -> TierProxy:
        """获取会话记忆代理"""
        return TierProxy(self, MemoryTier.SESSION)

    def project(self) -> TierProxy:
        """获取项目记忆代理"""
        return TierProxy(self, MemoryTier.PROJECT)

    def long_term(self) -> TierProxy:
        """获取长期记忆代理"""
        return TierProxy(self, MemoryTier.LONG_TERM)

    # ==================== 序列化 ====================

    def to_dict(self) -> dict[str, Any]:
        """导出为字典"""
        return {
            "session_id": self._session_id,
            "stats": {tier.value: count for tier, count in self.stats().items()},
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "Memory":
        """从字典创建"""
        return cls(session_id=data.get("session_id", ""))
