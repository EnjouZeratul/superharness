# Continuum 架构详解

> 版本: v1.0
> 日期: 2026-05-24
> 目标读者: 开发者、贡献者

本文档解释 Continuum 的架构设计，帮助开发者理解系统结构、模块职责和扩展方式。

详细架构规格请参考 [ARCHITECTURE_V4.md](ARCHITECTURE_V4.md)。

---

## 一、架构概览

### 1.1 双产品线

Continuum 提供两个独立产品：

```
┌─────────────────────────────────────────────────────────────────┐
│  continuum-cli (cargo install)                                  │
│  ├── 纯 Rust 二进制                                              │
│  ├── 无 Python 依赖                                             │
│  └── 对标: Claude Code                                          │
├─────────────────────────────────────────────────────────────────┤
│  continuum-sdk (pip install)                                    │
│  ├── Python 接口层                                              │
│  ├── Rust 核心 (wheel 内置)                                      │
│  └── 对标: LangChain / LangGraph                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 技术栈分布

| 层级 | 语言 | 占比 | 说明 |
|------|------|------|------|
| 性能关键路径 | Rust | 70% | 执行引擎、调度、并发安全 |
| 业务逻辑层 | Python | 30% | 工具实现、解析、SDK接口 |

---

## 二、六层架构

### 2.1 分层图

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 5: Interface (产品层)                                      │
│   CLI (TUI)          │  Python SDK (PyO3 绑定)                   │
├─────────────────────────────────────────────────────────────────┤
│ Layer 4: Integration (集成层)                                   │
│   mcp-bridge  plugin-loader  worktree-manager  audit-logger     │
├─────────────────────────────────────────────────────────────────┤
│ Layer 3: Capabilities (能力层)                                  │
│   tool-executor  builtin-tools  memory-system  lsp-client       │
│   query-engine  retriever-engine  vector-store  sandbox        │
├─────────────────────────────────────────────────────────────────┤
│ Layer 2: Core (核心层)                                          │
│   agent-runtime  session-manager  tool-registry  workflow      │
│   checkpoint-system  hook-system  tasks  prompts                │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: Foundation (基础层)                                    │
│   llm-client  embeddings  storage  streaming  config           │
│   event-bus  observability  cost-tracker  cache  error          │
├─────────────────────────────────────────────────────────────────┤
│ Layer 0: Security Gateway (安全网关)                            │
│   input-validator  pii-scrubber  access-controller  rate-limiter│
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 依赖规则

**严格单向依赖：**
```
Layer N → 只依赖 Layer N-1

Layer 5 → Layer 4 → Layer 3 → Layer 2 → Layer 1 → Layer 0

禁止:
❌ 向上依赖 (Layer 2 不能依赖 Layer 3)
❌ 跨层依赖 (Layer 5 不能直接依赖 Layer 2)
❌ 循环依赖
```

---

## 三、各层职责

### Layer 0: Security Gateway

**职责**: 所有外部输入的安全检查

| 模块 | 功能 |
|------|------|
| input-validator | 输入格式验证、JSON Schema、长度限制 |
| pii-scrubber | PII 敏感数据检测和脱敏 |
| access-controller | RBAC 权限检查、审计 |
| rate-limiter | Token Bucket 速率限制 |

**扩展点**: 实现 `SecurityGateway` trait 添加自定义安全规则

### Layer 1: Foundation

**职责**: 通用基础设施

| 模块 | 功能 | 使用示例 |
|------|------|----------|
| llm-client | 多提供商 LLM 客户端 | `LlmClient::new(Provider::Anthropic)` |
| embeddings | 文本嵌入模型 | `embeddings.embed("text")` |
| storage-engine | 文件/对象存储 | `storage.write(path, data)` |
| streaming | SSE/WebSocket 流式处理 | `stream.subscribe(handler)` |
| config-manager | 多环境配置管理 | `config.get("api_key")` |
| event-bus | 发布订阅事件总线 | `bus.publish(event)` |
| observability | Tracing/Metrics/Logs | 自动集成 |
| cost-tracker | Token 计数、费用追踪 | `tracker.get_cost()` |
| cache-manager | LRU/TTL 缓存 | `cache.get_or_insert(key, f)` |
| error-handler | 错误分类、恢复策略 | `Error::recoverable(e)` |

**扩展点**: 实现 `LlmProvider` trait 添加新 LLM 提供商

### Layer 2: Core

**职责**: Agent 运行时核心

| 模块 | 功能 | 使用示例 |
|------|------|----------|
| agent-runtime | Agent 执行循环 | `agent.run(task)` |
| session-manager | 会话生命周期管理 | `session.create()` |
| tool-registry | 工具发现和注册 | `registry.register(tool)` |
| workflow-engine | DAG 工作流调度 | `workflow.execute(plan)` |
| checkpoint-system | 状态快照、断点恢复 | `checkpoint.save(state)` |
| hook-system | 生命周期钩子 | `hook.on_step_complete(cb)` |
| tasks | 任务队列管理 | `tasks.enqueue(task)` |
| prompts | 提示词模板管理 | `prompts.render(name, ctx)` |

**扩展点**: 
- 实现 `Tool` trait 添加自定义工具
- 实现 `Hook` trait 添加生命周期钩子

### Layer 3: Capabilities

**职责**: 特定领域能力

| 模块 | 功能 |
|------|------|
| tool-executor | 工具调用、超时控制 |
| builtin-tools | 40+ 内置工具 (文件、搜索、代码等) |
| memory-system | 分层记忆 (Working/Session/Project/Long-term) |
| query-engine | 代码语义搜索 |
| lsp-client | 语言服务器协议集成 |
| retriever-engine | 相似度搜索、混合检索 |
| vector-store | 向量索引和持久化 |
| sandbox-runtime | 代码沙箱执行 |
| process-manager | 子进程管理 |
| output-parsers | 结构化输出解析 |
| guard-rails | 输出安全约束 |

**扩展点**: 
- 实现 `BuiltinTool` trait 添加内置工具
- 实现 `Retriever` trait 添加检索策略

### Layer 4: Integration

**职责**: 外部系统对接

| 模块 | 功能 |
|------|------|
| mcp-bridge | Model Context Protocol 实现 |
| plugin-loader | 动态插件加载 |
| worktree-manager | Git Worktree 隔离环境 |
| channel-gateway | 多渠道消息路由 |
| audit-logger | 操作审计日志 |

**扩展点**: 实现 `Plugin` trait 开发插件

### Layer 5: Interface

**职责**: 用户界面

| 产品 | 技术 | 入口 |
|------|------|------|
| CLI | Rust + ratatui | `continuum run "task"` |
| Python SDK | PyO3 绑定 | `from continuum import Agent` |

---

## 四、核心数据流

### 4.1 Agent 执行流程

```
用户输入 "修复 auth.py 的 bug"
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 0: Security Gateway                                   │
│   ├── input-validator: 验证输入格式                          │
│   ├── pii-scrubber: 检查敏感数据                             │
│   └── access-controller: 权限检查                           │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Agent Runtime                                      │
│   ├── 解析任务意图                                          │
│   ├── 规划执行步骤 (workflow-engine)                         │
│   └── 开始执行循环                                          │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Tool Executor                                      │
│   ├── read_file: 读取 auth.py                               │
│   ├── grep: 搜索问题关键词                                   │
│   └── edit_file: 修复 bug                                   │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: LLM Client                                         │
│   ├── 调用 LLM API                                          │
│   ├── 流式响应处理                                          │
│   └── 成本追踪                                              │
└─────────────────────────────────────────────────────────────┘
    │
    ▼
输出结果给用户
```

### 4.2 检查点恢复流程

```
崩溃发生 → 重启
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Checkpoint System                                  │
│   ├── 加载最近检查点                                        │
│   ├── 恢复会话状态                                          │
│   └── 从断点继续执行                                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 五、扩展指南

### 5.1 添加自定义工具

```rust
// 1. 实现 Tool trait
pub struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "我的自定义工具" }
    
    async fn execute(&self, args: Value) -> Result<String> {
        // 真实实现
        let input = args["input"].as_str()?;
        Ok(process(input))
    }
}

// 2. 注册工具
registry.register(Box::new(MyTool));
```

### 5.2 添加 LLM 提供商

```rust
// 1. 实现 LlmProvider trait
pub struct MyProvider;

impl LlmProvider for MyProvider {
    async fn complete(&self, messages: &[Message]) -> Result<String> {
        // 调用自定义 API
    }
}

// 2. 注册提供商
llm_client.register_provider("my_provider", MyProvider);
```

### 5.3 添加生命周期钩子

```rust
// 1. 实现 Hook trait
pub struct MyHook;

impl Hook for MyHook {
    fn on_step_complete(&self, step: &Step, result: &Result) {
        // 自定义逻辑
    }
}

// 2. 注册钩子
agent.add_hook(MyHook);
```

---

## 六、与竞品对比

### 6.1 vs Claude Code

| 功能 | Claude Code | Continuum |
|------|-------------|-----------|
| 内置工具 | 40+ | 40+ ✅ |
| 流式输出 | ✅ | ✅ |
| Git 集成 | ✅ | ✅ |
| LSP 支持 | ✅ | ✅ |
| 检查点恢复 | ❌ | ✅ **独特** |
| 任务自检 | ❌ | ✅ **独特** |
| 开源 | ❌ | ✅ **独特** |

### 6.2 vs LangChain

| 功能 | LangChain | Continuum |
|------|-----------|-----------|
| Agent 抽象 | ✅ | ✅ |
| 工具系统 | 丰富生态 | 40+ 内置 |
| Memory | 多后端 | 分层记忆 |
| Workflow | LangGraph | DAG 引擎 |
| 性能 | Python | Rust **更快** |
| 单包安装 | 多依赖 | 单 wheel |

---

## 七、目录结构

```
continuum/
├── rust/
│   ├── layer0/          # Security Gateway
│   ├── layer1/          # Foundation
│   ├── layer2/          # Core
│   ├── layer3/          # Capabilities
│   ├── layer4/          # Integration
│   └── sh-python/       # PyO3 bindings
├── cli/
│   └── src/
│       ├── commands/    # CLI 命令
│       ├── tui/         # TUI 界面
│       └── agent/       # Agent 客户端
├── python/
│   └── continuum_sdk/
│       ├── agent/       # Agent API
│       ├── tools/       # 工具接口
│       ├── llm/         # LLM 客户端
│       ├── memory/      # 记忆系统
│       └── workflow/    # 工作流 API
└── docs/
    ├── ARCHITECTURE_V4.md     # 详细架构规格
    └── ARCHITECTURE_EXPLAINED.md  # 本文档
```

---

## 八、快速开始

### 安装 CLI

```bash
cargo install continuum-cli
continuum run "hello"
```

### 安装 SDK

```bash
pip install continuum-sdk
```

```python
from continuum import Agent

agent = Agent()
result = agent.run("hello")
```

---

## 九、参考资料

- [ARCHITECTURE_V4.md](ARCHITECTURE_V4.md) - 完整架构规格
- [docs/user/quick_start.md](user/quick_start.md) - 用户快速入门
- [examples/](../examples/) - 示例代码

---

**维护者**: T0-new
**最后更新**: 2026-05-24
