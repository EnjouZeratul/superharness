"""Built-in Tools API

Provides Python access to Continuum's built-in tools via Rust binding.

[STABILITY: STABLE] Core tools 稳定可用
[NOTE] 当 Rust binding 不可用时，自动降级到 placeholder 模式

The BuiltinTools class wraps the Rust ToolExecutor for high-performance
file operations, search, and shell command execution.

Features:
    - File operations: read, write, edit, list directory
    - Search tools: grep (regex search), glob (pattern match)
    - Shell execution: bash commands with timeout support
    - Tool discovery: list available tools and capabilities
    - Category classification: automatic tool categorization

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

Requirements:
    Rust binding (sh_python.pyd) required for real operations.
    Falls back to placeholder mode without the binding.
"""

from typing import Any, Dict, List, Optional
from enum import Enum
from dataclasses import dataclass
import json

# Import Rust binding
try:
    from sh_python import ToolExecutor as RustToolExecutor
    HAS_RUST_BINDING = True
except ImportError:
    HAS_RUST_BINDING = False


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
    parameters: Dict[str, Any] = None

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

    _tools_cache: Dict[str, ToolMeta] = None
    _executor: Optional[RustToolExecutor] = None

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
                "read_file", "write_file", "edit_file", "list_directory",
                "grep", "glob", "bash",
            ]
            for name in builtin_tool_names:
                self._tools_cache[name] = ToolMeta(
                    name=name,
                    description=f"Built-in tool: {name}",
                    category=self._guess_category(name),
                )

    def _guess_category(self, name: str) -> ToolCategory:
        """Guess category from tool name."""
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

    def _check_binding(self, name: str) -> None:
        """Check if Rust binding is available."""
        if not self._executor:
            raise RuntimeError(
                f"Tool '{name}' requires Rust binding. "
                "Ensure sh_python.pyd is in the package directory."
            )

    # ==================== File Operations ====================

    def read_file(
        self,
        path: str,
        offset: Optional[int] = None,
        limit: Optional[int] = None
    ) -> str:
        """Read file contents.

        Args:
            path: File path
            offset: Starting line number (optional)
            limit: Number of lines to read (optional)

        Returns:
            File contents
        """
        self._check_binding("read_file")
        return self._executor.read_file(path, offset, limit)

    def write_file(self, path: str, content: str) -> str:
        """Write content to file.

        Args:
            path: File path
            content: Content to write

        Returns:
            Result message
        """
        self._check_binding("write_file")
        return self._executor.write_file(path, content)

    def edit_file(self, path: str, old: str, new: str) -> str:
        """Edit file by replacing text.

        Args:
            path: File path
            old: Text to replace
            new: New text

        Returns:
            Result message
        """
        self._check_binding("edit_file")
        args = json.dumps({"path": path, "old_string": old, "new_string": new})
        return self._executor.execute("edit_file", args)

    def list_directory(self, path: str) -> List[Dict[str, Any]]:
        """List directory contents.

        Args:
            path: Directory path

        Returns:
            List of entries with name, type (file/dir)
        """
        self._check_binding("list_directory")
        result = self._executor.execute("list_directory", json.dumps({"path": path}))
        try:
            return json.loads(result)
        except json.JSONDecodeError:
            return [{"raw": result}]

    # ==================== Search ====================

    def grep(
        self,
        pattern: str,
        path: Optional[str] = None,
        glob: Optional[str] = None
    ) -> str:
        """Search file contents.

        Args:
            pattern: Regex pattern
            path: Search path (optional)
            glob: File filter pattern (optional)

        Returns:
            Search results
        """
        self._check_binding("grep")
        return self._executor.grep(pattern, path, glob)

    def glob(self, pattern: str, path: Optional[str] = None) -> str:
        """Find files matching pattern.

        Args:
            pattern: Glob pattern (e.g., "**/*.py")
            path: Search path (optional)

        Returns:
            Matching file paths
        """
        self._check_binding("glob")
        return self._executor.glob(pattern, path)

    # ==================== Shell ====================

    def bash(
        self,
        command: str,
        timeout_ms: Optional[int] = None,
        working_dir: Optional[str] = None
    ) -> str:
        """Execute shell command.

        Args:
            command: Bash command
            timeout_ms: Timeout in milliseconds
            working_dir: Working directory

        Returns:
            Command output
        """
        self._check_binding("bash")
        return self._executor.bash(command, timeout_ms, working_dir)

    # ==================== Tool Discovery ====================

    def list_tools(self) -> List[ToolMeta]:
        """List all available tools."""
        return list(self._tools_cache.values())

    def is_available(self, name: str) -> bool:
        """Check if tool is available."""
        if self._executor:
            return self._executor.is_available(name)
        return name in self._tools_cache

    def execute(self, name: str, args: Dict[str, Any]) -> str:
        """Execute tool by name.

        Args:
            name: Tool name
            args: Tool arguments

        Returns:
            Tool result
        """
        self._check_binding(name)
        return self._executor.execute(name, json.dumps(args))


# Module-level singleton for convenience
_builtin_tools: Optional[BuiltinTools] = None


def get_builtin_tools() -> BuiltinTools:
    """Get or create BuiltinTools singleton."""
    global _builtin_tools
    if _builtin_tools is None:
        _builtin_tools = BuiltinTools()
    return _builtin_tools
