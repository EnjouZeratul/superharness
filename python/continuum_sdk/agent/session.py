"""
Session Module

Session management for Continuum SDK.

A Session represents a single conversation thread between a user and an Agent.
It maintains the message history, metadata, and tracks usage statistics.

Key Features:
    - Message history with roles (user, assistant, system, tool)
    - Metadata storage for custom session data
    - Tool usage tracking
    - Cost and token tracking
    - Export/import for persistence
    - File-based persistence (save/load)

Example:
    >>> session = Session(id="my-session")
    >>> session.add_user_message("Hello")
    >>> session.add_assistant_message("Hi there!")
    >>> print(session.message_count)  # 2
    >>> session.save("~/.continuum/sessions/my-session.json")
    >>> session2 = Session.load("~/.continuum/sessions/my-session.json")
"""

from typing import Optional, List, Dict, Any, Union
from datetime import datetime
from enum import Enum
from pathlib import Path
import json
import os

# Import Rust bindings
try:
    from sh_core import Session as RustSession
    HAS_RUST_BINDINGS = True
except ImportError:
    HAS_RUST_BINDINGS = False


class MessageRole(Enum):
    """消息角色枚举"""
    USER = "user"
    ASSISTANT = "assistant"
    SYSTEM = "system"
    TOOL = "tool"


class Message:
    """会话消息"""

    def __init__(
        self,
        role: MessageRole,
        content: str,
        timestamp: Optional[datetime] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ):
        self.role = role
        self.content = content
        self.timestamp = timestamp or datetime.now()
        self.metadata = metadata or {}

    def to_dict(self) -> Dict[str, Any]:
        return {
            "role": self.role.value,
            "content": self.content,
            "timestamp": self.timestamp.isoformat(),
            "metadata": self.metadata,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Message":
        return cls(
            role=MessageRole(data["role"]),
            content=data["content"],
            timestamp=datetime.fromisoformat(data["timestamp"]),
            metadata=data.get("metadata", {}),
        )


class Session:
    """
    Continuum Session class.

    Manages a single conversation thread's state and history.

    A Session maintains:
        - Message history with timestamps
        - Metadata (key-value store)
        - Tool usage tracking
        - Cost and token statistics

    Sessions can be exported to JSON and restored later, enabling
    conversation persistence and resumption.

    Attributes:
        id: Unique session identifier
        created_at: Session creation timestamp
        message_count: Number of messages in history
        cost: Total accumulated cost
        tokens: Total token count

    Example:
        >>> session = Session(id="chat-001")
        >>> session.add_user_message("What is Python?")
        >>> session.add_assistant_message("Python is a programming language.")
        >>> print(session.get_last_message().content)
        'Python is a programming language.'
    """

    def __init__(self, id: Optional[str] = None):
        """
        Create a new Session.

        Args:
            id: Optional session identifier. Auto-generated if not provided.
        """
        self._id = id or "default-session"
        self._messages: List[Message] = []
        self._created_at = datetime.now()
        self._metadata: Dict[str, Any] = {}
        self._tools_used: List[str] = []
        self._cost: float = 0.0
        self._token_count: int = 0

        # Rust 绑定
        if HAS_RUST_BINDINGS:
            self._rust_session = RustSession(self._id)
        else:
            self._rust_session = None

    @property
    def id(self) -> str:
        """会话 ID"""
        if self._rust_session:
            return self._rust_session.id()
        return self._id

    @property
    def created_at(self) -> datetime:
        """创建时间"""
        if self._rust_session:
            return datetime.fromisoformat(self._rust_session.created_at())
        return self._created_at

    @property
    def message_count(self) -> int:
        """消息数量"""
        if self._rust_session:
            return self._rust_session.message_count()
        return len(self._messages)

    @property
    def cost(self) -> float:
        """总成本"""
        return self._cost

    @property
    def tokens(self) -> int:
        """Token 数量"""
        return self._token_count

    def add_message(
        self,
        role: MessageRole,
        content: str,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> Message:
        """
        添加消息

        Args:
            role: 消息角色
            content: 消息内容
            metadata: 元数据

        Returns:
            添加的消息
        """
        message = Message(role=role, content=content, metadata=metadata)
        self._messages.append(message)

        # 同步到 Rust
        if self._rust_session:
            if role == MessageRole.USER:
                self._rust_session.add_user_message(content)
            elif role == MessageRole.ASSISTANT:
                self._rust_session.add_assistant_message(content)

        return message

    def add_user_message(self, content: str) -> Message:
        """添加用户消息"""
        return self.add_message(MessageRole.USER, content)

    def add_assistant_message(self, content: str) -> Message:
        """添加助手消息"""
        return self.add_message(MessageRole.ASSISTANT, content)

    def add_system_message(self, content: str) -> Message:
        """添加系统消息"""
        return self.add_message(MessageRole.SYSTEM, content)

    def get_messages(self) -> List[Message]:
        """获取所有消息"""
        if self._rust_session:
            # 从 Rust 绑定获取
            rust_messages = self._rust_session.get_messages()
            return [
                Message(
                    role=MessageRole(r[0]),
                    content=r[1],
                )
                for r in rust_messages
            ]
        return self._messages.copy()

    def get_last_message(self) -> Optional[Message]:
        """获取最后一条消息"""
        if not self._messages:
            return None
        return self._messages[-1]

    def clear_messages(self) -> None:
        """清空消息历史"""
        self._messages.clear()
        if self._rust_session:
            self._rust_session.clear_messages()

    def set_metadata(self, key: str, value: Any) -> None:
        """设置元数据"""
        self._metadata[key] = value

    def get_metadata(self, key: str) -> Optional[Any]:
        """获取元数据"""
        return self._metadata.get(key)

    def record_tool_use(self, tool_name: str) -> None:
        """记录工具使用"""
        self._tools_used.append(tool_name)

    def get_tools_used(self) -> List[str]:
        """获取使用的工具列表"""
        return self._tools_used.copy()

    def update_cost(self, cost: float, tokens: int) -> None:
        """更新成本"""
        self._cost += cost
        self._token_count += tokens

    def export(self) -> str:
        """导出会话为 JSON"""
        if self._rust_session:
            return self._rust_session.export()

        data = {
            "id": self._id,
            "created_at": self._created_at.isoformat(),
            "messages": [m.to_dict() for m in self._messages],
            "metadata": self._metadata,
            "tools_used": self._tools_used,
            "cost": self._cost,
            "tokens": self._token_count,
        }
        return json.dumps(data, indent=2)

    @classmethod
    def from_export(cls, export_data: str) -> "Session":
        """从导出数据恢复会话"""
        data = json.loads(export_data)
        session = cls(id=data["id"])
        session._created_at = datetime.fromisoformat(data["created_at"])
        session._messages = [Message.from_dict(m) for m in data["messages"]]
        session._metadata = data.get("metadata", {})
        session._tools_used = data.get("tools_used", [])
        session._cost = data.get("cost", 0.0)
        session._token_count = data.get("tokens", 0)
        return session

    def __repr__(self) -> str:
        return f"Session(id={self._id}, messages={len(self._messages)})"

    def save(self, path: Union[str, Path]) -> None:
        """
        Save session to file.

        Args:
            path: File path to save to (JSON format)
        """
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        data = {
            "id": self._id,
            "created_at": self._created_at.isoformat(),
            "messages": [m.to_dict() for m in self._messages],
            "metadata": self._metadata,
            "tools_used": self._tools_used,
            "cost": self._cost,
            "tokens": self._token_count,
            "version": "1.0",
        }

        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)

    @classmethod
    def load(cls, path: Union[str, Path]) -> "Session":
        """
        Load session from file.

        Args:
            path: File path to load from

        Returns:
            Restored Session instance

        Raises:
            FileNotFoundError: If file doesn't exist
            ValueError: If file format is invalid
        """
        path = Path(path)
        if not path.exists():
            raise FileNotFoundError(f"Session file not found: {path}")

        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)

        session = cls(id=data["id"])
        session._created_at = datetime.fromisoformat(data["created_at"])
        session._messages = [Message.from_dict(m) for m in data.get("messages", [])]
        session._metadata = data.get("metadata", {})
        session._tools_used = data.get("tools_used", [])
        session._cost = data.get("cost", 0.0)
        session._token_count = data.get("tokens", 0)

        return session

    def delete(self, path: Union[str, Path]) -> None:
        """
        Delete session file.

        Args:
            path: File path to delete
        """
        path = Path(path)
        if path.exists():
            path.unlink()

    @staticmethod
    def get_default_session_dir() -> Path:
        """
        Get default session storage directory.

        Returns:
            Path to ~/.continuum/sessions/
        """
        home = Path.home()
        return home / ".continuum" / "sessions"

    def save_to_default(self) -> Path:
        """
        Save session to default directory.

        Returns:
            Path where session was saved
        """
        session_dir = self.get_default_session_dir()
        path = session_dir / f"{self._id}.json"
        self.save(path)
        return path

    @classmethod
    def load_from_default(cls, session_id: str) -> "Session":
        """
        Load session from default directory.

        Args:
            session_id: Session ID to load

        Returns:
            Restored Session instance
        """
        session_dir = cls.get_default_session_dir()
        path = session_dir / f"{session_id}.json"
        return cls.load(path)

    @classmethod
    def list_saved_sessions(cls) -> List[str]:
        """
        List all saved session IDs in default directory.

        Returns:
            List of session IDs
        """
        session_dir = cls.get_default_session_dir()
        if not session_dir.exists():
            return []

        return [f.stem for f in session_dir.glob("*.json")]


def create_session(id: Optional[str] = None) -> Session:
    """
    Convenience function to create a Session.

    Args:
        id: Session ID

    Returns:
        Session instance
    """
    return Session(id=id)