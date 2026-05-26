"""Retriever Engine

检索引擎：向量相似度检索和 RAG 支持。

Features:
    - 文档索引与检索
    - 多种分块策略（固定大小、段落、代码）
    - 混合检索（向量 + 关键词）
    - RAG Pipeline（带重排序）
"""

from __future__ import annotations

import threading
import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Protocol, runtime_checkable

from .vectorstore import DistanceMetric, InMemoryVectorStore, VectorItem

if TYPE_CHECKING:
    from collections.abc import Callable

__all__ = [
    "Document",
    "RetrievalResult",
    "Chunk",
    "ChunkPosition",
    "RetrieverEngine",
    "DefaultRetrieverEngine",
    "EmbeddingModel",
    "ChunkingStrategy",
    "FixedSizeChunker",
    "HybridWeights",
    "MockEmbeddingModel",
]


@dataclass
class Document:
    """文档结构

    Attributes:
        id: 文档 ID（可选，自动生成）
        content: 文档内容
        metadata: 元数据
        source: 来源（文件路径、URL 等）
    """

    content: str
    id: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)
    source: str | None = None

    def with_source(self, source: str) -> Document:
        """设置来源"""
        self.source = source
        return self

    def with_metadata(self, key: str, value: Any) -> Document:
        """添加元数据"""
        self.metadata[key] = value
        return self


@dataclass
class RetrievalResult:
    """检索结果

    Attributes:
        doc_id: 文档 ID
        content: 文档内容
        score: 相似度分数 (0.0-1.0)
        metadata: 元数据
        source: 来源
    """

    doc_id: str
    content: str
    score: float
    metadata: dict[str, Any] = field(default_factory=dict)
    source: str | None = None


@dataclass
class ChunkPosition:
    """分块位置

    Attributes:
        start: 起始字符位置
        end: 结束字符位置
        index: 分块索引
        total: 总分块数
    """

    start: int
    end: int
    index: int
    total: int


@dataclass
class Chunk:
    """文档分块

    Attributes:
        id: 分块 ID
        doc_id: 文档 ID
        content: 分块内容
        position: 在原文中的位置
        metadata: 元数据
    """

    id: str
    doc_id: str
    content: str
    position: ChunkPosition
    metadata: dict[str, Any] = field(default_factory=dict)


class HybridWeights:
    """混合检索权重配置

    用于配置向量搜索和关键词搜索的权重比例。
    """

    def __init__(self, vector: float = 0.7, keyword: float = 0.3):
        """初始化权重

        Args:
            vector: 向量搜索权重 (默认 0.7)
            keyword: 关键词搜索权重 (默认 0.3)

        Raises:
            ValueError: 权重之和不为 1.0
        """
        if abs(vector + keyword - 1.0) > 0.001:
            raise ValueError(f"Weights must sum to 1.0, got {vector + keyword}")
        self.vector = vector
        self.keyword = keyword

    @classmethod
    def vector_only(cls) -> HybridWeights:
        """仅使用向量搜索"""
        return cls(vector=1.0, keyword=0.0)

    @classmethod
    def keyword_only(cls) -> HybridWeights:
        """仅使用关键词搜索"""
        return cls(vector=0.0, keyword=1.0)

    @classmethod
    def balanced(cls) -> HybridWeights:
        """均衡权重"""
        return cls(vector=0.5, keyword=0.5)


class RetrieverEngine(ABC):
    """检索引擎抽象基类

    提供向量相似度检索能力。
    """

    @abstractmethod
    async def index(self, documents: list[Document]) -> list[str]:
        """索引文档

        Args:
            documents: 要索引的文档列表

        Returns:
            文档 ID 列表
        """
        pass

    @abstractmethod
    async def retrieve(self, query: str, top_k: int = 5) -> list[RetrievalResult]:
        """检索相似文档

        Args:
            query: 查询文本
            top_k: 返回结果数量

        Returns:
            检索结果列表
        """
        pass

    @abstractmethod
    async def hybrid_retrieve(
        self, query: str, top_k: int = 5, weights: HybridWeights | None = None
    ) -> list[RetrievalResult]:
        """混合检索（向量 + 关键词）

        Args:
            query: 查询文本
            top_k: 返回结果数量
            weights: 权重配置（默认 70% 向量 + 30% 关键词）

        Returns:
            检索结果列表
        """
        pass

    async def retrieve_with_filter(
        self, query: str, top_k: int = 5, filter: dict[str, Any] | None = None
    ) -> list[RetrievalResult]:
        """带过滤条件的检索

        Args:
            query: 查询文本
            top_k: 返回结果数量
            filter: 元数据过滤条件

        Returns:
            检索结果列表
        """
        _ = filter
        return await self.retrieve(query, top_k)

    @abstractmethod
    async def delete(self, doc_ids: list[str]) -> bool:
        """删除文档

        Args:
            doc_ids: 要删除的文档 ID 列表

        Returns:
            是否成功删除
        """
        pass

    @abstractmethod
    async def clear(self) -> bool:
        """清空索引

        Returns:
            是否成功清空
        """
        pass

    @abstractmethod
    async def count(self) -> int:
        """获取文档数量

        Returns:
            文档数量
        """
        pass


@runtime_checkable
class EmbeddingModel(Protocol):
    """Embedding 模型协议

    定义文本嵌入向量生成接口。
    """

    async def embed(self, text: str) -> list[float]:
        """生成文本嵌入向量

        Args:
            text: 输入文本

        Returns:
            嵌入向量
        """
        ...

    async def embed_batch(self, texts: list[str]) -> list[list[float]]:
        """批量生成嵌入向量

        Args:
            texts: 输入文本列表

        Returns:
            嵌入向量列表
        """
        ...

    @property
    def dimension(self) -> int:
        """向量维度"""
        ...

    @property
    def model_name(self) -> str:
        """模型名称"""
        ...


@runtime_checkable
class ChunkingStrategy(Protocol):
    """分块策略协议

    定义文档分块接口。
    """

    def chunk(self, document: Document) -> list[Chunk]:
        """分块文档

        Args:
            document: 输入文档

        Returns:
            分块列表
        """
        ...


class FixedSizeChunker:
    """固定大小分块策略

    按固定字符数分块，支持重叠。
    """

    def __init__(self, chunk_size: int = 500, overlap: int = 50):
        """初始化分块器

        Args:
            chunk_size: 分块大小（字符数）
            overlap: 重叠大小
        """
        self.chunk_size = chunk_size
        self.overlap = overlap

    def chunk(self, document: Document) -> list[Chunk]:
        """分块文档"""
        content = document.content

        # 内容小于分块大小，直接返回
        if len(content) <= self.chunk_size:
            return [
                Chunk(
                    id=f"{document.id or 'doc'}-0",
                    doc_id=document.id or "",
                    content=content,
                    position=ChunkPosition(
                        start=0, end=len(content), index=0, total=1
                    ),
                    metadata=document.metadata.copy(),
                )
            ]

        chunks: list[Chunk] = []
        start = 0
        index = 0
        doc_id = document.id or str(uuid.uuid4())

        while start < len(content):
            end = min(start + self.chunk_size, len(content))
            chunks.append(
                Chunk(
                    id=f"{doc_id}-{index}",
                    doc_id=doc_id,
                    content=content[start:end],
                    position=ChunkPosition(
                        start=start, end=end, index=index, total=0
                    ),
                    metadata=document.metadata.copy(),
                )
            )

            # 防止死循环：到达末尾时直接设置 start = end
            start = end - self.overlap if end < len(content) else end
            index += 1

        # 更新总分块数
        total = len(chunks)
        for chunk in chunks:
            chunk.position.total = total

        return chunks


class ParagraphChunker:
    """段落分块策略

    按自然段落边界分块，保持语义完整性。
    适合文档、文章等自然语言内容。
    """

    def __init__(self, max_chunk_size: int = 1000, min_chunk_size: int = 100):
        """初始化段落分块器

        Args:
            max_chunk_size: 最大分块大小
            min_chunk_size: 最小分块大小
        """
        self.max_chunk_size = max_chunk_size
        self.min_chunk_size = min_chunk_size

    def chunk(self, document: Document) -> list[Chunk]:
        """分块文档"""
        content = document.content
        paragraphs = [p for p in content.split("\n") if p.strip()]

        if not paragraphs:
            return [
                Chunk(
                    id=f"{document.id or 'doc'}-0",
                    doc_id=document.id or "",
                    content=content,
                    position=ChunkPosition(
                        start=0, end=len(content), index=0, total=1
                    ),
                    metadata=document.metadata.copy(),
                )
            ]

        chunks: list[Chunk] = []
        current_chunk = ""
        start = 0
        index = 0

        for paragraph in paragraphs:
            if len(current_chunk) + len(paragraph) + 1 <= self.max_chunk_size:
                if current_chunk:
                    current_chunk += "\n"
                current_chunk += paragraph
            else:
                if len(current_chunk) >= self.min_chunk_size:
                    end = start + len(current_chunk)
                    chunks.append(
                        Chunk(
                            id=f"{document.id or 'doc'}-{index}",
                            doc_id=document.id or "",
                            content=current_chunk,
                            position=ChunkPosition(
                                start=start, end=end, index=index, total=0
                            ),
                            metadata=document.metadata.copy(),
                        )
                    )
                    start = end
                    index += 1

                current_chunk = paragraph

        # 处理最后一个分块
        if len(current_chunk) >= self.min_chunk_size:
            chunks.append(
                Chunk(
                    id=f"{document.id or 'doc'}-{index}",
                    doc_id=document.id or "",
                    content=current_chunk,
                    position=ChunkPosition(
                        start=start, end=len(content), index=index, total=0
                    ),
                    metadata=document.metadata.copy(),
                )
            )

        total = len(chunks) or 1
        for chunk in chunks:
            chunk.position.total = total

        if not chunks:
            return [
                Chunk(
                    id=f"{document.id or 'doc'}-0",
                    doc_id=document.id or "",
                    content=content,
                    position=ChunkPosition(
                        start=0, end=len(content), index=0, total=1
                    ),
                    metadata=document.metadata.copy(),
                )
            ]

        return chunks


class RecursiveChunker:
    """递归分块策略

    依次尝试多种分隔符，从大到小。
    适合通用文本，保持语义完整性。
    """

    def __init__(self, max_chunk_size: int = 1000):
        """初始化递归分块器

        Args:
            max_chunk_size: 最大分块大小
        """
        self.max_chunk_size = max_chunk_size
        self._separators = ["\n\n\n", "\n\n", "\n", ". ", " ", ""]

    def chunk(self, document: Document) -> list[Chunk]:
        """分块文档"""
        return self._recursive_split(document, document.content, 0, 0)

    def _recursive_split(
        self,
        document: Document,
        text: str,
        start_offset: int,
        initial_index: int,
    ) -> list[Chunk]:
        """递归分块"""
        if len(text) <= self.max_chunk_size:
            return [
                Chunk(
                    id=f"{document.id or 'doc'}-{initial_index}",
                    doc_id=document.id or "",
                    content=text,
                    position=ChunkPosition(
                        start=start_offset,
                        end=start_offset + len(text),
                        index=initial_index,
                        total=1,
                    ),
                    metadata=document.metadata.copy(),
                )
            ]

        for separator in self._separators:
            if separator == "":
                # 最后手段：按字符分割
                chunks: list[Chunk] = []
                start = 0
                index = initial_index

                while start < len(text):
                    end = min(start + self.max_chunk_size, len(text))
                    chunks.append(
                        Chunk(
                            id=f"{document.id or 'doc'}-{index}",
                            doc_id=document.id or "",
                            content=text[start:end],
                            position=ChunkPosition(
                                start=start_offset + start,
                                end=start_offset + end,
                                index=index,
                                total=0,
                            ),
                            metadata=document.metadata.copy(),
                        )
                    )
                    start = end
                    index += 1

                total = len(chunks)
                for chunk in chunks:
                    chunk.position.total = total
                return chunks

            if separator in text:
                parts = text.split(separator)
                chunks = []
                current_chunk = ""
                current_start = start_offset
                index = initial_index

                for i, part in enumerate(parts):
                    part_with_sep = f"{part}{separator}" if i < len(parts) - 1 else part

                    if len(current_chunk) + len(part_with_sep) <= self.max_chunk_size:
                        current_chunk += part_with_sep
                    else:
                        if current_chunk:
                            chunks.append(
                                Chunk(
                                    id=f"{document.id or 'doc'}-{index}",
                                    doc_id=document.id or "",
                                    content=current_chunk,
                                    position=ChunkPosition(
                                        start=current_start,
                                        end=current_start + len(current_chunk),
                                        index=index,
                                        total=0,
                                    ),
                                    metadata=document.metadata.copy(),
                                )
                            )
                            current_start += len(current_chunk)
                            index += 1

                        if len(part_with_sep) > self.max_chunk_size:
                            # 递归分割
                            sub_chunks = self._recursive_split(
                                document, part_with_sep, current_start, index
                            )
                            for sub in sub_chunks:
                                current_start = sub.position.end
                                index += 1
                                chunks.append(sub)
                        else:
                            current_chunk = part_with_sep

                if current_chunk:
                    chunks.append(
                        Chunk(
                            id=f"{document.id or 'doc'}-{index}",
                            doc_id=document.id or "",
                            content=current_chunk,
                            position=ChunkPosition(
                                start=current_start,
                                end=start_offset + len(text),
                                index=index,
                                total=0,
                            ),
                            metadata=document.metadata.copy(),
                        )
                    )

                total = len(chunks) or 1
                for chunk in chunks:
                    chunk.position.total = total
                return chunks

        return [
            Chunk(
                id=f"{document.id or 'doc'}-{initial_index}",
                doc_id=document.id or "",
                content=text,
                position=ChunkPosition(
                    start=start_offset,
                    end=start_offset + len(text),
                    index=initial_index,
                    total=1,
                ),
                metadata=document.metadata.copy(),
            )
        ]


class MockEmbeddingModel:
    """Mock Embedding 模型（用于测试）

    生成基于文本哈希的伪向量。
    """

    def __init__(self, dimension: int = 128):
        """初始化 Mock 模型

        Args:
            dimension: 向量维度
        """
        self._dimension = dimension

    async def embed(self, text: str) -> list[float]:
        """生成基于文本的伪向量"""
        vector: list[float] = []
        bytes_data = text.encode("utf-8")
        for i in range(self._dimension):
            byte_val = bytes_data[i % len(bytes_data)] if bytes_data else 0
            vector.append(byte_val / 255.0)
        return vector

    async def embed_batch(self, texts: list[str]) -> list[list[float]]:
        """批量生成嵌入向量"""
        embeddings: list[list[float]] = []
        for text in texts:
            embeddings.append(await self.embed(text))
        return embeddings

    @property
    def dimension(self) -> int:
        """向量维度"""
        return self._dimension

    @property
    def model_name(self) -> str:
        """模型名称"""
        return "mock-embedding-model"


# Re-export from vectorstore for convenience
VectorStoreAdapter = InMemoryVectorStore


class DefaultRetrieverEngine(RetrieverEngine):
    """默认检索引擎实现

    结合 Embedding 模型、分块策略和向量存储提供完整的 RAG 功能。

    Example:
        >>> from continuum_sdk.rag import DefaultRetrieverEngine, MockEmbeddingModel, FixedSizeChunker
        >>>
        >>> # 创建引擎
        >>> engine = DefaultRetrieverEngine(
        ...     embedding_model=MockEmbeddingModel(128),
        ...     chunker=FixedSizeChunker()
        ... )
        >>>
        >>> # 索引文档
        >>> doc = Document(content="Hello world", source="test.txt")
        >>> doc_ids = await engine.index([doc])
        >>>
        >>> # 检索
        >>> results = await engine.retrieve("Hello", top_k=5)
    """

    def __init__(
        self,
        embedding_model: EmbeddingModel,
        chunker: ChunkingStrategy | None = None,
        vector_store: InMemoryVectorStore | None = None,
        hybrid_weights: HybridWeights | None = None,
    ):
        """初始化检索引擎

        Args:
            embedding_model: Embedding 模型
            chunker: 分块策略（默认 FixedSizeChunker）
            vector_store: 向量存储（默认 InMemoryVectorStore）
            hybrid_weights: 混合检索权重（默认 70% 向量 + 30% 关键词）
        """
        self._embedding_model = embedding_model
        self._chunker = chunker or FixedSizeChunker()
        self._vector_store = vector_store or InMemoryVectorStore()
        self._hybrid_weights = hybrid_weights or HybridWeights()

        # 文档索引（文档 ID -> 分块 ID 列表）
        self._doc_index: dict[str, list[str]] = {}
        # 分块内容缓存（分块 ID -> 内容）
        self._chunk_cache: dict[str, str] = {}
        self._lock = threading.RLock()

    async def index(self, documents: list[Document]) -> list[str]:
        """索引文档"""
        doc_ids: list[str] = []

        for doc in documents:
            # 生成文档 ID
            doc_id = doc.id or str(uuid.uuid4())

            # 创建带 ID 的文档副本
            doc_with_id = Document(
                id=doc_id,
                content=doc.content,
                metadata=doc.metadata.copy(),
                source=doc.source,
            )

            # 分块
            chunks = self._chunker.chunk(doc_with_id)
            chunk_ids = [c.id for c in chunks]
            chunk_contents = [c.content for c in chunks]

            # 批量生成 embeddings
            embeddings = await self._embedding_model.embed_batch(chunk_contents)

            # 构建向量项并存储
            with self._lock:
                for chunk, embedding in zip(chunks, embeddings, strict=True):
                    metadata = chunk.metadata.copy()
                    metadata["doc_id"] = chunk.doc_id
                    metadata["chunk_index"] = chunk.position.index
                    if doc.source:
                        metadata["source"] = doc.source

                    # 使用 VectorStore 的同步接口
                    self._vector_store.upsert(
                        chunk.id,
                        embedding,
                        metadata,
                    )

                    # 缓存分块内容
                    self._chunk_cache[chunk.id] = chunk.content

                # 记录文档索引
                self._doc_index[doc_id] = chunk_ids

            doc_ids.append(doc_id)

        return doc_ids

    async def retrieve(self, query: str, top_k: int = 5) -> list[RetrievalResult]:
        """检索相似文档"""
        # 生成查询向量
        query_embedding = await self._embedding_model.embed(query)

        # 搜索相似向量（使用 VectorStore 的同步接口）
        results = self._vector_store.search(query_embedding, top_k)

        # 转换为 RetrievalResult
        retrieval_results: list[RetrievalResult] = []
        with self._lock:
            for r in results:
                content = r.content or self._chunk_cache.get(r.id, "")
                retrieval_results.append(
                    RetrievalResult(
                        doc_id=r.id,
                        content=content,
                        score=r.score,
                        metadata=r.metadata.copy(),
                        source=r.metadata.get("source"),
                    )
                )

        return retrieval_results

    async def hybrid_retrieve(
        self, query: str, top_k: int = 5, weights: HybridWeights | None = None
    ) -> list[RetrievalResult]:
        """混合检索（向量 + 关键词）"""
        w = weights or self._hybrid_weights

        # 向量搜索
        vector_results = await self.retrieve(query, top_k * 2)

        if w.keyword == 0.0:
            return vector_results[:top_k]

        # 关键词匹配增强
        query_lower = query.lower()
        query_keywords = query_lower.split()

        # 重新计算分数（向量分数 + 关键词匹配奖励）
        scored_results: list[RetrievalResult] = []
        for r in vector_results:
            content_lower = r.content.lower()
            keyword_matches = sum(1 for kw in query_keywords if kw in content_lower)

            # 混合分数
            keyword_score = (
                (keyword_matches / max(len(query_keywords), 1)) * w.keyword
                if query_keywords
                else 0.0
            )
            final_score = r.score * w.vector + keyword_score

            scored_results.append(
                RetrievalResult(
                    doc_id=r.doc_id,
                    content=r.content,
                    score=final_score,
                    metadata=r.metadata.copy(),
                    source=r.source,
                )
            )

        # 按分数排序并截断
        scored_results.sort(key=lambda x: x.score, reverse=True)
        return scored_results[:top_k]

    async def delete(self, doc_ids: list[str]) -> bool:
        """删除文档"""
        all_chunk_ids: list[str] = []

        with self._lock:
            for doc_id in doc_ids:
                if doc_id in self._doc_index:
                    chunk_ids = self._doc_index.pop(doc_id)
                    for chunk_id in chunk_ids:
                        self._chunk_cache.pop(chunk_id, None)
                    all_chunk_ids.extend(chunk_ids)

        if not all_chunk_ids:
            return False

        # 使用 VectorStore 的同步批量删除接口
        count = self._vector_store.delete_batch(all_chunk_ids)
        return count > 0

    async def clear(self) -> bool:
        """清空索引"""
        # 使用 VectorStore 的同步接口
        self._vector_store.clear()
        with self._lock:
            self._doc_index.clear()
            self._chunk_cache.clear()
        return True

    async def count(self) -> int:
        """获取文档数量"""
        with self._lock:
            return len(self._doc_index)
