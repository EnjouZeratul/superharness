"""Vector Store Implementation

向量存储：持久化向量索引。

Features:
    - 内存向量存储（适合测试和开发）
    - 多种距离度量支持（Cosine, Euclidean, DotProduct, Manhattan）
    - 批量操作优化
    - 元数据过滤支持
"""

from __future__ import annotations

import math
import threading
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class DistanceMetric(Enum):
    """距离度量类型"""

    COSINE = "cosine"
    EUCLIDEAN = "euclidean"
    DOT_PRODUCT = "dot_product"
    MANHATTAN = "manhattan"


@dataclass
class VectorItem:
    """向量项"""

    id: str
    vector: list[float]
    metadata: dict[str, Any] = field(default_factory=dict)
    content: str | None = None


@dataclass
class SearchResult:
    """搜索结果"""

    id: str
    score: float
    content: str
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class MetadataFilter:
    """元数据过滤条件"""

    must: dict[str, Any] = field(default_factory=dict)
    should: dict[str, Any] = field(default_factory=dict)
    must_not: dict[str, Any] = field(default_factory=dict)

    def matches(self, metadata: dict[str, Any]) -> bool:
        """检查元数据是否匹配过滤条件"""
        # 检查 must 条件（全部匹配）
        for key, value in self.must.items():
            if key not in metadata or metadata[key] != value:
                return False

        # 检查 must_not 条件（全部不匹配）
        for key, value in self.must_not.items():
            if key in metadata and metadata[key] == value:
                return False

        # 检查 should 条件（至少一个匹配，如果为空则通过）
        if self.should:
            matched = False
            for key, value in self.should.items():
                if key in metadata and metadata[key] == value:
                    matched = True
                    break
            if not matched:
                return False

        return True


class VectorStore(ABC):
    """向量存储抽象类"""

    @abstractmethod
    def upsert(self, id: str, vector: list[float], metadata: dict[str, Any] | None = None) -> bool:
        """插入或更新向量

        Args:
            id: 向量唯一标识
            vector: 向量数据
            metadata: 元数据

        Returns:
            是否成功
        """
        pass

    @abstractmethod
    def search(
        self,
        vector: list[float],
        top_k: int = 10,
        filter: MetadataFilter | None = None,
    ) -> list[SearchResult]:
        """搜索相似向量

        Args:
            vector: 查询向量
            top_k: 返回数量
            filter: 元数据过滤条件

        Returns:
            搜索结果列表
        """
        pass

    @abstractmethod
    def delete(self, id: str) -> bool:
        """删除向量

        Args:
            id: 向量唯一标识

        Returns:
            是否成功
        """
        pass

    @abstractmethod
    def get(self, id: str) -> VectorItem | None:
        """获取向量

        Args:
            id: 向量唯一标识

        Returns:
            向量项或 None
        """
        pass

    @abstractmethod
    def count(self) -> int:
        """获取向量数量"""
        pass

    @abstractmethod
    def clear(self) -> bool:
        """清空存储"""
        pass


def cosine_similarity(a: list[float], b: list[float]) -> float:
    """计算余弦相似度"""
    if len(a) != len(b):
        return 0.0

    dot = sum(x * y for x, y in zip(a, b))
    norm_a = math.sqrt(sum(x * x for x in a))
    norm_b = math.sqrt(sum(x * x for x in b))

    if norm_a == 0.0 or norm_b == 0.0:
        return 0.0

    return dot / (norm_a * norm_b)


def euclidean_similarity(a: list[float], b: list[float]) -> float:
    """计算欧几里得相似度（距离转换为相似度）"""
    if len(a) != len(b):
        return 0.0

    sum_sq = sum((x - y) ** 2 for x, y in zip(a, b))
    return 1.0 / (1.0 + math.sqrt(sum_sq))


def dot_product_similarity(a: list[float], b: list[float]) -> float:
    """计算点积相似度"""
    if len(a) != len(b):
        return 0.0

    return sum(x * y for x, y in zip(a, b))


def manhattan_similarity(a: list[float], b: list[float]) -> float:
    """计算曼哈顿相似度（距离转换为相似度）"""
    if len(a) != len(b):
        return 0.0

    sum_abs = sum(abs(x - y) for x, y in zip(a, b))
    return 1.0 / (1.0 + sum_abs)


class InMemoryVectorStore(VectorStore):
    """内存向量存储实现

    使用内存存储向量，支持基本的相似度搜索。
    适用于测试和开发环境，不适合大规模生产使用。
    """

    def __init__(self, metric: DistanceMetric = DistanceMetric.COSINE):
        """初始化内存向量存储

        Args:
            metric: 距离度量类型
        """
        self._data: dict[str, VectorItem] = {}
        self._metric = metric
        self._lock = threading.RLock()

        # 选择相似度计算函数
        self._similarity_funcs = {
            DistanceMetric.COSINE: cosine_similarity,
            DistanceMetric.EUCLIDEAN: euclidean_similarity,
            DistanceMetric.DOT_PRODUCT: dot_product_similarity,
            DistanceMetric.MANHATTAN: manhattan_similarity,
        }

    def _compute_similarity(self, a: list[float], b: list[float]) -> float:
        """计算向量相似度"""
        return self._similarity_funcs[self._metric](a, b)

    def upsert(self, id: str, vector: list[float], metadata: dict[str, Any] | None = None) -> bool:
        """插入或更新向量"""
        with self._lock:
            self._data[id] = VectorItem(
                id=id,
                vector=vector,
                metadata=metadata or {},
            )
            return True

    def search(
        self,
        vector: list[float],
        top_k: int = 10,
        filter: MetadataFilter | None = None,
    ) -> list[SearchResult]:
        """搜索相似向量"""
        with self._lock:
            # 先过滤
            candidates: list[VectorItem] = []
            for item in self._data.values():
                if filter is None or filter.matches(item.metadata):
                    candidates.append(item)

            # 计算相似度
            scores: list[tuple[VectorItem, float]] = []
            for item in candidates:
                score = self._compute_similarity(vector, item.vector)
                scores.append((item, score))

            # 按相似度降序排序
            scores.sort(key=lambda x: x[1], reverse=True)

            # 取 top_k
            results: list[SearchResult] = []
            for item, score in scores[:top_k]:
                results.append(SearchResult(
                    id=item.id,
                    score=score,
                    content=item.content or "",
                    metadata=item.metadata,
                ))

            return results

    def delete(self, id: str) -> bool:
        """删除向量"""
        with self._lock:
            if id in self._data:
                del self._data[id]
                return True
            return False

    def get(self, id: str) -> VectorItem | None:
        """获取向量"""
        with self._lock:
            return self._data.get(id)

    def count(self) -> int:
        """获取向量数量"""
        with self._lock:
            return len(self._data)

    def clear(self) -> bool:
        """清空存储"""
        with self._lock:
            self._data.clear()
            return True

    # 批量操作
    def upsert_batch(self, items: list[tuple[str, list[float], dict[str, Any] | None]]) -> list[bool]:
        """批量插入或更新向量

        Args:
            items: (id, vector, metadata) 元组列表

        Returns:
            每个操作的结果列表
        """
        results = []
        with self._lock:
            for id, vector, metadata in items:
                self._data[id] = VectorItem(
                    id=id,
                    vector=vector,
                    metadata=metadata or {},
                )
                results.append(True)
        return results

    def delete_batch(self, ids: list[str]) -> int:
        """批量删除向量

        Args:
            ids: 向量 ID 列表

        Returns:
            删除的数量
        """
        count = 0
        with self._lock:
            for id in ids:
                if id in self._data:
                    del self._data[id]
                    count += 1
        return count
