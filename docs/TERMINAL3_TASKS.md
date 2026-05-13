# Terminal 3 任务清单

> 分配时间: 2026-05-10
> 负责层级: Layer 3 (Capabilities)
> 角色: 能力扩展开发者

---

## 当前状态

**已完成模块**: 0/15

### Layer 3 待开发模块:
- [ ] tool_executor
- [ ] builtin_tools (40+工具)
- [ ] skills
- [ ] memory_system
- [ ] retriever_engine
- [ ] query_engine
- [ ] output_parsers
- [ ] guard_rails
- [ ] example_selectors
- [ ] process_manager
- [ ] sandbox_runtime
- [ ] lsp_client
- [ ] document_loaders
- [ ] text_splitters
- [ ] vector_store

---

## 当前任务

### 任务 3.1: 定义所有 Layer 3 接口

**优先级**: P0
**预计时间**: 1-2小时
**可立即开始**: ✅

```rust
// 文件: rust/layer3/src/lib.rs

/// 工具执行器接口
pub trait ToolExecutorTrait {
    async fn execute(&self, tool: &str, args: &Value) -> Result<ToolResult>;
    async fn execute_stream(&self, tool: &str, args: &Value) -> Result<impl Stream<Item = Result<String>>>;
}

/// 内置工具接口
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    async fn execute(&self, args: &Value) -> Result<ToolResult>;
}

/// 记忆系统接口
pub trait MemorySystemTrait {
    async fn remember(&self, content: &str, level: MemoryLevel) -> Result<()>;
    async fn recall(&self, query: &str) -> Result<Vec<Memory>>;
    async fn forget(&self, id: &MemoryId) -> Result<()>;
}

pub enum MemoryLevel {
    Working,
    Session,
    Project,
    LongTerm,
}

/// 查询引擎接口 (Claude Code 核心)
pub trait QueryEngineTrait {
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
    async fn semantic_search(&self, query: &str) -> Result<Vec<SearchResult>>;
    async fn code_search(&self, pattern: &str) -> Result<Vec<CodeResult>>;
}

/// 进程管理接口 (OpenClaw 风格)
pub trait ProcessManagerTrait {
    async fn spawn(&self, cmd: &str, args: &[&str]) -> Result<ProcessHandle>;
    async fn kill(&self, pid: u32) -> Result<()>;
    async fn wait(&self, handle: &ProcessHandle) -> Result<ExitStatus>;
    async fn output(&self, handle: &ProcessHandle) -> Result<String>;
}

/// 沙箱运行时接口
pub trait SandboxRuntimeTrait {
    async fn execute(&self, code: &str, lang: &str) -> Result<ExecutionResult>;
    fn set_timeout(&mut self, secs: u64);
    fn set_memory_limit(&mut self, bytes: u64);
}

/// LSP 客户端接口
pub trait LspClientTrait {
    async fn initialize(&self) -> Result<()>;
    async fn goto_definition(&self, uri: &str, position: Position) -> Result<Option<Location>>;
    async fn find_references(&self, uri: &str, position: Position) -> Result<Vec<Location>>;
    async fn hover(&self, uri: &str, position: Position) -> Result<Option<Hover>>;
    async fn document_symbols(&self, uri: &str) -> Result<Vec<Symbol>>;
}

/// 向量存储接口
pub trait VectorStoreTrait {
    async fn insert(&self, id: &str, vector: &[f32], metadata: Value) -> Result<()>;
    async fn search(&self, vector: &[f32], k: usize) -> Result<Vec<SearchResult>>;
    async fn delete(&self, id: &str) -> Result<()>;
}
```

---

### 任务 3.2: 实现 builtin_tools 接口和基础工具

**优先级**: P0
**预计时间**: 4-5小时
**可立即开始**: ✅ (不依赖 Layer 2)

```
目录结构:
rust/layer3/src/builtin_tools/
├── mod.rs           # 工具注册和导出
├── file_ops.rs      # 文件操作
├── search.rs        # 搜索工具
├── bash.rs          # Shell 执行
├── code.rs          # 代码操作
├── web.rs           # Web 请求
└── memory.rs        # 记忆操作
```

**必须实现的工具（优先）**:
```rust
// 文件操作
- file_read(path) -> String
- file_write(path, content) -> ()
- file_append(path, content) -> ()
- file_delete(path) -> ()
- file_exists(path) -> bool
- file_list(dir, pattern) -> Vec<String>

// 搜索
- grep(pattern, path) -> Vec<Match>
- glob(pattern) -> Vec<String>
- search_semantic(query) -> Vec<Result>

// Shell
- bash(command) -> (stdout, stderr, exit_code)
- bash_stream(command) -> Stream<String>

// 代码
- lsp_goto_definition(file, line, col) -> Location
- lsp_find_references(file, line, col) -> Vec<Location>
- lsp_hover(file, line, col) -> HoverInfo
```

---

### 任务 3.3: 实现 memory_system

**优先级**: P1
**预计时间**: 4-5小时
**依赖**: Terminal 2 的 session_manager 接口

```
目录结构:
rust/layer3/src/memory_system/
├── mod.rs
├── system.rs        # MemorySystem 主结构
├── working.rs       # 工作记忆 (有限窗口)
├── session.rs       # 会话记忆
├── project.rs       # 项目记忆 (类似 EGG.md)
└── long_term.rs     # 长期记忆 (向量存储)
```

关键实现:
```rust
pub struct MemorySystem {
    working: WorkingMemory,
    session: SessionMemory,
    project: ProjectMemory,
    long_term: LongTermMemory,
}

impl MemorySystem {
    pub async fn remember(&self, content: &str, level: MemoryLevel) -> Result<()> {
        match level {
            MemoryLevel::Working => self.working.add(content),
            MemoryLevel::Session => self.session.add(content).await,
            MemoryLevel::Project => self.project.append(content).await,
            MemoryLevel::LongTerm => self.long_term.insert(content).await,
        }
    }

    pub async fn recall(&self, query: &str) -> Result<Vec<Memory>> {
        let mut results = Vec::new();
        results.extend(self.working.search(query));
        results.extend(self.session.search(query).await);
        results.extend(self.project.search(query).await);
        results.extend(self.long_term.search(query).await);
        Ok(results)
    }
}
```

---

### 任务 3.4: 实现 query_engine

**优先级**: P1
**预计时间**: 4-5小时
**依赖**: Terminal 2 的 agent_runtime 接口

```
功能 (Claude Code 核心):
├── 代码搜索 (grep + 语义)
├── 文件搜索
├── 符号搜索 (LSP)
├── 引用跳转
└── 智能排序

目录结构:
rust/layer3/src/query_engine/
├── mod.rs
├── engine.rs        # QueryEngine 实现
├── code_search.rs   # 代码搜索
├── file_search.rs   # 文件搜索
├── symbol_search.rs # 符号搜索
└── ranker.rs        # 结果排序
```

---

### 任务 3.5: 实现 process_manager

**优先级**: P2
**预计时间**: 3-4小时

```
功能 (OpenClaw 风格):
├── 子进程管理
├── 资源隔离
├── 流式输出
├── 超时控制
└── 清理机制

目录结构:
rust/layer3/src/process_manager/
├── mod.rs
├── manager.rs       # ProcessManager
├── handle.rs        # ProcessHandle
└── stream.rs        # 输出流处理
```

---

### 任务 3.6: 实现 lsp_client

**优先级**: P2
**预计时间**: 4-5小时

```
功能:
├── LSP 协议实现
├── 多语言服务器支持
├── 请求/响应处理
└── 连接管理

目录结构:
rust/layer3/src/lsp_client/
├── mod.rs
├── client.rs        # LspClient
├── protocol.rs      # LSP 协议
├── transport.rs     # 传输层
└── capabilities.rs  # 能力协商
```

---

### 任务 3.7: 实现其他模块

**优先级**: P3
**预计时间**: 各 2-4小时

```
├── tool_executor: 工具执行器
├── skills: Skills 系统
├── retriever_engine: 检索引擎
├── output_parsers: 输出解析
├── guard_rails: 边界约束
├── example_selectors: 示例选择
├── sandbox_runtime: 沙箱运行时
├── document_loaders: 文档加载
├── text_splitters: 文本分割
└── vector_store: 向量存储
```

---

## 工作目录

```
rust/layer3/src/
rust/layer3/src/builtin_tools/
rust/layer3/src/memory_system/
rust/layer3/src/query_engine/
rust/layer3/src/process_manager/
rust/layer3/src/lsp_client/
tests/integration/test_layer3.rs
```

## 依赖关系

```
任务依赖图:

3.1 (接口定义) ──→ 可立即开始 3.2 (builtin_tools)

等待 Terminal 2:
├── 3.3 (memory_system) 等 session_manager 接口
├── 3.4 (query_engine) 等 agent_runtime 接口
└── 其他模块等相应接口
```

## 自检清单

```
□ cargo clippy 无警告
□ cargo fmt 通过
□ cargo test 通过
□ 所有 trait 有文档注释
□ 工具有使用示例
```

## 完成标准

- 所有 trait 定义完整
- 基础工具集（文件、搜索、Shell）完成
- memory_system 框架完成
- 单元测试覆盖

---

## 注意事项

1. **接口定义优先（任务 3.1）**
2. **builtin_tools 可独立开发**
3. **记忆系统等 Terminal 2 接口**
4. **每个工具需要独立测试**
