"""Tools 单元测试"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
from superharness_sdk.tools import (
    BuiltinTools, ToolMeta, ToolCategory, ToolResult,
    CustomTool, ToolRegistry, tool, register_tool, get_registry
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
        assert meta.requires_confirmation == False
        assert meta.is_dangerous == False

    def test_tool_meta_with_params(self):
        """测试带参数的工具元数据"""
        meta = ToolMeta(
            name="dangerous_tool",
            description="Dangerous",
            category=ToolCategory.SHELL,
            requires_confirmation=True,
            is_dangerous=True
        )
        assert meta.requires_confirmation == True
        assert meta.is_dangerous == True


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
        assert result.is_error == False

    def test_tool_result_error(self):
        """测试错误结果"""
        result = ToolResult(
            call_id="call-456",
            name="fail_tool",
            content="Error message",
            is_error=True
        )
        assert result.is_error == True


class TestBuiltinTools:
    """BuiltinTools 测试"""

    def test_builtin_tools_creation(self):
        """测试内置工具集创建"""
        tools = BuiltinTools()
        assert tools is not None

    def test_list_tools(self):
        """测试列出工具"""
        tools = BuiltinTools()
        tool_list = tools.list_tools()
        assert len(tool_list) > 0
        assert any(t.name == "read_file" for t in tool_list)

    def test_get_tool_meta(self):
        """测试获取工具元数据"""
        tools = BuiltinTools()
        meta = tools.get_tool_meta("read_file")
        assert meta is not None
        assert meta.name == "read_file"

    def test_is_available(self):
        """测试工具可用性检查"""
        tools = BuiltinTools()
        assert tools.is_available("read_file") == True
        assert tools.is_available("nonexistent_tool") == False

    def test_read_file_not_implemented(self):
        """测试未实现的读取文件"""
        tools = BuiltinTools()
        with pytest.raises(NotImplementedError):
            tools.read_file("test.txt")

    def test_write_file_not_implemented(self):
        """测试未实现的写入文件"""
        tools = BuiltinTools()
        with pytest.raises(NotImplementedError):
            tools.write_file("test.txt", "content")


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
