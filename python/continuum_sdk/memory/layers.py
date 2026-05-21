"""分层记忆 API

提供 Working -> Session -> Project -> LongTerm 四层记忆。
"""

from typing import Any, Dict, List, Optional
from datetime import datetime
from enum import Enum
from dataclasses import dataclass, field
import uuid


class MemoryTier(Enum):
    """记忆层级"""
    WORKING = "working"      # 当前对话上下文
    SESSION = "session"     # 会话记忆
    PROJECT = "project"     # 项目知识库
    LONG_TERM = "long_term" # 跨项目知识


@dataclass
class MemoryEntry:
    """记忆条目"""
    id: str
    tier: MemoryTier
    content: str
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.utcnow)
    last_accessed: datetime = field(default_factory=datetime.utcnow)
    access_count: int = 0
    importance: float = 0.5

    def touch(self) -> None:
        """更新访问时间和计数"""
        self.last_accessed = datetime.utcnow()
        self.access_count += 1


@dataclass
class MemoryQuery:
    """记忆查询"""
    query: str
    tier: Optional[MemoryTier] = None
    limit: int = 10
    time_range: Optional[tuple] = None


class TierProxy:
    """层级代理

    Usage:
        memory.working().add("内容")
        memory.working().search("查询")
        memory.working().clear()
    """

    def __init__(self, memory: 'Memory', tier: MemoryTier):
        self._memory = memory
        self._tier = tier

    def add(self, content: str, metadata: Optional[Dict[str, Any]] = None, importance: float = 0.5) -> str:
        """添加记忆"""
        return self._memory.remember(content, tier=self._tier, metadata=metadata, importance=importance)

    def search(self, query: str, limit: int = 10) -> List[MemoryEntry]:
        """搜索记忆"""
        return self._memory.recall(query, tier=self._tier, limit=limit)

    def get(self, memory_id: str) -> Optional[MemoryEntry]:
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

        memory = Memory(session_id="session-123")

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
    """

    def __init__(self, session_id: str):
        """初始化记忆系统

        Args:
            session_id: 会话 ID
        """
        self._session_id = session_id

        # 各层级存储（占位实现，实际应调用 sh-core）
        self._working: List[MemoryEntry] = []
        self._session: List[MemoryEntry] = []
        self._project: List[MemoryEntry] = []
        self._long_term: List[MemoryEntry] = []

        # 层级映射
        self._storage = {
            MemoryTier.WORKING: self._working,
            MemoryTier.SESSION: self._session,
            MemoryTier.PROJECT: self._project,
            MemoryTier.LONG_TERM: self._long_term,
        }

    @property
    def session_id(self) -> str:
        """获取会话 ID"""
        return self._session_id

    def remember(
        self,
        content: str,
        tier: MemoryTier = MemoryTier.WORKING,
        metadata: Optional[Dict[str, Any]] = None,
        importance: float = 0.5
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

        # TODO: 调用 sh-core 实际存储
        storage = self._storage.get(tier, self._working)
        storage.append(entry)

        # 工作记忆限制大小
        if tier == MemoryTier.WORKING and len(storage) > 100:
            storage.pop(0)

        return entry.id

    def recall(
        self,
        query: str,
        tier: Optional[MemoryTier] = None,
        limit: int = 10
    ) -> List[MemoryEntry]:
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
        tiers = [tier] if tier else [
            MemoryTier.WORKING,
            MemoryTier.SESSION,
            MemoryTier.PROJECT,
            MemoryTier.LONG_TERM,
        ]

        for t in tiers:
            storage = self._storage.get(t, [])
            for entry in storage:
                if query.lower() in entry.content.lower():
                    entry.touch()
                    results.append(entry)
                    if len(results) >= limit:
                        return results

        return results

    def get(self, tier: MemoryTier, memory_id: str) -> Optional[MemoryEntry]:
        """获取特定记忆

        Args:
            tier: 记忆层级
            memory_id: 记忆 ID

        Returns:
            记忆条目（如果存在）
        """
        storage = self._storage.get(tier, [])
        for entry in storage:
            if entry.id == memory_id:
                entry.touch()
                return entry
        return None

    def forget(self, tier: MemoryTier, memory_id: str) -> bool:
        """删除记忆

        Args:
            tier: 记忆层级
            memory_id: 记忆 ID

        Returns:
            是否成功删除
        """
        storage = self._storage.get(tier, [])
        for i, entry in enumerate(storage):
            if entry.id == memory_id:
                storage.pop(i)
                return True
        return False

    def clear(self, tier: MemoryTier) -> int:
        """清空指定层级

        Args:
            tier: 记忆层级

        Returns:
            删除的记忆数量
        """
        storage = self._storage.get(tier, [])
        count = len(storage)
        storage.clear()
        return count

    def stats(self) -> Dict[MemoryTier, int]:
        """获取各层级统计

        Returns:
            各层级记忆数量
        """
        return {
            tier: len(storage)
            for tier, storage in self._storage.items()
        }

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

    def to_dict(self) -> Dict[str, Any]:
        """导出为字典"""
        return {
            "session_id": self._session_id,
            "stats": {tier.value: count for tier, count in self.stats().items()},
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Memory':
        """从字典创建"""
        return cls(session_id=data.get("session_id", ""))
