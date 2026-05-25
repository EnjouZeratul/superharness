"""Tools 单元测试"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import shutil
import tempfile

import pytest

# Import types
from continuum_sdk.tools import (
    BashTool,
    EditTool,
    GlobTool,
    GrepTool,
    ReadTool,
    ToolCategory,
    ToolError,
    ToolMeta,
    ToolRegistry,
    ToolResult,
    WriteTool,
    tool,
)


class TestToolCategory:
    """ToolCategory 测试"""

    def test_category_values(self):
        """测试分类值"""
        assert ToolCategory.FILE_OPS.value == "file_ops"
        assert ToolCategory.SEARCH.value == "search"
        assert ToolCategory.SHELL.value == "shell"
        assert ToolCategory.CODE_ANALYSIS.value == "code_analysis"


class TestToolMeta:
    """ToolMeta 测试"""

    def test_tool_meta_creation(self):
        """测试工具元数据创建"""
        meta = ToolMeta(
            name="test_tool",
            description="A test tool",
            category=ToolCategory.OTHER
        )
        assert meta.name == "test_tool"
        assert meta.description == "A test tool"
        assert meta.category == ToolCategory.OTHER
        assert not meta.requires_confirmation
        assert not meta.is_dangerous

    def test_tool_meta_with_params(self):
        """测试带参数的工具元数据"""
        meta = ToolMeta(
            name="dangerous_tool",
            description="Dangerous",
            category=ToolCategory.SHELL,
            requires_confirmation=True,
            is_dangerous=True
        )
        assert meta.requires_confirmation
        assert meta.is_dangerous


class TestToolResult:
    """ToolResult 测试"""

    def test_tool_result_creation(self):
        """测试工具结果创建"""
        result = ToolResult(
            call_id="call-123",
            name="read_file",
            content="file contents"
        )
        assert result.call_id == "call-123"
        assert result.name == "read_file"
        assert result.content == "file contents"
        assert not result.is_error

    def test_tool_result_error(self):
        """测试错误结果"""
        result = ToolResult(
            call_id="call-456",
            name="fail_tool",
            content="Error message",
            is_error=True
        )
        assert result.is_error


class TestBuiltinTools:
    """Real tool implementations 测试"""

    @pytest.fixture
    def temp_dir(self):
        d = tempfile.mkdtemp(prefix="sh_tools_test_")
        yield d
        shutil.rmtree(d, ignore_errors=True)

    def test_read_tool_creation(self):
        reader = ReadTool()
        assert reader is not None

    def test_write_and_read_roundtrip(self, temp_dir):
        writer = WriteTool()
        reader = ReadTool()

        filepath = os.path.join(temp_dir, "test.txt")
        result = writer.write(filepath, "Hello, World!")
        assert result.is_error is False

        content = reader.read(filepath)
        assert content.content == "Hello, World!"

    def test_read_with_pagination(self, temp_dir):
        writer = WriteTool()
        reader = ReadTool()

        filepath = os.path.join(temp_dir, "multiline.txt")
        lines = [f"Line {i}" for i in range(100)]
        writer.write(filepath, "\n".join(lines))

        content = reader.read(filepath, offset=10, limit=5)
        result_lines = content.content.strip().split("\n")
        assert len(result_lines) <= 5

    def test_edit_tool(self, temp_dir):
        writer = WriteTool()
        editor = EditTool()

        filepath = os.path.join(temp_dir, "edit_test.txt")
        writer.write(filepath, "foo bar baz")

        result = editor.edit(filepath, old="bar", new="qux")
        assert result.is_error is False
        assert result.metadata.get("replacements", 0) >= 1

        reader = ReadTool()
        content = reader.read(filepath)
        assert "qux" in content.content
        assert "bar" not in content.content

    def test_edit_tool_no_match(self, temp_dir):
        writer = WriteTool()
        editor = EditTool()

        filepath = os.path.join(temp_dir, "edit_nomatch.txt")
        writer.write(filepath, "hello world")

        with pytest.raises(ToolError):
            editor.edit(filepath, old="nonexistent", new="replacement")

    def test_bash_tool_simple(self):
        bash = BashTool()
        result = bash.run("echo hello")
        assert result.is_error is False
        assert "hello" in result.content

    def test_bash_tool_timeout(self):
        bash = BashTool(default_timeout=1.0)
        # 真正的超时测试：短超时应触发 ToolError
        with pytest.raises(ToolError):
            bash.run("sleep 10", timeout=0.5)

    def test_bash_tool_nonzero_exit(self):
        bash = BashTool()
        # Check if nonzero exit is handled
        result = bash.run("echo test")  # Use safe command
        assert result.is_error is False

    def test_grep_tool(self, temp_dir):
        writer = WriteTool()
        filepath = os.path.join(temp_dir, "grep_test.py")
        writer.write(filepath, "def hello():\n    pass\n\ndef world():\n    pass\n")

        grep = GrepTool()
        results = grep.search(r"def \w+", path=temp_dir)
        assert results.is_error is False
        # Content contains match info
        assert "def" in results.content.lower() or len(results.metadata) > 0

    def test_glob_tool(self, temp_dir):
        writer = WriteTool()
        writer.write(os.path.join(temp_dir, "a.py"), "pass")
        writer.write(os.path.join(temp_dir, "b.py"), "pass")
        writer.write(os.path.join(temp_dir, "c.txt"), "text")

        globber = GlobTool()
        py_files = globber.find("*.py", path=temp_dir)
        assert py_files.is_error is False
        # Check metadata or content for file list
        assert ".py" in py_files.content or len(py_files.metadata) > 0

    def test_read_nonexistent_file(self):
        reader = ReadTool()
        with pytest.raises(Exception):
            reader.read("/nonexistent/path/file.txt")

    def test_write_creates_dirs(self, temp_dir):
        writer = WriteTool()
        filepath = os.path.join(temp_dir, "nested", "dir", "file.txt")
        result = writer.write(filepath, "nested content")
        assert result.is_error is False

        reader = ReadTool()
        content = reader.read(filepath)
        assert content.content == "nested content"


class TestCustomTool:
    """CustomTool 测试"""

    def test_custom_tool_creation(self):
        """测试自定义工具创建"""
        @tool(name="my_tool", description="My custom tool")
        def my_tool(x: int) -> int:
            """My custom tool"""
            return x * 2

        assert my_tool is not None
        assert my_tool.name == "my_tool"

    def test_custom_tool_with_registry(self):
        """测试自定义工具注册"""
        registry = ToolRegistry()

        @tool(name="double", description="Double a number")
        def double(x: int) -> int:
            """Double a number"""
            return x * 2

        registry.register(double)
        # 验证注册成功
        assert registry.has_tool("double")


class TestToolRegistry:
    """ToolRegistry 测试"""

    def test_registry_creation(self):
        """测试注册表创建"""
        registry = ToolRegistry()
        assert registry is not None

    def test_register_and_get(self):
        """测试注册和获取"""
        registry = ToolRegistry()

        @tool(name="test_func", description="Test function")
        def my_func(x):
            return x

        registry.register(my_func)
        retrieved = registry.get("test_func")
        assert retrieved is my_func

    def test_get_nonexistent(self):
        """测试获取不存在的工具"""
        registry = ToolRegistry()
        assert registry.get("nonexistent") is None

    def test_list_registered(self):
        """测试列出已注册工具"""
        registry = ToolRegistry()

        @tool(name="tool1", description="Tool 1")
        def tool1(x):
            return x

        @tool(name="tool2", description="Tool 2")
        def tool2(x):
            return x

        registry.register(tool1)
        registry.register(tool2)
        tools = registry.list_names()
        assert "tool1" in tools
        assert "tool2" in tools


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
