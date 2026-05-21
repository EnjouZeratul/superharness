# 场景：Agent 自主修复 Bug

> ID: scenario_fix_bug  
> 依赖: T1 P1.2 Agent智能增强  
> 优先级: P0

---

## 目标

验证 Agent 能够自主完成以下任务：
1. 理解 bug 描述
2. 定位 bug 位置
3. 分析 bug 原因
4. 实施修复方案
5. 验证修复有效
6. 提交变更

---

## 前置条件

- [ ] Agent 已初始化并连接 LLM
- [ ] 项目代码仓库已克隆
- [ ] 测试环境已配置
- [ ] Git 工具可用

---

## 测试步骤

### 步骤 1：提供 Bug 描述

**输入**：
```
修复 fizzbuzz.py 中的 bug。

问题描述：
- fizzbuzz(15) 的第15个元素应该是 "FizzBuzz"
- 但当前输出是 "Fizz"
- 这是因为缺少 FizzBuzz 组合判断 (i % 15 == 0)

预期行为：
- 对于能同时被 3 和 5 整除的数，输出 "FizzBuzz"
- 对于只被 3 整除的数，输出 "Fizz"
- 对于只被 5 整除的数，输出 "Buzz"
- 其他情况输出数字本身
```

**预期输出**：
- Agent 理解问题并确认收到任务

### 步骤 2：Agent 定位 Bug

**Agent 行为**：
1. 使用 `read_file` 工具读取 `fizzbuzz.py`
2. 分析代码逻辑
3. 确认 bug 原因：缺少 `i % 15 == 0` 判断

**验证点**：
- [ ] Agent 正确读取文件
- [ ] Agent 准确指出 bug 位置（行号）
- [ ] Agent 理解 bug 原因

### 步骤 3：Agent 实施修复

**Agent 行为**：
1. 使用 `edit_file` 工具修改代码
2. 在 `i % 3 == 0` 判断前添加 `i % 15 == 0` 判断
3. 添加 "FizzBuzz" 输出

**验证点**：
- [ ] edit_file 工具正确执行
- [ ] 代码逻辑正确修改
- [ ] 无语法错误

### 步骤 4：Agent 验证修复

**Agent 行为**：
1. 使用 `bash` 工具运行测试 `pytest test_fizzbuzz.py`
2. 分析测试结果
3. 确认所有测试通过

**验证点**：
- [ ] 测试命令正确执行
- [ ] 测试通过（test_fizzbuzz_15 现在应该通过）
- [ ] Agent 正确解读测试结果

### 步骤 5：Agent 提交变更

**Agent 行为**：
1. 使用 Git 工具查看变更
2. 生成 commit 消息
3. 提交变更

**验证点**：
- [ ] commit 消息质量良好
- [ ] 变更正确提交

---

## 成功标准

| 指标 | 预期值 |
|------|--------|
| Bug 定位准确率 | 100% |
| 修复正确率 | 100% |
| 测试通过率 | 100% |
| 完成时间 | ≤ 3 分钟 |
| 无用户干预 | 全程无用户额外输入 |

---

## 边界条件

### 边界 1：无 Bug 情况

**输入**：代码没有 bug  
**预期**：Agent 应报告"未发现 bug"

### 边界 2：多个 Bug

**输入**：代码有多个 bug  
**预期**：Agent 应逐一修复或报告所有 bug

### 边界 3：复杂 Bug

**输入**：需要多文件修改的 bug  
**预期**：Agent 应协调多文件修改

### 边界 4：无法修复

**输入**：超出 Agent 能力的 bug  
**预期**：Agent 应请求用户协助

---

## 错误恢复场景

### 错误 1：文件读取失败

**触发**：文件不存在或权限不足  
**预期**：Agent 重试或请求用户确认路径

### 错误 2：编辑失败

**触发**：字符串不匹配  
**预期**：Agent 重新读取文件并调整修复策略

### 错误 3：测试失败

**触发**：修复后测试仍失败  
**预期**：Agent 分析失败原因并重新修复

---

## 测试数据

**初始代码** (`fizzbuzz_buggy.py`)：
```python
def fizzbuzz(n):
    """FizzBuzz implementation with a bug."""
    result = []
    for i in range(1, n + 1):
        if i % 3 == 0:
            result.append("Fizz")
        elif i % 5 == 0:
            result.append("Buzz")
        else:
            result.append(str(i))
    return result
```

**测试文件** (`test_fizzbuzz.py`)：
```python
import pytest
from fizzbuzz import fizzbuzz

def test_fizzbuzz_15():
    result = fizzbuzz(15)
    assert result[14] == "FizzBuzz", f"Expected FizzBuzz, got {result[14]}"

def test_fizzbuzz_3():
    result = fizzbuzz(10)
    assert result[2] == "Fizz"

def test_fizzbuzz_5():
    result = fizzbuzz(10)
    assert result[4] == "Buzz"
```

**预期修复后代码**：
```python
def fizzbuzz(n):
    """FizzBuzz implementation - fixed."""
    result = []
    for i in range(1, n + 1):
        if i % 15 == 0:  # FizzBuzz combination
            result.append("FizzBuzz")
        elif i % 3 == 0:
            result.append("Fizz")
        elif i % 5 == 0:
            result.append("Buzz")
        else:
            result.append(str(i))
    return result
```

---

## 检查清单

执行前确认：
- [ ] LLM API Key 配置正确
- [ ] Agent 初始化成功
- [ ] 项目目录正确
- [ ] 测试框架可用

执行后验证：
- [ ] Bug 已修复
- [ ] 所有测试通过
- [ ] 代码已提交
- [ ] 无额外文件残留
- [ ] 日志完整记录

---

*Continuum User Scenario - Fix Bug*