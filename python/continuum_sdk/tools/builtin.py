"""Built-in Tools API

Provides Python access to Continuum's built-in tools via Rust binding.

[STABILITY: STABLE] Core tools 稳定可用
[NOTE] 当 Rust binding 不可用时，自动降级到纯 Python 实现

The BuiltinTools class wraps the Rust ToolExecutor for high-performance
file operations, search, and shell command execution.

When Rust binding is not available, it automatically falls back to pure
Python implementations from file_ops, search, and bash modules.

Features:
    - File operations: read, write, edit, list directory
    - Search tools: grep (regex search), glob (pattern match)
    - Shell execution: bash commands with timeout support
    - Tool discovery: list available tools and capabilities
    - Category classification: automatic tool categorization
    - Automatic fallback to pure Python when Rust binding unavailable

Quick Start:
    >>> from continuum_sdk.tools import BuiltinTools
    >>> tools = BuiltinTools()
    >>>
    >>> # Check available tools
    >>> for tool in tools.list_tools():
    ...     print(f"{tool.name}: {tool.description}")
    >>>
    >>> # Read a file
    >>> content = tools.read_file("README.md")
    >>>
    >>> # Search in files
    >>> matches = tools.grep("TODO", path="src/", glob="*.py")
    >>>
    >>> # Find files by pattern
    >>> files = tools.glob("**/*.py")

File Operations:
    >>> # Read entire file
    >>> content = tools.read_file("config.json")
    >>>
    >>> # Read specific lines
    >>> lines = tools.read_file("large.log", offset=100, limit=50)
    >>>
    >>> # Write file
    >>> tools.write_file("output.txt", "Hello, World!")
    >>>
    >>> # Edit file (find and replace)
    >>> tools.edit_file("app.py", old="DEBUG = False", new="DEBUG = True")
    >>>
    >>> # List directory
    >>> entries = tools.list_directory("src/")
    >>> for entry in entries:
    ...     print(f"{entry['name']} ({entry['type']})")

Search Operations:
    >>> # Grep for pattern
    >>> results = tools.grep("def test_", path="tests/", glob="*.py")
    >>>
    >>> # Find files
    >>> py_files = tools.glob("**/*.py", path="src/")

Shell Operations:
    >>> # Run command
    >>> output = tools.bash("git status --short")
    >>>
    >>> # With timeout
    >>> output = tools.bash("npm install", timeout_ms=60000)
    >>>
    >>> # In specific directory
    >>> output = tools.bash("pytest", working_dir="project/")

Tool Categories:
    - FILE_OPS: read_file, write_file, edit_file, list_directory
    - SEARCH: grep, glob
    - SHELL: bash
    - CODE_ANALYSIS: definition, reference, hover (LSP tools)
    - MEMORY: session memory operations
    - WORKFLOW: checkpoint operations

Fallback Mode:
    When Rust binding (sh_python.pyd) is not available, all core tools
    automatically fall back to pure Python implementations:
    - file_ops.read_file, file_ops.write_file, file_ops.edit_file
    - search.grep, search.glob
    - bash.bash_execute_sync

Requirements:
    Rust binding (sh_python.pyd) recommended for best performance.
    Pure Python fallback available without any native dependencies.
"""

import json
from dataclasses import dataclass
from enum import Enum
from typing import Any

# Import Rust binding
try:
    from sh_python import ToolExecutor as RustToolExecutor

    HAS_RUST_BINDING = True
except ImportError:
    HAS_RUST_BINDING = False

    # Define placeholder for type annotation
    class RustToolExecutor:
        pass

# Import Python fallback implementations
from .file_ops import read_file, write_file, edit_file
from .search import grep, glob
from .bash import bash_execute_sync


class ToolCategory(Enum):
    """Tool categories."""

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
    """Tool metadata."""

    name: str
    description: str
    category: ToolCategory
    requires_confirmation: bool = False
    is_dangerous: bool = False
    parameters: dict[str, Any] = None

    def __post_init__(self):
        if self.parameters is None:
            self.parameters = {}


@dataclass
class ToolResult:
    """Tool execution result."""

    call_id: str
    name: str
    content: str
    is_error: bool = False
    duration_ms: int = 0


class BuiltinTools:
    """Built-in tools collection.

    Wraps Rust ToolExecutor for real file/shell operations.

    Example:
        >>> tools = BuiltinTools()
        >>> content = tools.read_file("README.md")
        >>> print(content[:100])
    """

    _tools_cache: dict[str, ToolMeta] = None
    _executor: RustToolExecutor | None = None

    def __init__(self):
        """Initialize built-in tools."""
        self._tools_cache = {}
        if HAS_RUST_BINDING:
            self._executor = RustToolExecutor()
        self._load_tools()

    def _load_tools(self) -> None:
        """Load tool list from Rust binding or use defaults."""
        if self._executor:
            for name, desc in self._executor.list_tools():
                self._tools_cache[name] = ToolMeta(
                    name=name,
                    description=desc,
                    category=self._guess_category(name),
                )
        else:
            # Fallback without Rust binding
            builtin_tool_names = [
                "read_file",
                "write_file",
                "edit_file",
                "list_directory",
                "grep",
                "glob",
                "bash",
            ]
            for name in builtin_tool_names:
                self._tools_cache[name] = ToolMeta(
                    name=name,
                    description=f"Built-in tool: {name}",
                    category=self._guess_category(name),
                )

    def _guess_category(self, name: str) -> ToolCategory:
        """Guess category from tool name."""
        if any(
            x in name for x in ["file", "directory", "read", "write", "edit", "list"]
        ):
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

    def _check_binding(self, name: str) -> None:
        """Check if tool is available (Rust binding or Python fallback)."""
        if not self._executor and name not in self._fallback_tools:
            raise NotImplementedError(
                f"Tool '{name}' requires Rust binding. "
                "Ensure sh_python.pyd is in the package directory."
            )

    @property
    def _fallback_tools(self) -> set[str]:
        """Tools available via Python fallback."""
        return {"read_file", "write_file", "edit_file", "list_directory", "grep", "glob", "bash"}

    # ==================== File Operations ====================

    def read_file(
        self, path: str, offset: int | None = None, limit: int | None = None
    ) -> str:
        """Read file contents.

        Args:
            path: File path
            offset: Starting line number (optional)
            limit: Number of lines to read (optional)

        Returns:
            File contents
        """
        if self._executor:
            return self._executor.read_file(path, offset, limit)

        # Python fallback
        result = read_file(path, offset, limit, show_line_numbers=True)
        return result.content

    def write_file(self, path: str, content: str) -> str:
        """Write content to file.

        Args:
            path: File path
            content: Content to write

        Returns:
            Result message
        """
        if self._executor:
            return self._executor.write_file(path, content)

        # Python fallback
        result = write_file(path, content)
        return result.content

    def edit_file(self, path: str, old: str, new: str) -> str:
        """Edit file by replacing text.

        Args:
            path: File path
            old: Text to replace
            new: New text

        Returns:
            Result message
        """
        if self._executor:
            args = json.dumps({"path": path, "old_string": old, "new_string": new})
            return self._executor.execute("edit_file", args)

        # Python fallback
        result = edit_file(path, old, new)
        return result.content

    def list_directory(self, path: str) -> list[dict[str, Any]]:
        """List directory contents.

        Args:
            path: Directory path

        Returns:
            List of entries with name, type (file/dir)
        """
        if self._executor:
            result = self._executor.execute("list_directory", json.dumps({"path": path}))
            try:
                return json.loads(result)
            except json.JSONDecodeError:
                return [{"raw": result}]

        # Python fallback using pathlib
        from pathlib import Path
        dir_path = Path(path).expanduser().resolve()
        if not dir_path.exists():
            return [{"error": f"Directory not found: {dir_path}"}]

        entries = []
        for entry in dir_path.iterdir():
            entries.append({
                "name": entry.name,
                "type": "dir" if entry.is_dir() else "file",
                "path": str(entry),
            })
        return sorted(entries, key=lambda e: (e["type"], e["name"]))

    # ==================== Search ====================

    def grep(
        self, pattern: str, path: str | None = None, glob: str | None = None
    ) -> str:
        """Search file contents.

        Args:
            pattern: Regex pattern
            path: Search path (optional)
            glob: File filter pattern (optional)

        Returns:
            Search results
        """
        if self._executor:
            return self._executor.grep(pattern, path, glob)

        # Python fallback
        result = grep(pattern, path, glob)
        return result.content

    def glob(self, pattern: str, path: str | None = None) -> str:
        """Find files matching pattern.

        Args:
            pattern: Glob pattern (e.g., "**/*.py")
            path: Search path (optional)

        Returns:
            Matching file paths
        """
        if self._executor:
            return self._executor.glob(pattern, path)

        # Python fallback (rename to avoid shadowing)
        from .search import glob as glob_search
        result = glob_search(pattern, path)
        return result.content

    # ==================== Shell ====================

    def bash(
        self,
        command: str,
        timeout_ms: int | None = None,
        working_dir: str | None = None,
    ) -> str:
        """Execute shell command.

        Args:
            command: Bash command
            timeout_ms: Timeout in milliseconds
            working_dir: Working directory

        Returns:
            Command output
        """
        if self._executor:
            return self._executor.bash(command, timeout_ms, working_dir)

        # Python fallback
        timeout_sec = (timeout_ms / 1000) if timeout_ms else 120.0
        result = bash_execute_sync(command, timeout=timeout_sec, working_dir=working_dir)
        return result.content

    # ==================== Tool Discovery ====================

    def list_tools(self) -> list[ToolMeta]:
        """List all available tools."""
        return list(self._tools_cache.values())

    def is_available(self, name: str) -> bool:
        """Check if tool is available."""
        if self._executor:
            return self._executor.is_available(name)
        return name in self._fallback_tools

    def get_tool_meta(self, name: str) -> ToolMeta | None:
        """Get tool metadata by name.

        Args:
            name: Tool name

        Returns:
            ToolMeta if found, None otherwise
        """
        return self._tools_cache.get(name)

    def execute(self, name: str, args: dict[str, Any]) -> str:
        """Execute tool by name.

        Args:
            name: Tool name
            args: Tool arguments

        Returns:
            Tool result
        """
        if self._executor:
            return self._executor.execute(name, json.dumps(args))

        # Python fallback routing
        if name == "read_file":
            return self.read_file(args.get("path"), args.get("offset"), args.get("limit"))
        elif name == "write_file":
            return self.write_file(args.get("path"), args.get("content"))
        elif name == "edit_file":
            return self.edit_file(args.get("path"), args.get("old"), args.get("new"))
        elif name == "list_directory":
            result = self.list_directory(args.get("path"))
            return json.dumps(result)
        elif name == "grep":
            return self.grep(args.get("pattern"), args.get("path"), args.get("glob"))
        elif name == "glob":
            return self.glob(args.get("pattern"), args.get("path"))
        elif name == "bash":
            return self.bash(args.get("command"), args.get("timeout_ms"), args.get("working_dir"))
        else:
            raise NotImplementedError(f"Tool '{name}' not available in Python fallback mode")


# Module-level singleton for convenience
_builtin_tools: BuiltinTools | None = None


def get_builtin_tools() -> BuiltinTools:
    """Get or create BuiltinTools singleton."""
    global _builtin_tools
    if _builtin_tools is None:
        _builtin_tools = BuiltinTools()
    return _builtin_tools
