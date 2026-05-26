"""Memory Storage Backend

提供存储后端抽象和实现。

[STABILITY: STABLE] 存储接口稳定
"""

from __future__ import annotations

import json
import threading
from abc import ABC, abstractmethod
from datetime import datetime, timezone
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any, TYPE_CHECKING

if TYPE_CHECKING:
    pass


def _utc_now() -> datetime:
    """获取 UTC 时间（timezone-aware）"""
    return datetime.now(timezone.utc)


class MemoryTier(Enum):
    """记忆层级"""

    WORKING = "working"  # 当前对话上下文
    SESSION = "session"  # 会话记忆
    PROJECT = "project"  # 项目知识库
    LONG_TERM = "long_term"  # 跨项目知识


@dataclass
class MemoryEntry:
    """记忆条目"""

    id: str
    tier: MemoryTier
    content: str
    metadata: dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=_utc_now)
    last_accessed: datetime = field(default_factory=_utc_now)
    access_count: int = 0
    importance: float = 0.5

    def touch(self) -> None:
        """更新访问时间和计数"""
        self.last_accessed = _utc_now()
        self.access_count += 1


class StorageBackend(ABC):
    """存储后端抽象类

    支持多种存储后端：
    - MemoryStorage: 内存存储（默认，无持久化）
    - FileStorage: JSON 文件存储
    - SQLiteStorage: SQLite 数据库存储
    """

    @abstractmethod
    def save(self, tier: MemoryTier, entry: MemoryEntry) -> None:
        """保存记忆条目"""
        pass

    @abstractmethod
    def load(self, tier: MemoryTier, entry_id: str) -> MemoryEntry | None:
        """加载记忆条目"""
        pass

    @abstractmethod
    def load_all(self, tier: MemoryTier) -> list[MemoryEntry]:
        """加载指定层级的所有条目"""
        pass

    @abstractmethod
    def delete(self, tier: MemoryTier, entry_id: str) -> bool:
        """删除记忆条目"""
        pass

    @abstractmethod
    def clear(self, tier: MemoryTier) -> int:
        """清空指定层级"""
        pass

    @abstractmethod
    def count(self, tier: MemoryTier) -> int:
        """获取条目数量"""
        pass

    @abstractmethod
    def search(
        self, tier: MemoryTier, query: str, limit: int = 10
    ) -> list[MemoryEntry]:
        """搜索条目"""
        pass

    @abstractmethod
    def flush(self) -> None:
        """刷新缓存到存储"""
        pass

    @abstractmethod
    def close(self) -> None:
        """关闭存储"""
        pass


class MemoryStorage(StorageBackend):
    """内存存储后端

    无持久化，数据仅在内存中保存。
    """

    def __init__(self):
        self._storage: dict[MemoryTier, dict[str, MemoryEntry]] = {
            MemoryTier.WORKING: {},
            MemoryTier.SESSION: {},
            MemoryTier.PROJECT: {},
            MemoryTier.LONG_TERM: {},
        }
        self._lock = threading.RLock()

    def save(self, tier: MemoryTier, entry: MemoryEntry) -> None:
        with self._lock:
            self._storage[tier][entry.id] = entry

    def load(self, tier: MemoryTier, entry_id: str) -> MemoryEntry | None:
        with self._lock:
            return self._storage[tier].get(entry_id)

    def load_all(self, tier: MemoryTier) -> list[MemoryEntry]:
        with self._lock:
            return list(self._storage[tier].values())

    def delete(self, tier: MemoryTier, entry_id: str) -> bool:
        with self._lock:
            if entry_id in self._storage[tier]:
                del self._storage[tier][entry_id]
                return True
            return False

    def clear(self, tier: MemoryTier) -> int:
        with self._lock:
            count = len(self._storage[tier])
            self._storage[tier].clear()
            return count

    def count(self, tier: MemoryTier) -> int:
        with self._lock:
            return len(self._storage[tier])

    def search(
        self, tier: MemoryTier, query: str, limit: int = 10
    ) -> list[MemoryEntry]:
        with self._lock:
            results = []
            for entry in self._storage[tier].values():
                if query.lower() in entry.content.lower():
                    entry.touch()
                    results.append(entry)
                    if len(results) >= limit:
                        break
            return results

    def flush(self) -> None:
        pass

    def close(self) -> None:
        self._storage.clear()


class FileStorage(StorageBackend):
    """JSON 文件存储后端

    每个层级存储为一个 JSON 文件。
    支持自动保存和手动保存。
    """

    def __init__(
        self,
        base_path: str | Path,
        auto_save: bool = True,
        session_id: str = "default",
    ):
        """初始化文件存储

        Args:
            base_path: 存储目录
            auto_save: 是否自动保存（每次修改后保存）
            session_id: 会话 ID
        """
        self._base_path = Path(base_path)
        self._auto_save = auto_save
        self._session_id = session_id

        self._storage: dict[MemoryTier, dict[str, MemoryEntry]] = {
            MemoryTier.WORKING: {},
            MemoryTier.SESSION: {},
            MemoryTier.PROJECT: {},
            MemoryTier.LONG_TERM: {},
        }
        self._lock = threading.RLock()
        self._dirty = False

        self._ensure_dir()
        self._load_all()

    def _ensure_dir(self) -> None:
        """确保存储目录存在"""
        self._base_path.mkdir(parents=True, exist_ok=True)

    def _get_file_path(self, tier: MemoryTier) -> Path:
        """获取层级文件路径"""
        tier_names = {
            MemoryTier.WORKING: "working",
            MemoryTier.SESSION: "session",
            MemoryTier.PROJECT: "project",
            MemoryTier.LONG_TERM: "long_term",
        }
        return self._base_path / f"{self._session_id}_{tier_names[tier]}.json"

    def _entry_to_dict(self, entry: MemoryEntry) -> dict[str, Any]:
        """转换条目为字典"""
        return {
            "id": entry.id,
            "tier": entry.tier.value,
            "content": entry.content,
            "metadata": entry.metadata,
            "created_at": entry.created_at.isoformat(),
            "last_accessed": entry.last_accessed.isoformat(),
            "access_count": entry.access_count,
            "importance": entry.importance,
        }

    def _dict_to_entry(self, data: dict[str, Any]) -> MemoryEntry:
        """转换字典为条目"""
        return MemoryEntry(
            id=data["id"],
            tier=MemoryTier(data["tier"]),
            content=data["content"],
            metadata=data.get("metadata", {}),
            created_at=datetime.fromisoformat(data["created_at"]),
            last_accessed=datetime.fromisoformat(data["last_accessed"]),
            access_count=data.get("access_count", 0),
            importance=data.get("importance", 0.5),
        )

    def _save_tier(self, tier: MemoryTier) -> None:
        """保存层级到文件"""
        file_path = self._get_file_path(tier)
        entries = [self._entry_to_dict(e) for e in self._storage[tier].values()]
        data = {
            "tier": tier.value,
            "session_id": self._session_id,
            "count": len(entries),
            "entries": entries,
            "updated_at": _utc_now().isoformat(),
        }

        with open(file_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    def _load_tier(self, tier: MemoryTier) -> None:
        """从文件加载层级"""
        file_path = self._get_file_path(tier)
        if not file_path.exists():
            return

        try:
            with open(file_path, encoding="utf-8") as f:
                data = json.load(f)

            for entry_data in data.get("entries", []):
                entry = self._dict_to_entry(entry_data)
                self._storage[tier][entry.id] = entry
        except (json.JSONDecodeError, KeyError):
            pass

    def _load_all(self) -> None:
        """加载所有层级"""
        for tier in MemoryTier:
            self._load_tier(tier)

    def save(self, tier: MemoryTier, entry: MemoryEntry) -> None:
        with self._lock:
            self._storage[tier][entry.id] = entry
            self._dirty = True
            if self._auto_save:
                self._save_tier(tier)

    def load(self, tier: MemoryTier, entry_id: str) -> MemoryEntry | None:
        with self._lock:
            entry = self._storage[tier].get(entry_id)
            if entry:
                entry.touch()
            return entry

    def load_all(self, tier: MemoryTier) -> list[MemoryEntry]:
        with self._lock:
            return list(self._storage[tier].values())

    def delete(self, tier: MemoryTier, entry_id: str) -> bool:
        with self._lock:
            if entry_id in self._storage[tier]:
                del self._storage[tier][entry_id]
                self._dirty = True
                if self._auto_save:
                    self._save_tier(tier)
                return True
            return False

    def clear(self, tier: MemoryTier) -> int:
        with self._lock:
            count = len(self._storage[tier])
            self._storage[tier].clear()
            self._dirty = True
            if self._auto_save:
                self._save_tier(tier)
            return count

    def count(self, tier: MemoryTier) -> int:
        with self._lock:
            return len(self._storage[tier])

    def search(
        self, tier: MemoryTier, query: str, limit: int = 10
    ) -> list[MemoryEntry]:
        with self._lock:
            results = []
            for entry in self._storage[tier].values():
                if query.lower() in entry.content.lower():
                    entry.touch()
                    results.append(entry)
                    if len(results) >= limit:
                        break
            return results

    def flush(self) -> None:
        """保存所有修改"""
        with self._lock:
            if self._dirty:
                for tier in MemoryTier:
                    self._save_tier(tier)
                self._dirty = False

    def close(self) -> None:
        """关闭存储，保存所有数据"""
        self.flush()
        self._storage.clear()

    def save_all(self) -> None:
        """保存所有层级"""
        self.flush()

    @staticmethod
    def get_default_storage_path() -> Path:
        """获取默认存储路径"""
        return Path.home() / ".continuum" / "memory"
