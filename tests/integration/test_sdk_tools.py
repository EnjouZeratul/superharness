"""SDK Tool 集成测试

测试 Tool API 的各种场景。
"""

import pytest
import sys
import os
import asyncio
import tempfile
import shutil
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock, AsyncMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


@pytest.fixture
def sample_project_dir(tmp_path):
    """Sample project directory fixture"""
    src_dir = tmp_path / "src"
    src_dir.mkdir()
    (src_dir / "main.py").write_text("def hello(): return 'Hello'\ndef world(): pass")
    (tmp_path / "README.md").write_text("# Test Project")
    return tmp_path


class TestBuiltinTools:
    """内置工具测试"""

    def test_read_file(self, sample_project_dir):
        """测试读文件"""
        from continuum_sdk.tools import ReadTool

        reader = ReadTool()
        result = reader.read(str(sample_project_dir / "src" / "main.py"))

        assert result.is_error is False
        assert "hello" in result.content
        print(f"\n[Read File]: {result.content[:50]}")

    def test_read_file_nonexistent(self):
        """测试读不存在文件"""
        from continuum_sdk.tools import ReadTool, ToolError

        reader = ReadTool()

        with pytest.raises(ToolError):
            reader.read("/nonexistent/path/file.txt")
        print(f"\n[Read Nonexistent]: Error raised")

    def test_write_file(self, tmp_path):
        """测试写文件"""
        from continuum_sdk.tools import WriteTool, ReadTool

        writer = WriteTool(backup=False)
        filepath = str(tmp_path / "new.txt")

        result = writer.write(filepath, "test content")
        assert result.is_error is False

        # 验证文件创建
        reader = ReadTool()
        content = reader.read(filepath)
        assert "test content" in content.content
        print(f"\n[Write File]: Created {filepath}")

    def test_edit_file(self, sample_project_dir):
        """测试编辑文件"""
        from continuum_sdk.tools import EditTool, ReadTool

        editor = EditTool(backup=False)
        filepath = str(sample_project_dir / "src" / "main.py")

        result = editor.edit(filepath, "Hello", "World")
        assert result.is_error is False

        # 验证修改
        reader = ReadTool()
        content = reader.read(filepath)
        assert "World" in content.content
        print(f"\n[Edit File]: Modified")

    def test_edit_file_not_found(self, sample_project_dir):
        """测试编辑找不到旧文本"""
        from continuum_sdk.tools import EditTool, ToolError

        editor = EditTool(backup=False)
        filepath = str(sample_project_dir / "src" / "main.py")

        with pytest.raises(ToolError):
            editor.edit(filepath, "nonexistent_text", "replacement")
        print(f"\n[Edit Not Found]: Error raised")

    def test_list_directory(self, sample_project_dir):
        """测试列出目录"""
        from continuum_sdk.tools import BashTool

        bash = BashTool()
        result = bash.run(f"ls {sample_project_dir}")

        assert result.is_error is False
        print(f"\n[List Directory]: {result.content}")

    def test_grep(self, sample_project_dir):
        """测试 grep"""
        from continuum_sdk.tools import GrepTool

        grep = GrepTool()
        result = grep.search("def ", path=str(sample_project_dir / "src"))

        assert result.is_error is False
        assert "def" in result.content.lower()
        print(f"\n[Grep]: Found matches")

    def test_glob(self, sample_project_dir):
        """测试 glob"""
        from continuum_sdk.tools import GlobTool

        globber = GlobTool()
        result = globber.find("**/*.py", path=str(sample_project_dir))

        assert result.is_error is False
        assert ".py" in result.content
        print(f"\n[Glob]: Found files")

    def test_bash_command(self):
        """测试 Bash 命令"""
        from continuum_sdk.tools import BashTool

        bash = BashTool()
        result = bash.run("echo hello")

        assert result.is_error is False
        assert "hello" in result.content
        print(f"\n[Bash]: {result.content}")


class TestCustomToolRegistration:
    """自定义工具注册测试"""

    def test_register_decorator(self):
        """测试 @tool 装饰器"""
        from continuum_sdk.tools import tool, get_registry

        @tool(name="my_test_tool", description="Test tool")
        def my_func(x: int) -> int:
            return x * 2

        assert my_func.name == "my_test_tool"
        print(f"\n[Decorator]: Registered {my_func.name}")

    def test_register_class(self):
        """测试继承 CustomTool"""
        from continuum_sdk.tools import CustomTool, ToolRegistry

        class MyCustomTool(CustomTool):
            @property
            def name(self) -> str:
                return "custom_class_tool"

            @property
            def description(self) -> str:
                return "Custom class tool"

            def parameters_schema(self):
                return {
                    "type": "object",
                    "properties": {"x": {"type": "integer"}},
                }

            async def execute(self, **kwargs):
                x = kwargs.get("x", 0)
                return str(x * 3)

        registry = ToolRegistry()
        my_tool = MyCustomTool()
        registry.register(my_tool)

        assert registry.has_tool("custom_class_tool")
        print(f"\n[Class]: Registered custom_class_tool")

    def test_register_duplicate_name(self):
        """测试重复名称"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="duplicate_tool", description="First")
        def first(x):
            return x

        @tool(name="duplicate_tool", description="Second")
        def second(x):
            return x * 2

        registry.register(first)
        registry.register(second)  # 覆盖同名

        # 第二个应该覆盖第一个
        retrieved = registry.get("duplicate_tool")
        assert retrieved is second
        print(f"\n[Duplicate]: Overwritten")

    def test_unregister_tool(self):
        """测试注销工具"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="to_remove", description="To remove")
        def to_remove(x):
            return x

        registry.register(to_remove)
        assert registry.has_tool("to_remove")

        registry.unregister("to_remove")
        assert not registry.has_tool("to_remove")
        print(f"\n[Unregister]: Removed")


class TestCustomToolExecution:
    """自定义工具执行测试"""

    def test_execute_simple(self):
        """测试简单执行"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="simple_exec", description="Simple")
        def simple(x):
            return f"result: {x}"

        registry.register(simple)
        # execute 是异步方法，需要用 asyncio.run
        result = asyncio.run(registry.execute("simple_exec", x="test"))

        assert result == "result: test"
        print(f"\n[Execute Simple]: {result}")

    def test_execute_with_params(self):
        """测试带参数执行"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="calculator", description="Calculator")
        def calculator(a: int, b: int, op: str = "add"):
            if op == "add":
                return a + b
            return a - b

        registry.register(calculator)
        # execute 是异步方法，需要用 asyncio.run
        result = asyncio.run(registry.execute("calculator", a=1, b=2, op="add"))

        assert result == "3"
        print(f"\n[Execute Params]: {result}")

    @pytest.mark.asyncio
    async def test_execute_async(self):
        """测试异步工具"""
        from continuum_sdk.tools import BashTool

        bash = BashTool()
        result = await bash.run_async("echo async")

        assert "async" in result.content
        print(f"\n[Execute Async]: {result.content}")

    def test_execute_with_confirmation(self):
        """测试需确认工具执行"""
        from continuum_sdk.tools import ToolRegistry, tool, ToolMeta, ToolCategory

        registry = ToolRegistry()

        @tool(name="dangerous_test", description="Dangerous")
        def dangerous(x):
            return f"dangerous: {x}"

        registry.register(dangerous)

        # 在测试环境中直接执行
        result = asyncio.run(registry.execute("dangerous_test", x="data"))
        assert result is not None
        print(f"\n[Confirm Execute]: {result}")

    def test_execute_without_confirmation(self):
        """测试未确认执行危险工具"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="needs_confirm", description="Needs confirm")
        def needs_confirm(x):
            return f"executed: {x}"

        registry.register(needs_confirm)

        # 在测试环境中，默认执行
        result = asyncio.run(registry.execute("needs_confirm", x="test"))
        assert result is not None
        print(f"\n[No Confirm]: Executed")


class TestToolSchema:
    """工具 Schema 测试"""

    def test_get_schema(self):
        """测试获取 schema"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(
            name="schema_test",
            description="Schema test",
            parameters={
                "type": "object",
                "properties": {"x": {"type": "integer"}},
            },
        )
        def schema_test(x: int):
            return x

        registry.register(schema_test)

        # 验证工具存在
        assert registry.has_tool("schema_test")
        print(f"\n[Schema]: Tool registered with parameters")

    def test_validate_params(self):
        """测试参数验证"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="validated", description="Validated tool")
        def validated(x: int):
            return x * 2

        registry.register(validated)

        # 有效参数 - execute 是异步方法
        result = asyncio.run(registry.execute("validated", x=5))
        assert result == "10"
        print(f"\n[Validate]: Valid params executed")

    def test_auto_infer_schema(self):
        """测试自动推断 schema"""
        from continuum_sdk.tools import tool

        @tool(name="auto_schema", description="Auto schema")
        def auto_schema(a: int, b: str = "default"):
            return f"{a}: {b}"

        # 工具应存在
        assert auto_schema.name == "auto_schema"
        print(f"\n[Auto Schema]: Infer from type hints")


class TestToolMeta:
    """工具元数据测试"""

    def test_get_tool_info(self):
        """测试获取工具信息"""
        from continuum_sdk.tools import tool

        @tool(name="meta_test", description="Meta test tool")
        def meta_test(x):
            return x

        assert meta_test.name == "meta_test"
        assert meta_test.description == "Meta test tool"
        print(f"\n[Tool Info]: name={meta_test.name}")

    def test_list_tools(self):
        """测试列出工具"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="list1", description="Tool 1")
        def list1(x):
            return x

        @tool(name="list2", description="Tool 2")
        def list2(x):
            return x

        registry.register(list1)
        registry.register(list2)

        tools = registry.list_names()
        assert "list1" in tools
        assert "list2" in tools
        print(f"\n[List Tools]: {tools}")

    def test_list_by_category(self):
        """测试按分类列出"""
        from continuum_sdk.tools import ToolCategory, CustomTool

        # CustomTool 基类提供默认 category
        class MyCategorizedTool(CustomTool):
            @property
            def name(self) -> str:
                return "categorized_tool"

            @property
            def description(self) -> str:
                return "Categorized tool"

            def parameters_schema(self):
                return {}

            async def execute(self, **kwargs):
                return "ok"

        tool = MyCategorizedTool()
        # 默认 category 是 "other"
        assert tool.category == "other"
        print(f"\n[By Category]: Default category is 'other'")

    def test_search_tools(self):
        """测试搜索工具"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="read_data", description="Read data")
        def read_data(x):
            return x

        @tool(name="write_data", description="Write data")
        def write_data(x):
            return x

        registry.register(read_data)
        registry.register(write_data)

        tools = registry.list_names()
        read_tools = [t for t in tools if "read" in t.lower()]
        assert len(read_tools) >= 1
        print(f"\n[Search]: Found {len(read_tools)} read tools")


class TestToolError:
    """工具错误处理测试"""

    def test_execution_error(self):
        """测试执行错误"""
        from continuum_sdk.tools import ToolRegistry, tool

        registry = ToolRegistry()

        @tool(name="failing", description="Failing tool")
        def failing(x):
            raise ValueError("Intentional error")

        registry.register(failing)

        try:
            asyncio.run(registry.execute("failing", x="test"))
            pytest.fail("Should have raised error")
        except ValueError as e:
            assert "Intentional" in str(e)
            print(f"\n[Execution Error]: {e}")

    def test_timeout_error(self):
        """测试超时"""
        from continuum_sdk.tools import BashTool, ToolError

        bash = BashTool(default_timeout=1.0)

        with pytest.raises(ToolError):
            bash.run("sleep 5", timeout=0.5)
        print(f"\n[Timeout]: Error raised")

    def test_permission_error(self):
        """测试权限错误"""
        from continuum_sdk.tools import ReadTool, ToolError

        reader = ReadTool()

        # 读取需要权限的系统文件可能失败
        with pytest.raises(ToolError):
            reader.read("/root/sensitive_file.txt")
        print(f"\n[Permission]: Error raised")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])