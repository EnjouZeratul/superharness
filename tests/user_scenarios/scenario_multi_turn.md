# 场景：多轮对话上下文保持

> ID: scenario_multi_turn  
> 依赖: T1 P1.2 Agent智能增强  
> 优先级: P0  
> 状态: **已验证通过**

---

## 目标

验证多轮对话能力：
1. 上下文正确传递
2. 历史记录完整保留
3. 会话恢复后上下文完整
4. 长对话管理合理
5. 主题切换与回归

---

## 前置条件

- [ ] Session 模块正常工作
- [ ] 存储路径可写
- [ ] LLM 连接正常

---

## 测试步骤

### 步骤 1：创建会话并开始对话

**操作**：
```python
session = Session(id="multi-turn-test")
session.add_user_message("My name is Alice and I work on Python projects.")
session.add_assistant_message("Nice to meet you, Alice! I can help with Python.")
```

**验证点**：
- [ ] 会话创建成功
- [ ] 消息添加成功
- [ ] 消息顺序正确

### 步骤 2：引用第一轮信息

**操作**：
```python
session.add_user_message("What is my name?")
session.add_assistant_message("Your name is Alice, as you mentioned earlier.")
```

**验证点**：
- [ ] 上下文正确传递
- [ ] 第一轮信息被引用
- [ ] 回答准确

### 步骤 3：深度对话（多轮）

**操作序列**：
```
用户: "I'm building a web API with Flask."
助手: "Flask is great for web APIs..."
用户: "Can you help me add authentication?"
助手: "Sure! I recommend Flask-Login or JWT..."
用户: "Let's go with JWT."
助手: "First, install PyJWT..."
用户: "What about database?"
助手: "SQLAlchemy is a good choice..."
用户: "Remind me, what framework am I using?"
助手: "You're using Flask, as mentioned earlier."
```

**验证点**：
- [ ] 上下文连续传递
- [ ] 早期信息仍可引用
- [ ] 主题回归正确

### 步骤 4：验证上下文完整性

**验证项**：
```python
messages = session.get_messages()
checks = {
    "name_context": any("Alice" in m.content for m in messages),
    "framework_context": any("Flask" in m.content for m in messages),
    "auth_context": any("JWT" in m.content for m in messages),
    "db_context": any("SQLAlchemy" in m.content for m in messages),
}
assert all(checks.values())
```

**验证点**：
- [ ] 名字信息保留
- [ ] 框架选择保留
- [ ] 认证方案保留
- [ ] 数据库选择保留

### 步骤 5：会话导出导入

**操作**：
```python
exported = session.export()
restored = Session.from_export(exported)
assert restored.message_count == session.message_count
```

**验证点**：
- [ ] 导出成功
- [ ] 导入成功
- [ ] 消息数量一致

### 步骤 6：恢复后上下文保持

**验证项**：
```python
messages = restored.get_messages()
assert any("Alice" in m.content for m in messages)
assert any("Flask" in m.content for m in messages)
```

**验证点**：
- [ ] 恢复后上下文完整
- [ ] 可以继续对话

---

## 成功标准

| 指标 | 预期值 |
|------|--------|
| 上下文传递准确率 | 100% |
| 历史记录完整性 | 100% |
| 导入导出成功率 | 100% |
| 恢复后完整性 | 100% |
| 长对话稳定性 | ≥ 50 轮无问题 |

---

## 边界条件

### 边界 1：超长对话

**输入**：100+ 轮对话  
**预期**：
- 合理管理历史长度
- 关键信息不丢失
- Token 使用可控

### 边界 2：主题切换

**输入**：中途切换完全不同的话题  
**预期**：
- 正确处理切换
- 原话题信息保留
- 可随时回归原话题

### 边界 3：多语言混合

**输入**：中英文混合对话  
**预期**：
- 正确处理多语言
- 编码无乱码
- 上下文无歧义

### 边界 4：代码上下文

**输入**：讨论中包含代码片段  
**预期**：
- 代码正确保留
- 格式保持
- 可引用代码上下文

---

## 错误恢复场景

### 错误 1：会话导出失败

**触发**：存储不可写  
**预期**：报告错误，内存数据保留

### 错误 2：导入损坏数据

**触发**：JSON 格式错误  
**预期**：报告错误，拒绝加载

### 错误 3：并发修改

**触发**：多线程同时修改会话  
**预期**：线程安全处理，无数据损坏

---

## 测试代码参考

```python
from continuum_sdk.agent.session import Session

session = Session(id="test")
session.add_user_message("My name is Alice")
session.add_assistant_message("Hello Alice!")

# 导出导入
exported = session.export()
restored = Session.from_export(exported)
assert restored.message_count == session.message_count
```

---

## 检查清单

执行前确认：
- [ ] Session 模块导入正常
- [ ] 存储路径可写
- [ ] 基本操作测试通过

执行后验证：
- [ ] 所有轮次消息保留
- [ ] 导出导入成功
- [ ] 恢复后继续对话正常
- [ ] 无消息丢失

---

*Continuum User Scenario - Multi-turn Context*