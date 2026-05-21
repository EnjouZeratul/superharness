# 场景：会话中断恢复

> ID: scenario_session_recovery  
> 依赖: 无（基础设施已存在）  
> 优先级: P0  
> 状态: **已验证通过**

---

## 目标

验证会话中断后完整恢复：
1. 检查点自动保存
2. 中断状态检测
3. 状态完整恢复
4. 无数据丢失

---

## 前置条件

- [ ] CheckpointWriter 已初始化
- [ ] 存储路径可写
- [ ] 会话有活跃状态

---

## 测试步骤

### 步骤 1：创建活跃会话

**操作**：
```python
session = Session(id="interrupt-test")
session.add_user_message("I'm working on a Python project")
session.add_assistant_message("I can help with your Python project")
# 保存检查点
checkpoint_id = session.save_checkpoint()
```

**验证点**：
- [ ] 会话创建成功
- [ ] 消息添加成功
- [ ] 检查点保存成功

### 步骤 2：添加更多消息（未保存）

**操作**：
```python
session.add_user_message("Let me describe my problem...")
session.add_assistant_message("Please go ahead")
# 不保存检查点
```

**验证点**：
- [ ] 消息添加成功
- [ ] 这些消息不在检查点中

### 步骤 3：模拟中断

**模拟方式**：
- 强制终止进程
- 网络断开
- 用户取消操作

**验证点**：
- [ ] 进程异常终止
- [ ] 最后保存的检查点保留

### 步骤 4：检测中断

**Agent 行为**：
1. 启动时检查是否有未关闭的会话
2. 检查检查点时间戳与退出时间
3. 确认是否需要恢复

**预期输出**：
```
检测到未正常关闭的会话：
  session_id: interrupt-test
  last_checkpoint: 2026-05-21T10:30:00
  recovery_available: true
```

**验证点**：
- [ ] 中断正确检测
- [ ] 会话信息正确

### 步骤 5：恢复会话

**Agent 行为**：
1. 加载最后检查点
2. 恢复消息历史
3. 恢复工具状态
4. 提示用户可选择继续或放弃

**预期输出**：
```
会话已恢复：
  消息数: 4
  检查点迭代: 4
  工具调用待处理: 1
  
是否继续上次任务？(Y/n)
```

**验证点**：
- [ ] 检查点加载成功
- [ ] 消息历史完整
- [ ] 工具状态恢复
- [ ] 用户确认提示

### 步骤 6：验证数据完整性

**验证项**：
- [ ] 消息顺序正确
- [ ] 消息内容完整
- [ ] Token 统计正确
- [ ] 元数据保留

---

## 成功标准

| 指标 | 预期值 |
|------|--------|
| 检查点保存成功率 | 100% |
| 恢复成功率 | 100% |
| 数据完整性 | 100% |
| 无数据丢失 | 所有保存的数据恢复 |
| 恢复时间 | ≤ 2 秒 |

---

## 边界条件

### 边界 1：空会话恢复

**输入**：无任何消息的会话  
**预期**：正确处理，无错误

### 边界 2：损坏的检查点

**输入**：检查点文件损坏  
**预期**：从备份恢复或报告无法恢复

### 边界 3：多次中断

**输入**：连续多次中断  
**预期**：恢复到最近有效检查点

### 边界 4：大会话恢复

**输入**：10000+ 消息的会话  
**预期**：正确恢复，时间合理

---

## 错误恢复场景

### 错误 1：检查点加载失败

**触发**：文件损坏或格式不兼容  
**预期**：
- 尝试备份文件
- 如无备份，报告丢失并创建新会话

### 错误 2：存储不可访问

**触发**：磁盘满或权限错误  
**预期**：
- 报告错误
- 使用内存缓冲（临时）

### 错误 3：校验和不匹配

**触发**：数据被篡改  
**预期**：
- 报告校验失败
- 拒绝加载损坏数据

---

## 测试代码参考

```python
from continuum.checkpoint_writer import CheckpointWriter, CrashRecovery

# 创建检查点
writer = CheckpointWriter(storage_path="./checkpoints")
checkpoint = {
    "session_id": "test-session",
    "messages": [{"role": "user", "content": "Hello"}],
    "iteration": 1
}
writer.save_checkpoint(checkpoint, "test-session")

# 模拟中断后恢复
recovery = CrashRecovery(writer)
restored, error = recovery.recover_session("test-session")
assert restored is not None
assert restored["session_id"] == "test-session"
```

---

## 检查清单

执行前确认：
- [ ] 存储路径存在
- [ ] 有写入权限
- [ ] CheckpointWriter 正常工作

执行后验证：
- [ ] 检查点文件存在
- [ ] 校验和正确
- [ ] 恢复后数据完整
- [ ] 无临时文件残留

---

*Continuum User Scenario - Session Recovery*