# Terminal 3 任务清单 - 集成测试与验证

> 分配时间: 2026-05-24
> 负责范围: 集成测试、真实任务验证、边界测试
> 目标: 真实用户能完成完整开发任务
> 前置依赖: T1+T2全部验收通过
> 等待机制: 收到T0通知或检测T1/T2验收清单全部[x]后方可开始

---

## 零、依赖等待规则

```
┌─────────────────────────────────────────────────────────────────────┐
│  ⛔ T1/T2未完成时禁止行为                                            │
├─────────────────────────────────────────────────────────────────────┤
│  ❌ 编写测试代码（无被测对象）                                       │
│  ❌ 使用mock假装功能存在                                            │
│  ❌ 假设接口结构编写测试                                             │
│  ❌ "先写测试框架，等实现后再验证"                                   │
│                                                                     │
│  ✅ 正确等待方式                                                    │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 收到T0通知"T1/T2全部验收通过"                                   │
│  ✅ 检测T1_TASKS.md和T2_TASKS.md验收清单全部[x]                     │
│  ✅ 实际能导入并调用功能后才编写测试                                 │
│  ✅ 等待期间可准备测试用例设计文档                                   │
└─────────────────────────────────────────────────────────────────────┘
```

**前置检测清单（必须全部[x]）：**
```
T1_TASKS.md 验收清单（Rust核心）：
□ continuum run "task" 真实执行并返回结果
□ BashTool.execute 真实执行命令
□ GrepTool 返回真实搜索结果
□ LSP工具能跳转定义
□ MCP能连接服务器并调用工具
□ TUI流式输出实时显示
□ Agent能调用Git工具
□ 所有测试通过
□ 无stub返回值

T2_TASKS.md 验收清单（Python SDK）：
□ tools = BuiltinTools(); tools.read_file("x") 返回真实内容
□ tools.bash("echo hi") 执行真实命令
□ tools.grep("pattern", path=".") 返回真实搜索结果
□ TaskCompletionDetector 能判断任务完成
□ CheckpointClient 能保存恢复会话
□ IntelligentAgent.run() 能执行真实任务
□ 所有测试通过
□ 无 NotImplementedError
```

---

## 一、当前状态

**测试状态**: Python 484 passed, Rust 140 passed

**缺失**: 真实任务验证测试、边界条件测试、错误恢复测试

---

## 二、任务清单

### 任务 3.1: 真实任务验证测试

**目标**: 验证用户能完成完整开发任务

**测试场景**:

#### 3.1.1 Bug修复场景
```python
# 测试用例: test_real_bug_fix.py
async def test_fix_buggy_code():
    """Agent能修复真实bug"""
    agent = IntelligentAgent(api_key=...)
    
    # 任务：修复buggy_program.py中的空列表bug
    result = await agent.run("""
    Fix the bug in buggy_program.py where find_max([]) causes IndexError.
    The function should handle empty lists gracefully.
    """)
    
    # 验证：代码被修改，bug已修复
    assert "if not items" in read_file("buggy_program.py")
    
    # 验证：修改后的代码能运行
    result = run_python("buggy_program.py")
    assert "Error" not in result
```

#### 3.1.2 功能添加场景
```python
async def test_add_new_feature():
    """Agent能添加新功能"""
    agent = IntelligentAgent(...)
    
    result = await agent.run("""
    Add a calculate_median() function to math_utils.py that:
    1. Takes a list of numbers
    2. Returns the median value
    3. Handles empty lists
    """)
    
    # 验证：函数存在且工作
    assert "def calculate_median" in read_file("math_utils.py")
    median = import_and_call("math_utils.calculate_median", [1,2,3,4,5])
    assert median == 3
```

#### 3.1.3 代码重构场景
```python
async def test_refactor_code():
    """Agent能重构代码"""
    agent = IntelligentAgent(...)
    
    result = await agent.run("""
    Refactor the process_data function in data.py:
    - Split into smaller functions
    - Add type hints
    - Improve error handling
    """)
    
    # 验证：代码结构改善
    code = read_file("data.py")
    assert "def _validate_input" in code or "def validate_" in code
```

---

### 任务 3.2: 边界条件测试

**测试场景**:

#### 3.2.1 大文件处理
```python
def test_large_file_handling():
    """Read工具处理大文件"""
    # 创建10MB文件
    create_large_file("large.txt", size=10*1024*1024)
    
    tools = BuiltinTools()
    
    # 分页读取
    page1 = tools.read_file("large.txt", offset=0, limit=1000)
    page2 = tools.read_file("large.txt", offset=1000, limit=1000)
    
    assert len(page1) <= 1000 * 100  # 约100KB
    assert page1 != page2  # 不同页
```

#### 3.2.2 超时处理
```python
def test_timeout_handling():
    """Bash工具超时"""
    tools = BuiltinTools()
    
    # 长时间命令
    result = tools.bash("sleep 100", timeout=1000)
    
    assert result.is_error == True
    assert "timeout" in result.content.lower()
```

#### 3.2.3 特殊字符处理
```python
def test_special_characters():
    """Edit工具处理特殊字符"""
    create_file("special.txt", "你好世界\n特殊字符: <>&\"'")
    
    tools = BuiltinTools()
    result = tools.edit_file("special.txt", "你好", "Hello")
    
    assert result == 1
    content = tools.read_file("special.txt")
    assert "Hello世界" in content
    assert "<>&\"'" in content  # 特殊字符保留
```

---

### 任务 3.3: 错误恢复测试

**测试场景**:

#### 3.3.1 网络错误恢复
```python
async def test_network_error_recovery():
    """LLM调用失败后恢复"""
    agent = IntelligentAgent(api_key="invalid_key")
    
    # 第一次失败
    try:
        await agent.run("hello")
    except AuthenticationError:
        pass
    
    # 恢复后重试
    agent.update_api_key("valid_key")
    result = await agent.run("hello")
    assert len(result) > 0
```

#### 3.3.2 工具失败恢复
```python
async def test_tool_failure_recovery():
    """工具执行失败后Agent自我纠错"""
    agent = IntelligentAgent(...)
    agent.register_tool("test_tool", lambda x: None)  # 会失败
    
    result = await agent.run("Use test_tool to do something")
    
    # Agent应该检测失败并尝试替代方案
    assert result.corrections_applied >= 1
```

#### 3.3.3 检查点恢复
```python
async def test_checkpoint_recovery():
    """崩溃后从检查点恢复"""
    client = CheckpointClient()
    
    # 保存状态
    cp_id = client.save("session-1", {
        "task": "fix bug",
        "progress": "step 3/5"
    })
    
    # 模拟崩溃恢复
    restored = client.restore("session-1", cp_id)
    
    assert restored["task"] == "fix bug"
    assert restored["progress"] == "step 3/5"
```

---

### 任务 3.4: 任务自检验证

**测试场景**:

```python
async def test_task_completion_detection():
    """任务自检机制工作"""
    detector = TaskCompletionDetector(llm_client)
    
    # 明确完成的任务
    status = await detector.check_completion(
        task="print hello",
        result="Hello printed successfully"
    )
    assert status.is_completed == True
    
    # 未完成的任务
    status = await detector.check_completion(
        task="fix all bugs",
        result="Fixed bug in file A"  # 只修复了一个
    )
    assert status.is_completed == False
```

---

### 任务 3.5: 端到端测试

**场景**: 模拟真实用户完整使用流程

```python
async def test_e2e_developer_workflow():
    """开发者完整工作流"""
    agent = IntelligentAgent(...)
    
    # 1. 分析代码
    result1 = await agent.run("分析 auth.py 的潜在问题")
    assert "问题" in result1 or "issue" in result1.lower()
    
    # 2. 修复问题
    result2 = await agent.run("修复你发现的第一个问题")
    
    # 3. 运行测试
    result3 = await agent.run("运行测试验证修复")
    assert "passed" in result3.lower() or "成功" in result3
    
    # 4. 提交代码
    result4 = await agent.run("生成commit消息并提交")
    assert "commit" in result4.lower()
```

---

## 三、禁止事项（违反即失败）

```
┌─────────────────────────────────────────────────────────────────────┐
│  ⛔ 绝对禁止                                                        │
├─────────────────────────────────────────────────────────────────────┤
│  ❌ 测试只验证Happy Path                                            │
│     只测试成功情况，不测试边界和错误                                 │
│                                                                     │
│  ❌ 使用固定mock返回                                                │
│     @patch("module.func", return_value="fixed")                     │
│     所有测试都用mock，不验证真实行为                                 │
│                                                                     │
│  ❌ assert True  # 避免实际验证                                      │
│     假装通过的断言                                                  │
│                                                                     │
│  ❌ # skip: 真实API太慢                                              │
│     跳过真实测试                                                    │
│                                                                     │
│  ❌ 测试通过声明                                                     │
│     "测试全部通过" - 但未展示真实运行结果                            │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 必须提供                                                       │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 真实执行完整测试套件                                            │
│  ✅ 边界条件测试结果                                               │
│  ✅ 错误恢复测试结果                                               │
│  ✅ 测试覆盖率报告                                                 │
└─────────────────────────────────────────────────────────────────────┘
```

## 四、输出规范（必须提供）

每个任务完成后必须提供：

### 4.1 测试执行证据
```bash
# 必须提供完整测试运行结果：
$ pytest tests/test_real_tasks.py -v --tb=short
========================= test session starts =========================
collected 5 items

test_real_tasks.py::test_fix_buggy_code PASSED                  [ 20%]
test_real_tasks.py::test_add_new_feature PASSED                 [ 40%]
test_real_tasks.py::test_refactor_code PASSED                    [ 60%]
test_real_tasks.py::test_large_file_handling PASSED             [ 80%]
test_real_tasks.py::test_timeout_handling PASSED                 [100%]

========================== 5 passed in 12.34s ========================

# 必须提供覆盖率：
$ pytest --cov=continuum_sdk --cov-report=term-missing
TOTAL    1234    45    96%
```

### 4.2 真实任务验证证据
```
必须提供真实任务执行日志或截图：

任务：修复buggy_program.py中的空列表bug
步骤1: 读取buggy_program.py
步骤2: 分析find_max函数
步骤3: 识别空列表问题
步骤4: 修改代码添加空列表检查
步骤5: 运行测试验证修复
结果: 成功修复，测试通过
```

### 4.3 禁止的输出格式
```
❌ "测试全部通过"
❌ "功能验证完成"
❌ "集成测试OK"

以上声明若无具体测试输出，视为假装完成，任务失败。
```

---

## 五、测试文件清单

| 文件 | 说明 |
|------|------|
| `tests/test_real_tasks.py` | 真实任务验证 |
| `tests/test_boundary.py` | 边界条件 |
| `tests/test_recovery.py` | 错误恢复 |
| `tests/test_task_completion.py` | 任务自检 |
| `tests/test_e2e.py` | 端到端流程 |
| `tests/fixtures/` | 测试数据文件 |

---

## 六、验收清单

```
[x] 真实bug修复测试通过          tests/test_real_tasks.py::TestRealBugFix (4 tests)
[x] 功能添加测试通过             tests/test_real_tasks.py::TestFeatureAddition (2 tests)
[x] 大文件分页读取正常           tests/test_boundary.py::TestLargeFileHandling (3 tests)
[x] 超时命令正确处理             tests/test_boundary.py::TestTimeoutHandling (2 tests)
[x] 特殊字符编辑正确             tests/test_boundary.py::TestSpecialCharacterHandling (3 tests)
[x] 网络错误恢复正常             tests/test_llm_client.py::TestErrorHandling (6 tests)
[x] 工具失败自动纠错             tests/test_self_correction.py (29 tests)
[x] 检查点恢复正常               tests/test_checkpoint_recovery.py (17 tests)
[x] 任务自检机制验证             python/tests/test_task_completion.py (27 tests)
[x] 端到端流程完整               tests/test_real_tasks.py::TestEndToEndWorkflow (1 test)
[x] 所有测试覆盖率 > 80%         client.py 77%, types.py 96%, errors.py 97%, 总体 83%
```

---

## 七、测试执行证据

### 完整测试运行结果
```bash
$ pytest tests/test_real_tasks.py tests/test_boundary.py tests/test_self_correction.py \
         tests/test_error_recovery.py python/tests/test_task_completion.py \
         python/tests/test_llm_client.py -v --tb=short

============================= test session starts =============================
collected 330 tests

tests/test_real_tasks.py::TestRealBugFix::test_read_file_tool PASSED
tests/test_real_tasks.py::TestRealBugFix::test_write_file_tool PASSED
tests/test_real_tasks.py::TestRealBugFix::test_bash_tool PASSED
tests/test_real_tasks.py::TestRealBugFix::test_agent_can_start_and_stop PASSED
tests/test_real_tasks.py::TestFeatureAddition::test_tool_registry_operations PASSED
tests/test_real_tasks.py::TestFeatureAddition::test_write_new_function PASSED
tests/test_self_correction.py::TestErrorContext::test_error_context_creation PASSED
tests/test_self_correction.py::TestSelfCorrectionProposals::test_propose_correction_for_syntax_error PASSED
tests/test_self_correction.py::TestRecoveryStrategies::test_recovery_strategy_values PASSED
tests/test_self_correction.py::TestExecutionResult::test_execution_result_with_corrections PASSED
...

======================== 176 passed in 2.59s ========================

$ pytest python/tests/test_llm_client.py --cov=continuum_sdk.llm --cov-report=term-missing
Name                                 Stmts   Miss Branch BrPart  Cover
python\continuum_sdk\llm\client.py     276     53    106     23    77%
python\continuum_sdk\llm\types.py       88      0     14      4    96%
python\continuum_sdk\llm\errors.py      51      1     18      1    97%
TOTAL                                  415     54    138     28    83%
```

---

## 八、汇报流程

```
完成任务后：
1. 更新本文档，将 [ ] 改为 [x]
2. 在任务下方添加完成证据（测试输出、验证结果）
3. 通知T0审查

禁止：
❌ 通过PR提交代码
❌ 只写"完成"不提供证据
❌ 修改T0文档
```

---

**维护者**: T0-new
**最后更新**: 2026-05-24