# Terminal 2 任务清单 - Python SDK 接口层

> 分配时间: 2026-05-24
> 负责范围: Python SDK 接口层、高层功能
> 产品定位: 混合包（Python接口 + Rust核心），对标LangChain/LangGraph
> 前置依赖: T1完成Rust工具实现 + PyO3 binding
> 等待机制: 收到T0通知或检测T1_TASKS.md任务状态为[x]后方可开始依赖任务

---

## 零、依赖等待规则

```
┌─────────────────────────────────────────────────────────────────────┐
│  ⛔ 依赖未完成时禁止行为                                             │
├─────────────────────────────────────────────────────────────────────┤
│  ❌ 假设T1已完成，直接编写调用代码                                    │
│  ❌ 使用mock/stub模拟binding接口                                     │
│  ❌ 自行定义binding接口结构                                          │
│  ❌ "先写框架，等T1完成后填充"                                        │
│                                                                     │
│  ✅ 正确等待方式                                                    │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 收到T0明确通知"T1任务X已完成"                                    │
│  ✅ 检测T1_TASKS.md中对应任务已标记[x]                               │
│  ✅ 实际导入binding成功后才编写调用代码                              │
│  ✅ 等待期间可做无依赖任务（如任务2.3、2.6）                          │
└─────────────────────────────────────────────────────────────────────┘
```

**依赖任务对照表：**
| 任务 | 依赖T1任务 | 状态检测位置 |
|------|-----------|-------------|
| 2.1 工具接口层 | 1.3 PyO3 Binding | T1_TASKS.md 任务1.3 |
| 2.2 Binding验证 | 1.1-1.3 全部 | T1_TASKS.md 验收清单 |
| 2.4 检查点接口 | 1.3 CheckpointWriter | T1_TASKS.md 任务1.3 |
| 2.5 SDK打包 | 1.6 maturin构建 | T1_TASKS.md 任务1.6 |

---

## 一、产品定位

```
┌─────────────────────────────────────────────────────────────────┐
│  continuum-sdk（pip install）                                    │
├─────────────────────────────────────────────────────────────────┤
│  continuum_sdk/          # Python接口层（T1负责）                │
│  ├── agent/              # Agent API                            │
│  ├── tools/              # 工具接口（调用Rust binding）          │
│  └── workflow/           # 工作流API                            │
│                                                                 │
│  _continuum.*.pyd        # Rust核心（T2提供，编译进wheel）       │
└─────────────────────────────────────────────────────────────────┘
```

**T1职责：**
- Python接口定义和文档
- 调用Rust binding（非重复实现）
- 高层功能（任务自检、工作流等）
- 验证binding正确工作

---

## 二、任务清单

### 任务 2.1: 工具接口层 - 调用Rust Binding

**文件**: `python/continuum_sdk/tools/builtin.py`

**当前问题**: 全部 `NotImplementedError("waiting for sh-core binding")`

**实现内容**:
- [ ] 导入并调用Rust binding中的工具
- [ ] Python类型注解和文档
- [ ] 错误处理和转换

**代码示例**:
```python
from _continuum import ToolExecutor  # Rust binding

class BuiltinTools:
    def __init__(self):
        self._executor = ToolExecutor()  # Rust实现
    
    def read_file(self, path: str, offset: int = None, limit: int = None) -> str:
        """读取文件内容"""
        return self._executor.read_file(path, offset, limit)
    
    def bash(self, command: str, timeout: int = None) -> ToolResult:
        """执行Shell命令"""
        return self._executor.bash(command, timeout)
```

**验收标准**:
```python
tools = BuiltinTools()
content = tools.read_file("test.txt")  # 调用Rust实现
result = tools.bash("echo hello")       # 调用Rust实现
assert "hello" in result.content
```

**依赖**: T2完成 `ToolExecutor` binding导出

---

### 任务 2.2: Binding集成验证

**目标**: 确认Python能正确调用所有Rust工具

**验证内容**:
- [ ] read_file / write_file / edit_file
- [ ] bash命令执行
- [ ] grep / glob搜索
- [ ] 错误处理正确传递

**验收标准**:
```python
# 集成测试
tools = BuiltinTools()

# 文件操作
tools.write_file("test.txt", "hello world")
assert tools.read_file("test.txt") == "hello world"
assert tools.edit_file("test.txt", "world", "python") == 1

# Shell执行
result = tools.bash("echo test")
assert result.is_error == False

# 搜索
matches = tools.grep("test", path=".")
assert len(matches) > 0
```

**依赖**: T2完成所有工具binding

---

### 任务 2.3: 任务自检机制（高层功能）

**文件**: `python/continuum_sdk/agent/task_completion.py`（新建）

**设计目标**: 判断任务是否完成，支持连续对话

**实现内容**:
- [ ] `TaskCompletionDetector` 类
- [ ] LLM辅助判断任务完成
- [ ] `TASK_COMPLETED` / `USER_INTERRUPTED` 标记
- [ ] 会话状态持久化

**代码示例**:
```python
class TaskCompletionDetector:
    def __init__(self, llm_client: BaseLlmClient):
        self.llm = llm_client
    
    async def check_completion(self, task: str, result: str) -> CompletionStatus:
        """判断任务是否完成（LLM辅助）"""
        prompt = f"Task: {task}\nResult: {result}\nIs this task complete?"
        response = await self.llm.chat([Message.user(prompt)])
        return CompletionStatus(is_completed="yes" in response.lower())
    
    def mark_completed(self, session_id: str, task: str) -> None:
        """标记任务完成"""
        # 持久化到session状态
        pass
```

**验收标准**:
```python
detector = TaskCompletionDetector(llm_client)
status = await detector.check_completion(
    task="fix bug",
    result="Fixed the null check in line 42"
)
assert status.is_completed == True
```

---

### 任务 2.4: 检查点接口层

**文件**: `python/continuum_sdk/agent/checkpoint.py`

**目标**: Python调用Rust检查点功能

**实现内容**:
- [ ] 导入Rust checkpoint binding
- [ ] Python接口封装
- [ ] 会话状态序列化

**代码示例**:
```python
from _continuum import CheckpointWriter  # Rust binding

class CheckpointClient:
    def __init__(self):
        self._writer = CheckpointWriter()
    
    def save(self, session_id: str, state: Dict) -> str:
        """保存检查点"""
        return self._writer.save(session_id, json.dumps(state))
    
    def restore(self, session_id: str) -> Optional[Dict]:
        """恢复检查点"""
        data = self._writer.load(session_id)
        return json.loads(data) if data else None
```

**依赖**: T2完成 `CheckpointWriter` binding导出

---

### 任务 2.5: SDK打包配置

**文件**: `python/pyproject.toml`

**目标**: maturin混合包配置

**实现内容**:
- [ ] maturin构建配置
- [ ] Rust源码集成到wheel
- [ ] 单包安装验证

**验收标准**:
```bash
pip install continuum-sdk
python -c "from continuum import Agent; print('OK')"
# OK
```

---

### 任务 2.6: SDK文档和示例

**文件**: `python/continuum_sdk/` 各模块docstring

**实现内容**:
- [ ] 所有公共API文档
- [ ] 类型注解完整
- [ ] 使用示例

---

## 三、禁止事项（违反即失败）

```
┌─────────────────────────────────────────────────────────────────────┐
│  ⛔ 绝对禁止                                                        │
├─────────────────────────────────────────────────────────────────────┤
│  ❌ raise NotImplementedError("xxx: waiting for sh-core binding")    │
│  ❌ return f"Executed: {command}"  # 任何固定字符串返回              │
│  ❌ return f"Received: {input}"     # 任何stub响应                   │
│  ❌ # TODO: 实现xxx                                                 │
│  ❌ # 简化实现: 只处理基本情况                                       │
│  ❌ pass  # 空实现                                                  │
│  ❌ if DEBUG: return "mock"        # 任何mock返回                   │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 必须实现                                                       │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 真实调用操作系统API                                            │
│  ✅ 真实读写文件系统                                               │
│  ✅ 真实执行shell命令                                              │
│  ✅ 完整错误处理                                                   │
└─────────────────────────────────────────────────────────────────────┘
```

## 四、输出规范（必须提供）

每个任务完成后必须提供：

### 4.1 代码证据
```python
# 必须提供真实执行结果，例如：

# 错误示例（禁止）：
# "read_file已实现" - 无证据

# 正确示例：
"""
任务1.1 read_file 完成证据：

>>> tools = BuiltinTools()
>>> content = tools.read_file("test/fixtures/sample.py")
>>> print(content[:50])
'def calculate_average(numbers):\n    """Calculate'

>>> tools.write_file("test/output.txt", "hello")
>>> os.path.exists("test/output.txt")
True
>>> tools.read_file("test/output.txt")
'hello'
"""
```

### 4.2 测试证据
```bash
# 必须提供测试运行结果：
$ pytest tests/test_builtin.py::test_read_file -v
========================= test session starts =========================
collected 1 item

test_builtin.py::test_read_file PASSED                          [100%]

========================== 1 passed in 0.05s ==========================
```

### 4.3 禁止的输出格式
```
❌ "任务已完成"
❌ "功能已实现"
❌ "测试全部通过"
❌ "代码已提交"

以上声明若无具体证据，视为假装完成，任务失败。
```

---

## 五、文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `tools/builtin.py` | 修改 | 实现所有工具方法 |
| `tools/bash.py` | 修改 | 真实命令执行 |
| `tools/file_ops.py` | 修改 | 文件操作实现 |
| `tools/search.py` | 修改 | 搜索实现 |
| `agent/task_completion.py` | 新建 | 任务自检 |
| `agent/checkpoint.py` | 新建 | 检查点集成 |
| `agent/intelligent.py` | 修改 | 集成真实工具 |

---

## 六、验收清单

```
✅ tools = BuiltinTools(); tools.read_file("x") 返回真实内容
✅ tools.bash("echo hi") 执行真实命令
✅ tools.grep("pattern", path=".") 返回真实搜索结果
✅ TaskCompletionDetector 能判断任务完成
✅ CheckpointClient 能保存恢复会话
✅ IntelligentAgent.run() 能执行真实任务
✅ 所有测试通过
✅ 无 NotImplementedError
```

**验收通过时间**: 2026-05-24
**验收人**: T0

---

## 七、汇报流程

```
完成任务后：
1. 更新本文档，将 [ ] 改为 [x]
2. 在任务下方添加完成证据（代码输出、测试结果）
3. 通知T0审查

禁止：
❌ 通过PR提交代码
❌ 只写"完成"不提供证据
❌ 修改T0文档
```

---

**维护者**: T0-new
**最后更新**: 2026-05-24
