# 审查报告: Python SDK Core

## 审查人: Terminal 2 (Rust视角)

## 整体评价
- [x] 优秀 / [ ] 良好 / [ ] 需改进

---

## Rust兼容性

| 项目 | 评分 | 说明 |
|------|------|------|
| PyO3绑定正确性 | 5/5 | 正确使用 try/except 导入模式，优雅降级到纯 Python 实现 |
| 内存安全性 | 4/5 | 无明显泄漏风险，工作记忆有大小限制（100条），但缺少显式清理机制 |
| 异步正确性 | 3/5 | CustomTool/DAG 为异步，但 Agent.run() 不是异步，存在设计不一致 |
| 性能影响 | 4/5 | 轻量级封装，性能开销低，但占位实现较多 |

---

## 发现的问题

### 1. 异步设计不一致 (中等优先级)
**位置**: `agent/runtime.py:206-259`
```python
def execute(self, task: str) -> str:  # 不是 async
    if self._rust_agent:
        return self._rust_agent.execute(task)
    ...

def run(self, task: str, auto_start: bool = True) -> str:  # 不是 async
```
**问题**: LLM 调用本质是 I/O 密集型操作，应为异步。当前设计无法处理实际 LLM API 调用。

**建议**: 将 `execute()` 和 `run()` 改为 `async def`，或在 Rust 层提供同步包装。

### 2. 模型名称过时 (低优先级)
**位置**: `agent/runtime.py:38`
```python
model: str = "claude-3-sonnet",  # 过时
```
**问题**: 使用过时的模型名称，与 `config/providers.py` 中定义的 `"claude-sonnet-4-6"` 不一致。

**建议**: 从 Config 模块获取默认模型。

### 3. BuiltinTools 占位实现 (低优先级)
**位置**: `tools/builtin.py:118-226`
```python
def read_file(self, path: str, ...) -> str:
    raise NotImplementedError("read_file: waiting for sh-core binding")
```
**问题**: 所有内置工具都是占位实现，功能不可用。

**建议**: 完成 sh-core 绑定或添加 mock 实现用于测试。

### 4. 工具装饰器中的动态类创建 (低优先级)
**位置**: `tools/custom.py:159-185`
```python
class DecoratedTool(CustomTool):  # 动态创建类
    ...
```
**问题**: 每次装饰器调用都创建新类，可能导致内存碎片。

**建议**: 使用 `functools.update_wrapper` 或缓存类定义。

### 5. Memory 查询仅支持简单字符串匹配 (低优先级)
**位置**: `memory/layers.py:204-211`
```python
if query.lower() in entry.content.lower():  # 简单字符串匹配
```
**问题**: 缺少语义搜索能力，与设计文档中的"向量嵌入"不符。

**建议**: 集成 embeddings 模块或标注为 MVP 简化实现。

---

## 改进建议

### 高优先级
1. **统一异步设计**: 将 Agent 核心方法改为异步，或明确标注"同步包装"模式
2. **完善 Rust 绑定检测**: 添加版本检查，确保 Python SDK 与 sh-core 版本兼容

### 中优先级
1. 添加配置验证机制，避免运行时才发现配置错误
2. 在 Session 类中添加资源清理方法 `__del__` 或 `close()`

### 低优先级
1. 更新默认模型名称
2. 添加日志记录（当前无日志）
3. 添加类型检查配置（py.typed 文件）

---

## 架构亮点

1. **优雅降级**: `HAS_RUST_BINDINGS` 模式允许在没有 Rust 绑定时运行纯 Python 实现
2. **分层设计**: Config -> AgentConfig 清晰分离关注点
3. **链式调用**: DAG/Node 支持流畅的链式 API
4. **Provider 抽象**: 多提供商支持设计良好

---

## 结论
- [x] 可以通过
- [ ] 需要修改后通过
- [ ] 需要重大修改

**总结**: Python SDK Core 设计合理，PyO3 绑定模式正确。主要问题是异步设计不一致和BuiltinTools占位实现。建议在下一迭代中统一异步接口并完成 Rust 绑定。

**审查日期**: 2026-05-12
