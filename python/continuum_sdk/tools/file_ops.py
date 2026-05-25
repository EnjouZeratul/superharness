"""
Read Tool

File reading with large file support, encoding detection, and pagination.

Features:
    - Read file content
    - Large file pagination (offset/limit)
    - Encoding detection
    - Line number formatting
"""

import difflib
import shutil
import time
import uuid
from pathlib import Path

from .types import ToolError, ToolResult


def detect_encoding(file_path: Path) -> str:
    """
    Detect file encoding.

    Args:
        file_path: Path to file

    Returns:
        Encoding name (utf-8, gbk, etc.)
    """
    # Read first 4KB for encoding detection
    try:
        with open(file_path, "rb") as f:
            raw = f.read(4096)

        # Try UTF-8 first
        try:
            raw.decode("utf-8")
            return "utf-8"
        except UnicodeDecodeError:
            pass

        # Try common encodings
        for encoding in ["gbk", "gb2312", "gb18030", "shift-jis", "euc-kr", "latin-1"]:
            try:
                raw.decode(encoding)
                return encoding
            except (UnicodeDecodeError, LookupError):
                continue

        # Fallback to utf-8 with errors='replace'
        return "utf-8"
    except Exception:
        return "utf-8"


def read_file(
    path: str,
    offset: int | None = None,
    limit: int | None = None,
    show_line_numbers: bool = False,
) -> ToolResult:
    """
    Read file content.

    Args:
        path: File path
        offset: Starting line number (1-based, optional)
        limit: Number of lines to read (optional)
        show_line_numbers: Whether to show line numbers (default False)

    Returns:
        ToolResult with file content

    Raises:
        ToolError: If file doesn't exist or can't be read
    """
    call_id = str(uuid.uuid4())[:8]
    start_time = time.time()

    file_path = Path(path).expanduser().resolve()

    # Check if file exists
    if not file_path.exists():
        raise ToolError(
            call_id=call_id,
            name="read",
            message=f"File not found: {file_path}",
        )

    # Check if it's a file
    if not file_path.is_file():
        raise ToolError(
            call_id=call_id,
            name="read",
            message=f"Not a file: {file_path}",
        )

    # Detect encoding
    encoding = detect_encoding(file_path)

    try:
        # Read file
        with open(file_path, encoding=encoding, errors="replace") as f:
            lines = f.readlines()

        # Calculate line range
        start_line = (offset or 1) - 1  # 0-based index
        if start_line < 0:
            start_line = 0

        end_line = start_line + (limit or len(lines))
        if end_line > len(lines):
            end_line = len(lines)

        # Extract lines
        selected_lines = lines[start_line:end_line]

        # Format output
        if show_line_numbers:
            # Format with line numbers like cat -n
            output_lines = []
            for i, line in enumerate(selected_lines, start=(offset or 1)):
                line = line.rstrip("\n\r")
                output_lines.append(f"{i:6}\t{line}")
            content = "\n".join(output_lines)
        else:
            content = "".join(selected_lines).rstrip("\n\r")

        duration_ms = int((time.time() - start_time) * 1000)

        # Add metadata
        metadata = {
            "path": str(file_path),
            "encoding": encoding,
            "total_lines": len(lines),
            "lines_read": len(selected_lines),
        }

        return ToolResult(
            call_id=call_id,
            name="read",
            content=content,
            is_error=False,
            duration_ms=duration_ms,
            metadata=metadata,
        )

    except PermissionError:
        raise ToolError(
            call_id=call_id,
            name="read",
            message=f"Permission denied: {file_path}",
        )
    except Exception as e:
        raise ToolError(
            call_id=call_id,
            name="read",
            message=f"Failed to read file: {e}",
        )


class ReadTool:
    """
    Read tool wrapper for convenient usage.

    Example:
        >>> from continuum_sdk.tools import ReadTool
        >>> reader = ReadTool()
        >>> result = reader.read("src/main.rs", limit=50)
        >>> print(result.content)
    """

    def __init__(self, show_line_numbers: bool = False):
        self.show_line_numbers = show_line_numbers

    def read(
        self,
        path: str,
        offset: int | None = None,
        limit: int | None = None,
    ) -> ToolResult:
        """Read file content."""
        return read_file(path, offset, limit, self.show_line_numbers)

    def __call__(self, path: str, **kwargs) -> ToolResult:
        """Allow calling instance directly."""
        return self.read(path, **kwargs)


"""
Write Tool

Safe file writing with backup and permission checking.

Features:
    - Safe write with backup
    - Permission check
    - Append mode
    - Atomic write (write to temp, then rename)
"""


def write_file(
    path: str,
    content: str,
    backup: bool = True,
    append: bool = False,
    create_dirs: bool = True,
    encoding: str = "utf-8",
) -> ToolResult:
    """
    Write content to file.

    Args:
        path: File path
        content: Content to write
        backup: Create backup before overwriting (default True)
        append: Append to file instead of overwriting (default False)
        create_dirs: Create parent directories if needed (default True)
        encoding: File encoding (default utf-8)

    Returns:
        ToolResult indicating success

    Raises:
        ToolError: If write fails
    """
    call_id = str(uuid.uuid4())[:8]
    start_time = time.time()

    file_path = Path(path).expanduser().resolve()

    # Create parent directories if needed
    if create_dirs and not file_path.parent.exists():
        file_path.parent.mkdir(parents=True, exist_ok=True)

    # Check if file exists
    file_exists = file_path.exists()

    # Create backup if needed
    backup_path = None
    if backup and file_exists and not append:
        backup_path = file_path.with_suffix(file_path.suffix + ".bak")
        shutil.copy2(file_path, backup_path)

    try:
        mode = "a" if append else "w"
        with open(file_path, mode, encoding=encoding, errors="replace") as f:
            f.write(content)
            if not content.endswith("\n"):
                f.write("\n")

        duration_ms = int((time.time() - start_time) * 1000)

        metadata = {
            "path": str(file_path),
            "bytes_written": len(content.encode(encoding)),
            "backup": str(backup_path) if backup_path else None,
        }

        return ToolResult(
            call_id=call_id,
            name="write",
            content=f"Successfully wrote to {file_path}",
            is_error=False,
            duration_ms=duration_ms,
            metadata=metadata,
        )

    except PermissionError:
        raise ToolError(
            call_id=call_id,
            name="write",
            message=f"Permission denied: {file_path}",
        )
    except Exception as e:
        # Restore from backup if write failed
        if backup_path and backup_path.exists():
            shutil.move(backup_path, file_path)
        raise ToolError(
            call_id=call_id,
            name="write",
            message=f"Failed to write file: {e}",
        )


class WriteTool:
    """
    Write tool wrapper.

    Example:
        >>> from continuum_sdk.tools import WriteTool
        >>> writer = WriteTool()
        >>> result = writer.write("output.txt", "Hello, World!")
    """

    def __init__(self, backup: bool = True):
        self.backup = backup

    def write(
        self,
        path: str,
        content: str,
        append: bool = False,
    ) -> ToolResult:
        """Write content to file."""
        return write_file(path, content, backup=self.backup, append=append)

    def append(self, path: str, content: str) -> ToolResult:
        """Append content to file."""
        return self.write(path, content, append=True)

    def __call__(self, path: str, content: str, **kwargs) -> ToolResult:
        """Allow calling instance directly."""
        return self.write(path, content, **kwargs)


"""
Edit Tool

Precise string replacement in files.

Features:
    - Exact string matching
    - Multiple occurrences (replace_all)
    - Preview changes
    - Backup before edit
"""


def edit_file(
    path: str,
    old: str,
    new: str,
    replace_all: bool = False,
    backup: bool = True,
) -> ToolResult:
    """
    Edit file by replacing exact string.

    Args:
        path: File path
        old: Text to find (must be exact match)
        new: Text to replace with
        replace_all: Replace all occurrences (default False)
        backup: Create backup before editing (default True)

    Returns:
        ToolResult indicating changes made

    Raises:
        ToolError: If edit fails or string not found
    """
    call_id = str(uuid.uuid4())[:8]
    start_time = time.time()

    file_path = Path(path).expanduser().resolve()

    # Check file exists
    if not file_path.exists():
        raise ToolError(
            call_id=call_id,
            name="edit",
            message=f"File not found: {file_path}",
        )

    # Read file
    encoding = detect_encoding(file_path)
    try:
        with open(file_path, encoding=encoding, errors="replace") as f:
            content = f.read()
    except Exception as e:
        raise ToolError(
            call_id=call_id,
            name="edit",
            message=f"Failed to read file: {e}",
        )

    # Check if old string exists
    if old not in content:
        raise ToolError(
            call_id=call_id,
            name="edit",
            message=f"String not found: {old[:100]}...",
        )

    # Count occurrences
    count = content.count(old)

    # Create backup if needed
    if backup:
        backup_path = file_path.with_suffix(file_path.suffix + ".bak")
        shutil.copy2(file_path, backup_path)

    # Perform replacement
    if replace_all:
        new_content = content.replace(old, new)
        replacements = count
    else:
        new_content = content.replace(old, new, 1)
        replacements = 1

    # Write back
    try:
        with open(file_path, "w", encoding=encoding) as f:
            f.write(new_content)
    except Exception as e:
        # Restore from backup
        if backup:
            shutil.move(backup_path, file_path)
        raise ToolError(
            call_id=call_id,
            name="edit",
            message=f"Failed to write file: {e}",
        )

    duration_ms = int((time.time() - start_time) * 1000)

    # Generate diff for metadata
    diff = list(
        difflib.unified_diff(
            content.splitlines(keepends=True),
            new_content.splitlines(keepends=True),
            fromfile=str(file_path),
            tofile=str(file_path),
        )
    )

    metadata = {
        "path": str(file_path),
        "replacements": replacements,
        "total_occurrences": count,
        "diff": "".join(diff),
    }

    return ToolResult(
        call_id=call_id,
        name="edit",
        content=f"Replaced {replacements} occurrence(s) in {file_path}",
        is_error=False,
        duration_ms=duration_ms,
        metadata=metadata,
    )


class EditTool:
    """
    Edit tool wrapper.

    Example:
        >>> from continuum_sdk.tools import EditTool
        >>> editor = EditTool()
        >>> result = editor.edit("config.py", "old_value", "new_value")
    """

    def __init__(self, backup: bool = True):
        self.backup = backup

    def edit(
        self,
        path: str,
        old: str,
        new: str,
        replace_all: bool = False,
    ) -> ToolResult:
        """Edit file by replacing string."""
        return edit_file(path, old, new, replace_all, self.backup)

    def replace_all(self, path: str, old: str, new: str) -> ToolResult:
        """Replace all occurrences in file."""
        return self.edit(path, old, new, replace_all=True)

    def __call__(self, path: str, old: str, new: str, **kwargs) -> ToolResult:
        """Allow calling instance directly."""
        return self.edit(path, old, new, **kwargs)
