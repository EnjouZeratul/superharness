# 场景：MCP 工具调用

> ID: scenario_mcp_tool_call  
> 依赖: T2 P1.4 MCP协议支持  
> 优先级: P1

---

## 目标

验证 MCP 协议完整流程：
1. MCP 客户端连接服务器
2. 服务器工具发现
3. 工具调用与结果返回
4. 多服务器连接
5. 错误处理与恢复

---

## 前置条件

- [ ] MCP 客户端已实现
- [ ] 至少 1 个 MCP 服务器可用
- [ ] Transport 层正常工作

---

## 测试步骤

### 步骤 1：连接 MCP 服务器

**操作**：
```python
from continuum_sdk.mcp import MCPClient

client = MCPClient()
await client.connect("http://localhost:8080")
```

**验证点**：
- [ ] 连接建立成功
- [ ] 握手协议完成
- [ ] 服务器信息获取

### 步骤 2：发现可用工具

**Agent 行为**：
1. 请求工具列表
2. 解析工具描述
3. 分类展示

**预期输出**：
```
Available MCP tools:
  [search] web_search - Search the web
  [file]   read_remote - Read file from server
  [compute] evaluate - Execute code
```

**验证点**：
- [ ] 工具列表非空
- [ ] 描述信息完整
- [ ] 分类正确

### 步骤 3：调用工具

**操作**：
```python
result = await client.call_tool("web_search", {
    "query": "Python FizzBuzz"
})
```

**预期输出**：
```json
{
    "content": "Search results for 'Python FizzBuzz'...",
    "is_error": false,
    "metadata": {"source": "web_search", "latency_ms": 150}
}
```

**验证点**：
- [ ] 工具调用成功
- [ ] 结果格式正确
- [ ] 延迟合理

### 步骤 4：多服务器连接

**操作**：
```python
client2 = MCPClient()
await client2.connect("http://localhost:8081")

# 跨服务器工具调用
result = await client.call_tool("read_remote", {"path": "/data/config.json"})
result2 = await client2.call_tool("evaluate", {"code": "1 + 1"})
```

**验证点**：
- [ ] 多连接共存
- [ ] 跨服务器调用独立
- [ ] 无资源冲突

### 步骤 5：断开连接

**操作**：
```python
await client.disconnect()
```

**验证点**：
- [ ] 连接正确关闭
- [ ] 资源释放
- [ ] 无异常退出

---

## 成功标准

| 指标 | 预期值 |
|------|--------|
| 连接成功率 | 100% |
| 工具发现准确性 | 100% |
| 工具调用成功率 | ≥ 95% |
| 调用延迟 | ≤ 5s |
| 多服务器稳定 | 无冲突 |

---

## 边界条件

### 边界 1：服务器不可达

**输入**：连接不存在的服务器  
**预期**：超时后报告错误，不崩溃

### 边界 2：无效工具名

**输入**：调用不存在的工具  
**预期**：返回明确错误信息

### 边界 3：大参数

**输入**：工具参数超过 1MB  
**预期**：正确处理或优雅拒绝

### 边界 4：并发调用

**输入**：同时发起 10 个工具调用  
**预期**：全部完成，无竞态条件

---

## 错误恢复场景

### 错误 1：连接中断

**触发**：服务器中途断开  
**预期**：自动重连或通知用户

### 错误 2：调用超时

**触发**：工具执行超过时限  
**预期**：取消调用并返回超时错误

### 错误 3：结果解析失败

**触发**：返回格式不符合预期  
**预期**：报告格式错误，提供原始结果

---

## 检查清单

执行前确认：
- [ ] MCP 服务器运行中
- [ ] 端口未被占用
- [ ] 客户端已初始化

执行后验证：
- [ ] 所有调用完成
- [ ] 无连接泄漏
- [ ] 日志记录完整

---

*Continuum User Scenario - MCP Tool Call*