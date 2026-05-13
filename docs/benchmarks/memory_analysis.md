# 内存使用分析报告

> 生成日期: 2026-05-12
> 分析方法: 静态代码分析 + 理论估算
> 注意: Windows 环境下 valgrind 不可用，建议在 Linux 环境进行实测

---

## 1. 静态分析

### 1.1 主要数据结构

| 结构体 | 大小 (估算) | 说明 |
|--------|-------------|------|
| ConfigManager | ~200 bytes | HashMap + String + GlobalSettings |
| ProviderConfig | ~100 bytes | 4 String + 2 数值 |
| GlobalSettings | ~50 bytes | 6 布尔值 + 3 数值 |
| Session (简化) | ~50 bytes + messages | Vec<Message> |
| Message | ~50 bytes | role + content + timestamp |
| Memory | ~100 bytes + entries | 4 Vec<MemoryEntry> |
| MemoryEntry | ~100 bytes | content + metadata + timestamps |

### 1.2 动态内存分析

#### ConfigManager
```
Base: ~200 bytes
Per Provider: ~100 bytes
Per Extra Config: ~20 bytes (key) + value size

示例（10 providers, 20 extra configs）:
= 200 + 10*100 + 20*40 = 200 + 1000 + 800 = ~2 KB
```

#### Session
```
Base: ~50 bytes
Per Message: ~50 bytes + content_size

示例（100 messages, avg 200 bytes content）:
= 50 + 100*(50 + 200) = 50 + 25000 = ~25 KB
```

#### Memory (Working tier)
```
Base: ~100 bytes
Limit: 100 entries
Per Entry: ~100 bytes + content_size

示例（100 entries, avg 500 bytes）:
= 100 + 100*(100 + 500) = 100 + 60000 = ~60 KB
```

---

## 2. 内存泄漏风险分析

### 2.1 低风险

| 模块 | 风险 | 说明 |
|------|------|------|
| ConfigManager | ✅ 低 | Rust 自动管理，无手动分配 |
| ProviderConfig | ✅ 低 | 简单数据结构 |
| GlobalSettings | ✅ 低 | 原始类型和 String |
| Session | ✅ 低 | Vec 自动扩缩容 |

### 2.2 需关注

| 模块 | 风险 | 说明 |
|------|------|------|
| Memory | ⚠️ 中 | Working tier 有 100 条限制，但其他层无限制 |
| PyO3 bindings | ⚠️ 中 | Python 侧可能持有引用导致 Rust 对象不释放 |

### 2.3 Python 侧风险

| 问题 | 位置 | 风险 |
|------|------|------|
| Session 未清理 | agent/runtime.py | ⚠️ 中 |
| Memory 无大小限制 | memory/layers.py | ⚠️ 中 |
| 全局注册表 | tools/custom.py:265 | ⚠️ 中 |

```python
# tools/custom.py:265
default_registry = ToolRegistry()  # 全局变量，永不释放
```

---

## 3. 峰值内存估算

### 场景 1: 单 Agent 运行

```
Config: ~2 KB
Session (100 msgs): ~25 KB
Memory (Working): ~60 KB
Memory (Session): ~100 KB (估算)
Memory (Project): ~200 KB (估算)
Rust runtime: ~5 MB (估算)
Python runtime: ~30 MB (估算)
-------------------------------
Total: ~35.4 MB
```

### 场景 2: 多 Agent 并发

```
10 Agents * 35 MB = ~350 MB
+ 共享 Config: ~2 KB
+ 共享 Memory (Project): ~200 KB
-------------------------------
Total: ~350 MB
```

### 场景 3: 大型会话

```
Session (1000 msgs, avg 500 bytes):
= 50 + 1000*(50 + 500) = ~550 KB

Memory (all tiers, 500 entries):
= ~300 KB

-------------------------------
Single Agent: ~6 MB (不含运行时)
```

---

## 4. 内存安全检查清单

### 4.1 Rust 侧 ✅

- [x] 无 `unsafe` 代码（已审计）
- [x] 无手动内存分配
- [x] 使用 `Arc` 共享所有权
- [x] 使用 `Mutex` 保护并发访问
- [x] `Drop` trait 自动清理

### 4.2 Python 侧 ⚠️

- [ ] Session 缺少显式清理方法
- [ ] Memory 部分层无大小限制
- [ ] 全局注册表无清理机制
- [ ] Agent 未实现 `__del__`

---

## 5. 优化建议

### 5.1 高优先级

1. **添加 Session 清理**
```python
def close(self):
    """释放资源"""
    self._messages.clear()
    if self._rust_session:
        # 调用 Rust 侧清理
        pass
```

2. **限制 Memory 各层大小**
```python
TIER_LIMITS = {
    MemoryTier.WORKING: 100,
    MemoryTier.SESSION: 500,
    MemoryTier.PROJECT: 5000,
    MemoryTier.LONG_TERM: 10000,
}
```

### 5.2 中优先级

1. 添加 Agent `__del__` 方法
2. 实现全局注册表清理
3. 添加内存使用监控

### 5.3 低优先级

1. 使用 `__slots__` 优化 Python 对象内存
2. 实现消息压缩
3. 添加 LRU 缓存淘汰策略

---

## 6. 建议的内存测试方案

### Linux 环境

```bash
# Valgrind 内存泄漏检测
valgrind --leak-check=full --show-leak-kinds=all \
    cargo test --release

# Heaptrack 内存分析
heaptrack cargo test --release
```

### Windows 环境

```powershell
# 使用 Windows Performance Analyzer
# 或 Dr. Memory 工具
drmemory -lean -batch -cargo test --release
```

### Python 侧

```python
# 使用 memory_profiler
from memory_profiler import profile

@profile
def test_memory_usage():
    agent = Agent()
    # ...
```

---

## 7. 结论

### 当前状态
- Rust 侧内存安全有保障
- Python 侧存在潜在内存增长风险
- 峰值内存在可接受范围（<500 MB）

### 建议
1. 实现 Session 显式清理
2. 为 Memory 各层添加大小限制
3. 在 Linux 环境进行 Valgrind 测试
4. 添加内存使用监控 API

---

## 附录: 快速内存检查脚本

```bash
# Linux 环境
#!/bin/bash
echo "Running memory tests..."
cargo build --release
valgrind --leak-check=full ./target/release/deps/*test* 2>&1 | grep "LEAK SUMMARY"
```
