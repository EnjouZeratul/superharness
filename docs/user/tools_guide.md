# Continuum 工具使用指南

> 版本: v1.0.0+
> 更新时间: 2026-05-21

---

## 目录

1. [工具概述](#工具概述)
2. [文件操作工具](#文件操作工具)
3. [搜索工具](#搜索工具)
4. [Shell 工具](#shell-工具)
5. [代码分析工具](#代码分析工具)
6. [记忆工具](#记忆工具)
7. [自定义工具](#自定义工具)
8. [工具配置](#工具配置)

---

## 工具概述

Continuum 提供丰富的内置工具，让 Agent 能够执行各种任务。

### 工具分类

| 分类 | 工具 | 用途 |
|------|------|------|
| 文件操作 | read, write, edit, list | 文件读写和管理 |
| 搜索 | grep, glob | 内容和文件搜索 |
| Shell | bash | 执行命令 |
| 代码分析 | go_to_definition, find_references, hover | LSP 功能 |
| 记忆 | save_memory, query_memory | 信息存储和检索 |
| 工作流 | create_checkpoint, restore_checkpoint | 状态管理 |

### 工具使用流程

```
用户请求 → Agent 分析 → 选择工具 → 执行 → 返回结果
                ↓
        工具调用决策
```

---

## 文件操作工具

### read_file - 读取文件

**功能**: 安全读取文件内容，支持分页和编码处理。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| path | string | 是 | 文件路径 |
| offset | int | 否 | 起始行号（默认0） |
| limit | int 否 | 读取行数（默认2000） |

**示例**:

```python
from continuum_sdk.tools import BuiltinTools

tools = BuiltinTools()

# 读取完整文件
content = tools.read_file("src/main.py")

# 读取前 100 行
content = tools.read_file("src/main.py", limit=100)

# 读取第 50-100 行
content = tools.read_file("src/main.py", offset=50, limit=50)
```

**安全特性**:
- 自动检测文件大小，大文件提示分页
- 支持多种编码（UTF-8, GBK, Latin-1）
- 禁止访问项目目录外的文件（可配置）

### write_file - 写入文件

**功能**: 安全写入文件内容。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| path | string | 是 | 文件路径 |
| content | string | 是 | 写入内容 |

**示例**:

```python
# 写入文件
tools.write_file("output.txt", "Hello, World!")

# 覆盖已存在文件
tools.write_file("config.json", json.dumps(config, indent=2))
```

**安全特性**:
- 原子写入（temp file + rename）
- 写入前自动备份
- 需要用户确认（可配置跳过）

### edit_file - 编辑文件

**功能**: 精确查找替换文件内容。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| path | string | 是 | 文件路径 |
| old | string | 是 | 要替换的文本 |
| new | string | 是 | 新文本 |

**示例**:

```python
# 替换函数名
success = tools.edit_file(
    "src/main.py",
    old="def old_name():",
    new="def new_name():"
)

# 替换所有匹配（需要 replace_all=True）
success = tools.edit_file(
    "src/utils.py",
    old="import old_module",
    new="import new_module"
)
```

**安全特性**:
- 精确匹配，避免意外替换
- 替换前预览变更
- 失败时不修改原文件

### list_directory - 列出目录

**功能**: 列出目录内容。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| path | string | 是 | 目录路径 |

**返回**: 包含 name 和 type 的字典列表。

**示例**:

```python
entries = tools.list_directory("src")
for entry in entries:
    print(f"{entry['name']} - {entry['type']}")
# 输出:
# main.py - file
# utils.py - file
# lib - dir
```

---

## 搜索工具

### grep - 内容搜索

**功能**: 使用正则表达式搜索文件内容。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| pattern | string | 是 | 正则表达式 |
| path | string | 否 | 搜索路径（默认当前目录） |
| glob | string | 否 | 文件过滤（如 "*.py"） |

**示例**:

```python
# 搜索函数定义
results = tools.grep(r"def \w+\(", path="src/", glob="*.py")

# 搜索 TODO 注释
results = tools.grep(r"TODO.*", path=".")

for r in results:
    print(f"{r['file']}:{r['line']}: {r['content']}")
```

### glob - 文件匹配

**功能**: 使用 glob 模式查找文件。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| pattern | string | 是 | glob 模式 |
| path | string | 否 | 搜索路径 |

**示例**:

```python
# 查找所有 Python 文件
files = tools.glob("**/*.py")

# 查找测试文件
files = tools.glob("tests/**/test_*.py")

# 查找 Markdown 文档
files = tools.glob("docs/**/*.md")
```

**常用模式**:

| 模式 | 说明 |
|------|------|
| `*.py` | 当前目录所有 .py 文件 |
| `**/*.py` | 递归所有 .py 文件 |
| `src/**/*.rs` | src 目录下所有 Rust 文件 |
| `test_*.py` | 以 test_ 开头的文件 |

---

## Shell 工具

### bash - 执行命令

**功能**: 安全执行 shell 命令。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| command | string | 是 | bash 命令 |
| timeout | int | 否 | 超时毫秒（默认120000） |
| working_dir | string | 否 | 工作目录 |

**示例**:

```python
# 执行简单命令
result = tools.bash("echo hello")
print(result.content)  # "hello"

# 执行带超时的命令
result = tools.bash("npm install", timeout=300000)

# 在特定目录执行
result = tools.bash("git status", working_dir="/path/to/repo")
```

**安全特性**:
- 命令白名单检查
- 超时控制防止挂起
- 危险命令需要确认（rm -rf, sudo 等）

**常见用例**:

```python
# Git 操作
tools.bash("git status")
tools.bash("git diff HEAD~1")
tools.bash("git log --oneline -10")

# 运行测试
tools.bash("pytest tests/ -v")
tools.bash("cargo test --lib")

# 构建项目
tools.bash("pip install -r requirements.txt")
tools.bash("cargo build --release")
```

---

## 代码分析工具

### go_to_definition - 跳转定义

**功能**: 查找符号定义位置（需要 LSP 支持）。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| file | string | 是 | 文件路径 |
| line | int | 是 | 行号（1-based） |
| column | int | 是 | 列号（1-based） |

**示例**:

```python
# 查找函数定义
result = tools.go_to_definition("src/main.py", line=10, column=5)
if result:
    print(f"定义位置: {result['file']}:{result['line']}")
```

### find_references - 查找引用

**功能**: 查找符号的所有引用位置。

**参数**: 同 go_to_definition

**示例**:

```python
# 查找变量引用
refs = tools.find_references("src/main.py", line=15, column=10)
for ref in refs:
    print(f"{ref['file']}:{ref['line']}:{ref['column']}")
```

### hover - 获取类型信息

**功能**: 获取符号的类型信息和文档。

**示例**:

```python
# 获取变量类型
info = tools.hover("src/main.py", line=20, column=8)
print(info)  # "str" 或函数签名等
```

---

## 记忆工具

### save_memory - 保存记忆

**功能**: 存储重要信息供后续使用。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| key | string | 是 | 记忆键名 |
| value | any | 是 | 记忆值 |
| tier | string | 否 | 存储层级 |

**存储层级**:

| 层级 | 说明 | 持久性 |
|------|------|--------|
| working | 工作记忆 | 当前任务 |
| session | 会话记忆 | 当前会话 |
| project | 项目记忆 | 项目级别 |
| longterm | 长期记忆 | 永久存储 |

**示例**:

```python
# 保存项目配置
tools.save_memory("project_name", "MyProject", tier="project")

# 保存用户偏好
tools.save_memory("preferred_style", "Google Style", tier="longterm")

# 临时数据
tools.save_memory("current_file", "main.py", tier="working")
```

### query_memory - 查询记忆

**功能**: 检索已存储的信息。

**参数**:

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| query | string | 是 | 搜索关键词 |

**示例**:

```python
# 查询记忆
results = tools.query_memory("project")
for r in results:
    print(f"{r['key']}: {r['value']}")
```

---

## 自定义工具

### 创建自定义工具

**方式一：装饰器**

```python
from continuum_sdk.tools import tool, get_registry

@tool(name="weather", description="查询天气")
async def get_weather(city: str, unit: str = "celsius") -> dict:
    """获取指定城市的天气信息"""
    # 实现天气查询逻辑
    return {
        "city": city,
        "temperature": 25,
        "unit": unit,
        "condition": "sunny"
    }

# 注册工具
registry = get_registry()
registry.register(get_weather)
```

**方式二：类继承**

```python
from continuum_sdk.tools import CustomTool

class DatabaseTool(CustomTool):
    @property
    def name(self) -> str:
        return "query_db"
    
    @property
    def description(self) -> str:
        return "执行数据库查询"
    
    def parameters_schema(self) -> dict:
        return {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "SQL 查询语句"
                }
            },
            "required": ["query"]
        }
    
    async def execute(self, query: str) -> list:
        # 实现数据库查询
        return [{"id": 1, "name": "example"}]
```

### 工具参数定义

使用 JSON Schema 定义参数：

```python
@tool(name="search_web", description="搜索网页")
async def search_web(
    query: str,
    max_results: int = 10,
    filters: dict = None
) -> list:
    """搜索网页内容
    
    Args:
        query: 搜索关键词
        max_results: 最大结果数
        filters: 过滤条件
    
    Returns:
        搜索结果列表
    """
    pass
```

---

## 工具配置

### 启用/禁用工具

```python
from continuum_sdk import Agent, Config

# 只启用特定工具
config = Config(
    enabled_tools=["read_file", "write_file", "bash"]
)

# 禁用特定工具
config = Config(
    disabled_tools=["bash"]
)

agent = Agent(config=config)
```

### 工具权限配置

```toml
# ~/.sh/config.toml

[tools]
# 全局启用
enabled = true

# 工具特定配置
[tools.bash]
require_confirmation = true
allowed_commands = ["git", "npm", "cargo", "pytest"]
blocked_commands = ["rm -rf", "sudo"]

[tools.write_file]
require_confirmation = true
backup_before_write = true

[tools.read_file]
max_file_size = 10485760  # 10MB
```

### 工具执行超时

```python
config = Config(
    tool_timeout=60000,  # 60 秒
    bash_timeout=120000  # Bash 特定 120 秒
)
```

---

## 最佳实践

### 1. 组合使用工具

```python
# 分析项目结构
files = tools.glob("**/*.py")
for f in files:
    content = tools.read_file(f)
    todos = tools.grep(r"TODO", path=f)
    # 处理结果
```

### 2. 错误处理

```python
try:
    result = tools.bash("risky_command")
except TimeoutError:
    print("命令超时")
except PermissionError:
    print("权限不足")
```

### 3. 批量操作

```python
# 批量读取文件
files = tools.glob("src/**/*.py")
contents = {f: tools.read_file(f) for f in files}
```

---

*Continuum - 让工具触手可及*
