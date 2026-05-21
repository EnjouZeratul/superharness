"""
Continuum SDK Tools Module

Real tool implementations for file operations, search, and shell execution.

Tools:
    - BashTool: Safe command execution with timeout and output capture
    - ReadTool: File reading with pagination and encoding detection
    - WriteTool: Safe file writing with backup
    - EditTool: Precise string replacement in files
    - GrepTool: Regex content search
    - GlobTool: File pattern matching

Quick Start:
    >>> from continuum_sdk.tools import BashTool, ReadTool, WriteTool
    >>>
    >>> # Bash
    >>> bash = BashTool()
    >>> result = bash.run("echo hello")
    >>>
    >>> # Read
    >>> reader = ReadTool()
    >>> content = reader.read("config.toml")
    >>>
    >>> # Write
    >>> writer = WriteTool()
    >>> writer.write("output.txt", "Hello!")
"""

# Tool types
from .types import ToolResult, ToolError, ToolMeta, ToolCategory

# Real tool implementations
from .bash import BashTool, bash_execute, bash_execute_sync, validate_command
from .file_ops import (
    ReadTool,
    WriteTool,
    EditTool,
    read_file,
    write_file,
    edit_file,
    detect_encoding,
)
from .search import (
    GrepTool,
    GlobTool,
    grep,
    glob,
)

# Legacy compatibility (custom tools)
from .custom import CustomTool, ToolRegistry, tool, register_tool, get_registry

__all__ = [
    # Types
    "ToolResult",
    "ToolError",
    "ToolMeta",
    "ToolCategory",
    # Bash
    "BashTool",
    "bash_execute",
    "bash_execute_sync",
    "validate_command",
    # Read
    "ReadTool",
    "read_file",
    "detect_encoding",
    # Write
    "WriteTool",
    "write_file",
    # Edit
    "EditTool",
    "edit_file",
    # Search
    "GrepTool",
    "GlobTool",
    "grep",
    "glob",
    # Custom tools
    "CustomTool",
    "ToolRegistry",
    "tool",
    "register_tool",
    "get_registry",
]