# Terminal 1 任务清单 - Layer 4

> 分配时间: 2026-05-11
> 负责层级: Layer 4 (部分 Integration)
> 角色: 集成模块开发者
> 前置完成: Layer 0-1 全部完成 ✅

---

## 任务概览

**负责模块** (2/5):
- mcp_bridge - MCP 协议桥接
- audit_logger - 审计日志

---

## 当前任务

### 任务 1.1: 实现 mcp_bridge

**优先级**: P0
**预计时间**: 4-5小时

```
功能:
├── MCP (Model Context Protocol) 协议实现
├── 工具注册与发现
├── 消息路由
├── 流式响应支持
└── 错误处理

目录结构:
rust/layer4/src/mcp_bridge/
├── mod.rs           # 导出和 trait
├── bridge.rs        # McpBridge 实现
├── protocol.rs      # MCP 协议定义
├── transport.rs     # 传输层 (stdio/socket)
└── handler.rs       # 消息处理器
```

**接口设计**:
```rust
/// MCP 桥接器
pub struct McpBridge {
    transport: Box<dyn McpTransport>,
    handlers: HashMap<String, Box<dyn McpHandler>>,
}

pub trait McpTransport: Send + Sync {
    async fn send(&self, message: &McpMessage) -> Result<()>;
    async fn receive(&self) -> Result<McpMessage>;
}

pub trait McpHandler: Send + Sync {
    async fn handle(&self, request: &McpRequest) -> Result<McpResponse>;
}

impl McpBridge {
    pub async fn new(transport: McpTransportType) -> Result<Self>;
    pub async fn register_tool(&self, tool: ToolDefinition) -> Result<()>;
    pub async fn call_tool(&self, name: &str, args: &Value) -> Result<ToolResult>;
    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>>;
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self) -> Result<()>;
}
```

---

### 任务 1.2: 实现 audit_logger

**优先级**: P0
**预计时间**: 3-4小时

```
功能:
├── 操作审计记录
├── 合规日志格式
├── 敏感数据脱敏
├── 日志轮转
└── 查询接口

目录结构:
rust/layer4/src/audit_logger/
├── mod.rs           # 导出
├── logger.rs        # AuditLogger 实现
├── entry.rs         # AuditEntry 结构
├── storage.rs       # 存储后端
└── query.rs         # 查询接口
```

**接口设计**:
```rust
/// 审计日志记录器
pub struct AuditLogger {
    storage: Box<dyn AuditStorage>,
    config: AuditConfig,
}

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub result: AuditResult,
    pub details: Value,
    pub ip_address: Option<String>,
}

pub enum AuditResult {
    Success,
    Failure(String),
    Denied,
}

impl AuditLogger {
    pub fn new(config: AuditConfig) -> Self;
    pub async fn log(&self, entry: AuditEntry) -> Result<()>;
    pub async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>>;
    pub async fn export(&self, format: ExportFormat) -> Result<Vec<u8>>;
}
```

---

## 工作目录

```
rust/layer4/src/mcp_bridge/
rust/layer4/src/audit_logger/
tests/integration/test_layer4.rs
```

---

## 依赖关系

```
mcp_bridge 依赖:
├── Layer 3: tool_executor (ToolExecutor trait)
├── Layer 2: tool_registry (ToolRegistry trait)
└── Layer 1: streaming, error_handler

audit_logger 依赖:
├── Layer 1: storage_engine, error_handler
└── Layer 0: pii_scrubber (脱敏)
```

---

## 自检清单

```
□ cargo clippy 无警告
□ cargo fmt 通过
□ cargo test 通过
□ 所有 trait 有文档注释
□ MCP 协议兼容性测试
□ 审计日志格式符合 SOC2
```

---

## 完成标准

- MCP 协议基本功能可用
- 审计日志可记录和查询
- 单元测试覆盖率 > 80%
- 集成测试通过

---

## 注意事项

1. **MCP 协议兼容**: 参考 Anthropic MCP 规范
2. **审计日志安全**: 敏感数据必须脱敏
3. **与 Terminal 2/3 协调**: Layer 4 模块间有依赖
