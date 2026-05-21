"""
SessionManager 并发控制机制

修复说明：
- 原设计使用 threading.RLock() 并调用 wait() 方法，这是错误的
- RLock 没有 wait() 方法，只有 Condition 才有
- 本实现使用 Condition + Lock 实现读写分离锁

依赖：httpx + pydantic（无额外依赖）
"""

import threading
from contextlib import contextmanager
from typing import Dict, Callable, TypeVar, Optional, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
import uuid
import json
from pathlib import Path
import hashlib
import logging

# 配置日志
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

T = TypeVar('T')


class AgentState(Enum):
    """Agent执行状态机"""
    IDLE = "idle"
    RUNNING = "running"
    TOOL_CALLING = "tool_calling"
    WAITING_TOOL = "waiting_tool"
    STOPPED = "stopped"
    ERROR = "error"
    COMPLETED = "completed"


# ============================================================================
# 读写分离锁实现（核心修复）
# ============================================================================

class ReadWriteLock:
    """
    读写分离锁

    特性：
    - 读操作可并发执行（共享锁）
    - 写操作需互斥执行（排他锁）
    - 写优先：当有写者等待时，新的读者会被阻塞

    使用 threading.Condition 实现，而非错误的 RLock.wait()
    """

    def __init__(self):
        # 使用 Lock 保护内部状态
        self._state_lock = threading.Lock()
        # 使用 Condition 实现等待/通知机制
        self._read_condition = threading.Condition(self._state_lock)
        self._write_condition = threading.Condition(self._state_lock)

        # 状态计数
        self._readers: int = 0  # 当前活跃读者数
        self._writers: int = 0  # 当前活跃写者数（最多1）
        self._waiting_writers: int = 0  # 等待中的写者数
        self._waiting_readers: int = 0  # 等待中的读者数

        # 写优先标志
        self._write_preferred: bool = False

    @contextmanager
    def read_lock(self, timeout: Optional[float] = None):
        """
        获取读锁

        多个读者可以同时持有读锁
        当有写者活跃或等待时，读者会被阻塞

        Args:
            timeout: 超时时间（秒），None表示无限等待
        """
        acquired = False
        try:
            with self._state_lock:
                self._waiting_readers += 1
                # 等待条件：没有活跃写者，且没有写者在等待（写优先）
                while self._writers > 0 or (self._write_preferred and self._waiting_writers > 0):
                    result = self._read_condition.wait(timeout)
                    if result is False:  # 超时
                        self._waiting_readers -= 1
                        raise TimeoutError("获取读锁超时")
                self._waiting_readers -= 1
                self._readers += 1
                acquired = True
            yield
        finally:
            if acquired:
                with self._state_lock:
                    self._readers -= 1
                    # 如果没有读者了，通知等待的写者
                    if self._readers == 0:
                        self._write_condition.notify_all()
                        self._read_condition.notify_all()

    @contextmanager
    def write_lock(self, timeout: Optional[float] = None):
        """
        获取写锁

        写锁是排他的，同一时间只能有一个写者
        当有读者或写者活跃时，新的写者会被阻塞

        Args:
            timeout: 超时时间（秒），None表示无限等待
        """
        acquired = False
        try:
            with self._state_lock:
                self._waiting_writers += 1
                self._write_preferred = True  # 设置写优先标志
                # 等待条件：没有活跃读者和写者
                while self._readers > 0 or self._writers > 0:
                    result = self._write_condition.wait(timeout)
                    if result is False:  # 超时
                        self._waiting_writers -= 1
                        self._write_preferred = False
                        raise TimeoutError("获取写锁超时")
                self._waiting_writers -= 1
                self._writers += 1
                acquired = True
            yield
        finally:
            if acquired:
                with self._state_lock:
                    self._writers -= 1
                    # 如果没有等待的写者了，清除写优先标志
                    if self._waiting_writers == 0:
                        self._write_preferred = False
                    # 通知所有等待的线程
                    self._write_condition.notify_all()
                    self._read_condition.notify_all()

    def get_state(self) -> Dict[str, int]:
        """获取当前锁状态（用于调试）"""
        with self._state_lock:
            return {
                "readers": self._readers,
                "writers": self._writers,
                "waiting_readers": self._waiting_readers,
                "waiting_writers": self._waiting_writers,
                "write_preferred": self._write_preferred
            }


# ============================================================================
# 简化版状态锁（如果不需要读写分离，可用此简化版本）
# ============================================================================

class SimpleStateLock:
    """
    简化版状态锁

    使用单一的 RLock 保护状态，适用于读写分离不是瓶颈的场景
    """

    def __init__(self):
        self._lock = threading.RLock()
        self._condition = threading.Condition(self._lock)

    @contextmanager
    def read_lock(self):
        """读锁（与写锁互斥）"""
        with self._lock:
            yield

    @contextmanager
    def write_lock(self):
        """写锁（与读锁互斥）"""
        with self._lock:
            yield

    def wait_for_condition(self, condition: Callable[[], bool], timeout: Optional[float] = None) -> bool:
        """
        等待条件满足

        Args:
            condition: 条件函数，返回True表示条件满足
            timeout: 超时时间
        """
        with self._condition:
            return self._condition.wait_for(condition, timeout)


# ============================================================================
# 并发安全的 ExecutionContext
# ============================================================================

@dataclass
class ExecutionContext:
    """Agent执行的完整上下文，支持序列化"""
    # 标识
    session_id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])
    agent_id: str = "default"

    # 状态
    state: AgentState = AgentState.IDLE
    iteration: int = 0
    max_iterations: int = 100

    # 消息历史（OpenAI格式）
    messages: list = field(default_factory=list)

    # Tool管理
    tools_registered: list = field(default_factory=list)
    tool_calls_pending: list = field(default_factory=list)
    tool_results_cache: dict = field(default_factory=dict)

    # 配置快照
    model: str = "gpt-4o"
    temperature: float = 0.7
    system_prompt: str = ""

    # 追踪数据
    tokens_total: int = 0
    tokens_prompt: int = 0
    tokens_completion: int = 0
    cost_estimate: float = 0.0

    # 元数据
    created_at: datetime = field(default_factory=datetime.now)
    last_updated: datetime = field(default_factory=datetime.now)
    checkpoint_count: int = 0

    def to_dict(self) -> Dict[str, Any]:
        """转换为字典"""
        return {
            "session_id": self.session_id,
            "agent_id": self.agent_id,
            "state": self.state.value,
            "iteration": self.iteration,
            "max_iterations": self.max_iterations,
            "messages": self.messages,
            "tools_registered": self.tools_registered,
            "model": self.model,
            "temperature": self.temperature,
            "tokens_total": self.tokens_total,
            "tokens_prompt": self.tokens_prompt,
            "tokens_completion": self.tokens_completion,
            "cost_estimate": self.cost_estimate,
            "created_at": self.created_at.isoformat(),
            "last_updated": self.last_updated.isoformat(),
            "checkpoint_count": self.checkpoint_count
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ExecutionContext':
        """从字典恢复"""
        return cls(
            session_id=data.get("session_id", str(uuid.uuid4())[:8]),
            agent_id=data.get("agent_id", "default"),
            state=AgentState(data.get("state", "idle")),
            iteration=data.get("iteration", 0),
            max_iterations=data.get("max_iterations", 100),
            messages=data.get("messages", []),
            tools_registered=data.get("tools_registered", []),
            model=data.get("model", "gpt-4o"),
            temperature=data.get("temperature", 0.7),
            tokens_total=data.get("tokens_total", 0),
            tokens_prompt=data.get("tokens_prompt", 0),
            tokens_completion=data.get("tokens_completion", 0),
            cost_estimate=data.get("cost_estimate", 0.0),
            created_at=datetime.fromisoformat(data["created_at"]) if "created_at" in data else datetime.now(),
            last_updated=datetime.fromisoformat(data["last_updated"]) if "last_updated" in data else datetime.now(),
            checkpoint_count=data.get("checkpoint_count", 0)
        )


# ============================================================================
# 并发安全的 SessionManager
# ============================================================================

class ConcurrentSessionManager:
    """
    并发安全会话管理器

    特性：
    - 使用读写分离锁，读操作可并发，写操作互斥
    - 支持多线程环境下的安全访问
    - 支持超时获取锁
    """

    def __init__(self, max_sessions: int = 100):
        self._sessions: Dict[str, ExecutionContext] = {}
        self._locks: Dict[str, ReadWriteLock] = {}
        self._global_lock = threading.Lock()
        self._max_sessions = max_sessions

    def create_session(self, config: Optional[Dict] = None) -> ExecutionContext:
        """
        创建新会话（线程安全）

        Args:
            config: 可选的配置字典
        """
        with self._global_lock:
            if len(self._sessions) >= self._max_sessions:
                raise RuntimeError(f"已达到最大会话数限制: {self._max_sessions}")

            context = ExecutionContext()
            if config:
                if "model" in config:
                    context.model = config["model"]
                if "temperature" in config:
                    context.temperature = config["temperature"]
                if "system_prompt" in config:
                    context.system_prompt = config["system_prompt"]

            self._sessions[context.session_id] = context
            self._locks[context.session_id] = ReadWriteLock()

            logger.info(f"创建会话: {context.session_id}")
            return context

    def get_or_create_session(
        self,
        session_id: Optional[str] = None,
        config: Optional[Dict] = None
    ) -> ExecutionContext:
        """
        获取或创建会话（线程安全）

        Args:
            session_id: 可选的会话ID，如果不存在则创建
            config: 创建新会话时的配置
        """
        with self._global_lock:
            if session_id and session_id in self._sessions:
                return self._sessions[session_id]

            if len(self._sessions) >= self._max_sessions:
                raise RuntimeError(f"已达到最大会话数限制: {self._max_sessions}")

            context = ExecutionContext()
            if session_id:
                context.session_id = session_id
            if config:
                if "model" in config:
                    context.model = config["model"]
                if "temperature" in config:
                    context.temperature = config["temperature"]

            self._sessions[context.session_id] = context
            self._locks[context.session_id] = ReadWriteLock()

            return context

    def update_session(
        self,
        session_id: str,
        update_fn: Callable[[ExecutionContext], None],
        timeout: Optional[float] = None
    ) -> bool:
        """
        更新会话状态（线程安全，使用写锁）

        Args:
            session_id: 会话ID
            update_fn: 更新函数，接收ExecutionContext作为参数
            timeout: 获取锁的超时时间
        """
        if session_id not in self._locks:
            logger.warning(f"会话不存在: {session_id}")
            return False

        try:
            with self._locks[session_id].write_lock(timeout):
                context = self._sessions[session_id]
                update_fn(context)
                context.last_updated = datetime.now()
                return True
        except TimeoutError:
            logger.warning(f"获取写锁超时: {session_id}")
            return False

    def read_session(
        self,
        session_id: str,
        read_fn: Callable[[ExecutionContext], T],
        timeout: Optional[float] = None
    ) -> Optional[T]:
        """
        读取会话状态（线程安全，使用读锁）

        Args:
            session_id: 会话ID
            read_fn: 读取函数，接收ExecutionContext作为参数
            timeout: 获取锁的超时时间
        """
        if session_id not in self._locks:
            logger.warning(f"会话不存在: {session_id}")
            return None

        try:
            with self._locks[session_id].read_lock(timeout):
                context = self._sessions[session_id]
                return read_fn(context)
        except TimeoutError:
            logger.warning(f"获取读锁超时: {session_id}")
            return None

    def get_session_state(self, session_id: str) -> Optional[AgentState]:
        """获取会话状态（读操作，可并发）"""
        return self.read_session(session_id, lambda ctx: ctx.state)

    def set_session_state(self, session_id: str, new_state: AgentState) -> bool:
        """设置会话状态（写操作，需互斥）"""
        return self.update_session(session_id, lambda ctx: setattr(ctx, 'state', new_state))

    def add_message(self, session_id: str, role: str, content: str) -> bool:
        """添加消息（写操作）"""
        def _add(ctx: ExecutionContext):
            ctx.messages.append({"role": role, "content": content})
            ctx.iteration += 1

        return self.update_session(session_id, _add)

    def get_messages(self, session_id: str) -> Optional[list]:
        """获取消息列表（读操作）"""
        return self.read_session(session_id, lambda ctx: list(ctx.messages))

    def delete_session(self, session_id: str) -> bool:
        """删除会话"""
        with self._global_lock:
            if session_id in self._sessions:
                del self._sessions[session_id]
                del self._locks[session_id]
                logger.info(f"删除会话: {session_id}")
                return True
            return False

    def list_sessions(self) -> list:
        """列出所有会话ID"""
        with self._global_lock:
            return list(self._sessions.keys())

    def get_lock_state(self, session_id: str) -> Optional[Dict]:
        """获取锁状态（用于调试）"""
        if session_id not in self._locks:
            return None
        return self._locks[session_id].get_state()

    def get_stats(self) -> Dict:
        """获取统计信息"""
        with self._global_lock:
            return {
                "total_sessions": len(self._sessions),
                "max_sessions": self._max_sessions
            }


# ============================================================================
# Checkpoint 管理（带并发安全）
# ============================================================================

class CheckpointManager:
    """Checkpoint管理器"""

    def __init__(self, storage_path: str = "~/.continuum/sessions"):
        self.storage_path = Path(storage_path).expanduser()
        self.storage_path.mkdir(parents=True, exist_ok=True)
        self._lock = threading.Lock()

    def save_checkpoint(
        self,
        context: ExecutionContext,
        trigger: str = "manual"
    ) -> str:
        """保存Checkpoint"""
        checkpoint_id = str(uuid.uuid4())[:8]

        session_dir = self.storage_path / context.session_id / "checkpoints"
        session_dir.mkdir(parents=True, exist_ok=True)

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"cp_{timestamp}_{checkpoint_id}.json"
        filepath = session_dir / filename

        data = context.to_dict()
        data["checkpoint_id"] = checkpoint_id
        data["trigger"] = trigger
        data["created_at"] = datetime.now().isoformat()

        # 计算校验和
        data["_checksum"] = self._compute_checksum(data)

        with self._lock:
            with open(filepath, 'w', encoding='utf-8') as f:
                json.dump(data, f, ensure_ascii=False, indent=2)

        logger.info(f"保存Checkpoint: {filename}")
        return checkpoint_id

    def load_checkpoint(self, session_id: str, checkpoint_id: Optional[str] = None) -> Optional[ExecutionContext]:
        """加载Checkpoint"""
        session_dir = self.storage_path / session_id / "checkpoints"
        if not session_dir.exists():
            return None

        if checkpoint_id:
            # 查找指定checkpoint
            files = list(session_dir.glob(f"*_{checkpoint_id}.json"))
        else:
            # 加载最新的checkpoint
            files = sorted(session_dir.glob("cp_*.json"), reverse=True)

        if not files:
            return None

        filepath = files[0]

        with self._lock:
            with open(filepath, 'r', encoding='utf-8') as f:
                data = json.load(f)

        # 验证校验和
        expected_checksum = data.pop("_checksum", None)
        actual_checksum = self._compute_checksum(data)

        if expected_checksum and expected_checksum != actual_checksum:
            logger.error(f"Checkpoint校验失败: {filepath}")
            return None

        return ExecutionContext.from_dict(data)

    def _compute_checksum(self, data: Dict) -> str:
        """计算校验和"""
        canonical = json.dumps(data, sort_keys=True)
        return hashlib.sha256(canonical.encode()).hexdigest()[:16]


# ============================================================================
# 示例：Agent运行时集成
# ============================================================================

class AgentRuntime:
    """Agent运行时示例"""

    def __init__(self, session_manager: ConcurrentSessionManager):
        self.session_manager = session_manager
        self.checkpoint_manager = CheckpointManager()

    def start_session(self, user_input: str) -> str:
        """开始新会话"""
        context = self.session_manager.create_session({
            "model": "gpt-4o",
            "temperature": 0.7
        })

        # 添加用户消息
        self.session_manager.add_message(
            context.session_id,
            "user",
            user_input
        )

        # 设置状态
        self.session_manager.set_session_state(
            context.session_id,
            AgentState.RUNNING
        )

        return context.session_id

    def process_tool_call(self, session_id: str, tool_name: str, args: Dict) -> Any:
        """处理工具调用"""
        # 保存checkpoint（Tool调用前）
        session = self.session_manager.get_or_create_session(session_id)
        self.checkpoint_manager.save_checkpoint(session, "before_tool_call")

        # 更新状态
        self.session_manager.set_session_state(session_id, AgentState.TOOL_CALLING)

        # 模拟工具执行
        result = {"tool": tool_name, "args": args, "result": "success"}

        # 恢复运行状态
        self.session_manager.set_session_state(session_id, AgentState.RUNNING)

        return result

    def stop_session(self, session_id: str):
        """停止会话"""
        self.session_manager.set_session_state(session_id, AgentState.STOPPED)
        session = self.session_manager.get_or_create_session(session_id)
        self.checkpoint_manager.save_checkpoint(session, "user_stop")

    def resume_session(self, session_id: str) -> bool:
        """恢复会话"""
        context = self.checkpoint_manager.load_checkpoint(session_id)
        if not context:
            return False

        # 更新状态
        context.state = AgentState.RUNNING
        context.iteration += 1  # 恢复后递增

        return True


# ============================================================================
# 模块入口
# ============================================================================

if __name__ == "__main__":
    print("=" * 60)
    print("SessionManager 并发控制模块")
    print("=" * 60)
    print()
    print("核心组件:")
    print("  - ReadWriteLock: 读写分离锁")
    print("  - SimpleStateLock: 简化版状态锁")
    print("  - ConcurrentSessionManager: 并发安全会话管理器")
    print("  - CheckpointManager: Checkpoint管理器")
    print("  - AgentRuntime: Agent运行时示例")
    print()
    print("使用示例:")
    print("""
    from session_concurrency import ConcurrentSessionManager, ReadWriteLock

    # 创建会话管理器
    manager = ConcurrentSessionManager()

    # 创建会话
    context = manager.create_session({"model": "gpt-4o"})

    # 读操作（可并发）
    messages = manager.get_messages(context.session_id)

    # 写操作（互斥）
    manager.add_message(context.session_id, "user", "Hello!")
    """)
