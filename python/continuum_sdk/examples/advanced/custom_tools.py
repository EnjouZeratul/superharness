"""自定义工具示例

展示如何创建和注册自定义工具。
"""

import asyncio

from continuum_sdk.tools import (
    CustomTool,
    get_registry,
    register_tool,
    tool,
)

# ==================== 方式 1: 继承 CustomTool ====================

class CalculatorTool(CustomTool):
    """计算器工具"""

    @property
    def name(self) -> str:
        return "calculator"

    @property
    def description(self) -> str:
        return "执行基本数学运算"

    def parameters_schema(self):
        return {
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "运算类型"
                },
                "a": {"type": "number", "description": "第一个数"},
                "b": {"type": "number", "description": "第二个数"}
            },
            "required": ["operation", "a", "b"]
        }

    @property
    def category(self) -> str:
        return "math"

    async def execute(self, **kwargs) -> str:
        op = kwargs["operation"]
        a, b = kwargs["a"], kwargs["b"]

        ops = {
            "add": lambda x, y: x + y,
            "subtract": lambda x, y: x - y,
            "multiply": lambda x, y: x * y,
            "divide": lambda x, y: x / y if y != 0 else "Error: division by zero"
        }

        result = ops[op](a, b)
        return f"{a} {op} {b} = {result}"


# ==================== 方式 2: 使用 @tool 装饰器 ====================

@tool(
    name="greet",
    description="生成问候语",
    requires_confirmation=False
)
async def greet_user(name: str, greeting: str = "Hello") -> str:
    """生成个性化问候语"""
    return f"{greeting}, {name}!"


@tool(
    name="format_json",
    description="格式化 JSON 字符串"
)
async def format_json_string(data: str, indent: int = 2) -> str:
    """格式化 JSON"""
    import json
    try:
        parsed = json.loads(data)
        return json.dumps(parsed, indent=indent, ensure_ascii=False)
    except json.JSONDecodeError as e:
        return f"Error: {e}"


# ==================== 方式 3: 危险工具（需要确认） ====================

@tool(
    name="delete_temp_files",
    description="删除临时文件",
    is_dangerous=True,
    requires_confirmation=True
)
async def delete_temp_files(pattern: str) -> str:
    """删除匹配模式的临时文件"""
    # 实际实现需要文件操作
    return f"[模拟] 已删除匹配 '{pattern}' 的临时文件"


# ==================== 演示 ====================

async def main():
    print("=== 自定义工具示例 ===\n")

    # 获取注册表
    registry = get_registry()

    # 1. 注册工具
    print("1. 注册工具")
    register_tool(CalculatorTool())  # 使用默认注册表
    registry.register(greet_user)     # 装饰器已经创建了实例
    registry.register(format_json_string)
    registry.register(delete_temp_files)
    print(f"   已注册 {len(registry.list_names())} 个工具\n")

    # 2. 列出所有工具
    print("2. 已注册工具列表:")
    for t in registry.list():
        danger_flag = " [危险]" if t.is_dangerous else ""
        confirm_flag = " [需确认]" if t.requires_confirmation else ""
        print(f"   - {t.name}: {t.description}{danger_flag}{confirm_flag}")
    print()

    # 3. 执行工具
    print("3. 执行工具")

    # 计算器
    result = await registry.execute("calculator", operation="add", a=10, b=5)
    print(f"   calculator: {result}")

    # 问候
    result = await registry.execute("greet", name="World", greeting="Hi")
    print(f"   greet: {result}")

    # JSON 格式化
    result = await registry.execute(
        "format_json",
        data='{"name":"test","value":123}'
    )
    print(f"   format_json:\n{result}")

    # 4. 获取工具元数据
    print("\n4. 工具元数据:")
    meta = registry.get_meta("calculator")
    if meta:
        print(f"   名称: {meta['name']}")
        print(f"   描述: {meta['description']}")
        print(f"   分类: {meta['category']}")
        print(f"   参数: {meta['parameters']['required']}")


if __name__ == "__main__":
    asyncio.run(main())
