"""自定义工具 API

用于创建和注册自定义工具。
"""

from typing import Any, Callable, Dict, List, Optional, get_type_hints
from abc import ABC, abstractmethod
import inspect
import asyncio


class CustomTool(ABC):
    """自定义工具基类

    Usage:
        from continuum_sdk.tools import CustomTool

        class MyTool(CustomTool):
            @property
            def name(self) -> str:
                return "my_tool"

            @property
            def description(self) -> str:
                return "My custom tool"

            def parameters_schema(self) -> Dict[str, Any]:
                return {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    },
                    "required": ["input"]
                }

            async def execute(self, **kwargs) -> str:
                return f"Processed: {kwargs['input']}"

        # 注册
        registry.register(MyTool())
    """

    @property
    @abstractmethod
    def name(self) -> str:
        """工具名称"""
        ...

    @property
    @abstractmethod
    def description(self) -> str:
        """工具描述"""
        ...

    @abstractmethod
    def parameters_schema(self) -> Dict[str, Any]:
        """参数 JSON Schema"""
        ...

    @abstractmethod
    async def execute(self, **kwargs) -> str:
        """执行工具"""
        ...

    @property
    def category(self) -> str:
        """工具分类"""
        return "other"

    @property
    def requires_confirmation(self) -> bool:
        """是否需要用户确认"""
        return False

    @property
    def is_dangerous(self) -> bool:
        """是否为危险操作"""
        return False

    def to_meta(self) -> Dict[str, Any]:
        """转换为元数据字典"""
        return {
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters_schema(),
            "category": self.category,
            "requires_confirmation": self.requires_confirmation,
            "is_dangerous": self.is_dangerous,
        }


def tool(
    name: str,
    description: str,
    parameters: Optional[Dict[str, Any]] = None,
    requires_confirmation: bool = False,
    is_dangerous: bool = False
) -> Callable:
    """工具装饰器

    Args:
        name: 工具名称
        description: 工具描述
        parameters: 参数 Schema（可选，自动推断）
        requires_confirmation: 是否需要确认
        is_dangerous: 是否危险

    Usage:
        from continuum_sdk.tools import tool

        @tool(name="add", description="Add two numbers")
        async def add(a: int, b: int) -> int:
            return a + b

        @tool(name="greet", description="Say hello")
        async def greet(name: str, greeting: str = "Hello") -> str:
            return f"{greeting}, {name}!"
    """
    def decorator(func: Callable) -> CustomTool:
        # 自动推断参数 Schema
        inferred_params = parameters
        if inferred_params is None:
            hints = get_type_hints(func)
            sig = inspect.signature(func)

            properties = {}
            required = []

            for param_name, param in sig.parameters.items():
                if param_name == 'self':
                    continue

                param_type = hints.get(param_name, str)
                prop = {"type": "string"}

                if param_type == int:
                    prop = {"type": "integer"}
                elif param_type == float:
                    prop = {"type": "number"}
                elif param_type == bool:
                    prop = {"type": "boolean"}
                elif param_type == list:
                    prop = {"type": "array"}
                elif param_type == dict:
                    prop = {"type": "object"}

                properties[param_name] = prop

                if param.default == inspect.Parameter.empty:
                    required.append(param_name)

            inferred_params = {
                "type": "object",
                "properties": properties,
                "required": required,
            }

        # 创建动态类
        class DecoratedTool(CustomTool):
            @property
            def name(self) -> str:
                return name

            @property
            def description(self) -> str:
                return description

            def parameters_schema(self) -> Dict[str, Any]:
                return inferred_params

            @property
            def requires_confirmation(self) -> bool:
                return requires_confirmation

            @property
            def is_dangerous(self) -> bool:
                return is_dangerous

            async def execute(self, **kwargs) -> str:
                result = func(**kwargs)
                if asyncio.iscoroutine(result):
                    result = await result
                return str(result)

        return DecoratedTool()

    return decorator


class ToolRegistry:
    """工具注册表

    Usage:
        from continuum_sdk.tools import ToolRegistry, CustomTool

        registry = ToolRegistry()

        # 注册自定义工具
        registry.register(MyTool())

        # 列出所有工具
        tools = registry.list()

        # 执行工具
        result = await registry.execute("my_tool", input="test")
    """

    def __init__(self):
        """初始化注册表"""
        self._tools: Dict[str, CustomTool] = {}

    def register(self, tool: CustomTool) -> None:
        """注册工具

        Args:
            tool: 自定义工具实例
        """
        self._tools[tool.name] = tool

    def unregister(self, name: str) -> bool:
        """注销工具"""
        if name in self._tools:
            del self._tools[name]
            return True
        return False

    def get(self, name: str) -> Optional[CustomTool]:
        """获取工具"""
        return self._tools.get(name)

    def list(self) -> List[CustomTool]:
        """列出所有工具"""
        return list(self._tools.values())

    def list_names(self) -> List[str]:
        """列出所有工具名称"""
        return list(self._tools.keys())

    async def execute(self, name: str, **kwargs) -> str:
        """执行工具

        Args:
            name: 工具名称
            **kwargs: 工具参数

        Returns:
            执行结果
        """
        tool = self.get(name)
        if tool is None:
            raise ValueError(f"Tool not found: {name}")
        return await tool.execute(**kwargs)

    def has_tool(self, name: str) -> bool:
        """检查工具是否存在"""
        return name in self._tools

    def get_meta(self, name: str) -> Optional[Dict[str, Any]]:
        """获取工具元数据"""
        tool = self.get(name)
        return tool.to_meta() if tool else None


# 默认注册表实例
default_registry = ToolRegistry()


def register_tool(tool: CustomTool) -> None:
    """注册工具到默认注册表"""
    default_registry.register(tool)


def get_registry() -> ToolRegistry:
    """获取默认注册表"""
    return default_registry
