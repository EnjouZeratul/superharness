"""内置工具 API

提供对 SuperHarness 内置工具的 Python 访问。
"""

from typing import Any, Dict, List, Optional
from enum import Enum
from dataclasses import dataclass


class ToolCategory(Enum):
    """工具分类"""
    FILE_OPS = "file_ops"
    SEARCH = "search"
    SHELL = "shell"
    NETWORK = "network"
    CODE_ANALYSIS = "code_analysis"
    MEMORY = "memory"
    WORKFLOW = "workflow"
    SYSTEM = "system"
    OTHER = "other"


@dataclass
class ToolMeta:
    """工具元数据"""
    name: str
    description: str
    category: ToolCategory
    requires_confirmation: bool = False
    is_dangerous: bool = False
    parameters: Dict[str, Any] = None

    def __post_init__(self):
        if self.parameters is None:
            self.parameters = {}


@dataclass
class ToolResult:
    """工具执行结果"""
    call_id: str
    name: str
    content: str
    is_error: bool = False
    duration_ms: int = 0


class BuiltinTools:
    """内置工具集合

    Usage:
        from superharness_sdk.tools import BuiltinTools

        tools = BuiltinTools()

        # 文件操作
        content = tools.read_file("path/to/file")
        tools.write_file("path/to/file", content)
        tools.edit_file("path/to/file", old="foo", new="bar")

        # 搜索
        matches = tools.grep("pattern", path="src/")
        files = tools.glob("**/*.py")

        # Shell
        result = tools.bash("echo hello")

        # 代码分析
        definition = tools.go_to_definition("src/main.rs", line=10, column=5)
        refs = tools.find_references("src/main.rs", line=10, column=5)
    """

    # 工具列表（从 Rust 层获取）
    _tools_cache: Dict[str, ToolMeta] = None

    def __init__(self):
        """初始化内置工具"""
        self._tools_cache = {}
        self._load_tools()

    def _load_tools(self) -> None:
        """加载工具列表（占位实现）"""
        # TODO: 从 sh-core 获取工具列表
        builtin_tool_names = [
            "read_file", "write_file", "edit_file", "list_directory",
            "grep", "glob",
            "bash",
            "go_to_definition", "find_references", "hover",
            "save_memory", "query_memory",
            "create_checkpoint", "restore_checkpoint",
        ]
        for name in builtin_tool_names:
            self._tools_cache[name] = ToolMeta(
                name=name,
                description=f"Built-in tool: {name}",
                category=self._guess_category(name),
            )

    def _guess_category(self, name: str) -> ToolCategory:
        """根据名称推断分类"""
        if any(x in name for x in ["file", "directory", "read", "write", "edit", "list"]):
            return ToolCategory.FILE_OPS
        if any(x in name for x in ["grep", "glob", "search", "find"]):
            return ToolCategory.SEARCH
        if "bash" in name:
            return ToolCategory.SHELL
        if any(x in name for x in ["definition", "reference", "hover", "symbol"]):
            return ToolCategory.CODE_ANALYSIS
        if "memory" in name:
            return ToolCategory.MEMORY
        if "checkpoint" in name:
            return ToolCategory.WORKFLOW
        return ToolCategory.OTHER

    # ==================== 文件操作 ====================

    def read_file(
        self,
        path: str,
        offset: Optional[int] = None,
        limit: Optional[int] = None
    ) -> str:
        """读取文件内容

        Args:
            path: 文件路径
            offset: 起始行号（可选）
            limit: 读取行数（可选）

        Returns:
            文件内容
        """
        # TODO: 调用 sh-core
        raise NotImplementedError("read_file: waiting for sh-core binding")

    def write_file(self, path: str, content: str) -> None:
        """写入文件内容

        Args:
            path: 文件路径
            content: 写入内容

        Note:
            此操作需要用户确认
        """
        raise NotImplementedError("write_file: waiting for sh-core binding")

    def edit_file(self, path: str, old: str, new: str) -> bool:
        """编辑文件（查找替换）

        Args:
            path: 文件路径
            old: 要替换的文本
            new: 新文本

        Returns:
            是否成功
        """
        raise NotImplementedError("edit_file: waiting for sh-core binding")

    def list_directory(self, path: str) -> List[Dict[str, Any]]:
        """列出目录内容

        Args:
            path: 目录路径

        Returns:
            条目列表，每项包含 name, type (file/dir)
        """
        raise NotImplementedError("list_directory: waiting for sh-core binding")

    # ==================== 搜索 ====================

    def grep(
        self,
        pattern: str,
        path: Optional[str] = None,
        glob: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """搜索文件内容

        Args:
            pattern: 正则表达式
            path: 搜索路径（可选）
            glob: 文件过滤模式（可选）

        Returns:
            匹配结果列表
        """
        raise NotImplementedError("grep: waiting for sh-core binding")

    def glob(self, pattern: str, path: Optional[str] = None) -> List[str]:
        """查找文件

        Args:
            pattern: glob 模式（如 "**/*.py"）
            path: 搜索路径（可选）

        Returns:
            匹配的文件路径列表
        """
        raise NotImplementedError("glob: waiting for sh-core binding")

    # ==================== Shell ====================

    def bash(
        self,
        command: str,
        timeout: Optional[int] = None,
        working_dir: Optional[str] = None
    ) -> ToolResult:
        """执行 Shell 命令

        Args:
            command: bash 命令
            timeout: 超时时间（毫秒）
            working_dir: 工作目录

        Returns:
            执行结果

        Note:
            此操作需要用户确认
        """
        raise NotImplementedError("bash: waiting for sh-core binding")

    # ==================== 代码分析 ====================

    def go_to_definition(
        self,
        file: str,
        line: int,
        column: int
    ) -> Optional[Dict[str, Any]]:
        """跳转到定义

        Args:
            file: 文件路径
            line: 行号（1-based）
            column: 列号（1-based）

        Returns:
            定义位置，包含 file, line, column
        """
        raise NotImplementedError("go_to_definition: waiting for sh-core binding")

    def find_references(
        self,
        file: str,
        line: int,
        column: int
    ) -> List[Dict[str, Any]]:
        """查找引用

        Args:
            file: 文件路径
            line: 行号
            column: 列号

        Returns:
            引用位置列表
        """
        raise NotImplementedError("find_references: waiting for sh-core binding")

    def hover(self, file: str, line: int, column: int) -> Optional[str]:
        """获取悬停信息

        Args:
            file: 文件路径
            line: 行号
            column: 列号

        Returns:
            悬停信息文本
        """
        raise NotImplementedError("hover: waiting for sh-core binding")

    # ==================== 工具元数据 ====================

    def list_tools(self) -> List[ToolMeta]:
        """列出所有内置工具"""
        return list(self._tools_cache.values())

    def get_tool_meta(self, name: str) -> Optional[ToolMeta]:
        """获取工具元数据"""
        return self._tools_cache.get(name)

    def is_available(self, name: str) -> bool:
        """检查工具是否可用"""
        return name in self._tools_cache
