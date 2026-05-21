# 多终端并行开发分工指南

> 版本: v1.0
> 适用场景: Continuum 多终端并行开发
> 核心原则: 独立任务、无共享状态、最终合并

---

## 一、核心架构：终端角色分工

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           主控终端 (Terminal 0)                              │
│  职责：架构设计、文档维护、进度跟踪、最终合并、冲突解决                        │
│  文件：docs/*, README.md, Cargo.toml (workspace), 开发进度跟踪                 │
└─────────────────────────────────────────────────────────────────────────────┘
          │                    │                    │                    │
          ▼                    ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ Terminal 1      │  │ Terminal 2      │  │ Terminal 3      │  │ Terminal 4      │
│ Layer 0-1 安全   │  │ Layer 2 核心    │  │ Layer 3 能力    │  │ CLI + SDK      │
│ 基础模块        │  │ 引擎开发        │  │ 扩展开发        │  │ 产品开发        │
└─────────────────┘  └─────────────────┘  └─────────────────┘  └─────────────────┘
```

---

## 二、终端角色详细定义

### Terminal 0: 主控终端（架构师角色）

**核心职责**：
- 维护架构文档和设计规范
- 跟踪整体开发进度
- 解决各终端间的架构问题
- 最终代码合并和冲突解决
- 代码质量审查

**工作文件**：
```
docs/ARCHITECTURE_V4.md
docs/DESIGN_PHILOSOPHY.md
docs/DEVELOPMENT_PROGRESS.md
docs/PROJECT_STRUCTURE.md
Cargo.toml (workspace级别)
README.md
```

**工作流**：
```
1. 启动时：更新进度文档，分配任务
2. 开发中：响应架构问题，更新设计
3. 合并时：审查代码，解决冲突
4. 完成后：更新进度，规划下一步
```

---

### Terminal 1: 安全基础层开发者

**负责层级**：Layer 0 + Layer 1

**工作目录**：
```
rust/layer0/
rust/layer1/
tests/integration/test_layer01.rs
```

**任务队列**：
```
Phase 1 (已完成):
├── input_validator
├── pii_scrubber
├── access_controller
├── rate_limiter
├── llm_client
├── cost_tracker
└── 其他基础模块

Phase 2 (待开发):
├── secrets-manager (新增安全模块)
├── encryption-engine (新增加密模块)
└── 扩展测试覆盖
```

**输出物**：
- Layer 0/1 的 Rust 实现
- 单元测试
- 模块文档注释

---

### Terminal 2: 核心引擎开发者

**负责层级**：Layer 2

**工作目录**：
```
rust/layer2/
rust/layer2/src/
    ├── agent_runtime.rs
    ├── session_manager/
    ├── checkpoint_system/
    ├── tool_registry.rs
    ├── workflow_engine/
    └── tests/
tests/integration/test_layer2.rs
```

**任务队列**：
```
Priority 1 - 迁移Python实现:
├── session_manager (迁移 session_concurrency.py)
├── checkpoint_system (迁移 checkpoint_writer.py)
└── 添加 Rust 并发测试

Priority 2 - 新开发:
├── agent_runtime (Agent执行循环)
├── tool_registry (工具注册发现)
├── workflow_engine (DAG调度)
├── hook_system (生命周期钩子)
├── tasks (任务队列)
└── prompts (提示词管理)
```

**依赖关系**：
```
Layer 2 依赖 Layer 0 + Layer 1
→ Terminal 2 需要等 Terminal 1 完成基础模块后才能开始核心逻辑
→ 可以先开发接口定义，后实现具体逻辑
```

---

### Terminal 3: 能力扩展开发者

**负责层级**：Layer 3

**工作目录**：
```
rust/layer3/
rust/layer3/src/
    ├── tool_executor.rs
    ├── builtin_tools/
    ├── memory_system/
    ├── query_engine.rs
    ├── process_manager.rs
    ├── lsp_client.rs
    └── tests/
tests/integration/test_layer3.rs
```

**任务队列**：
```
Priority 1 - 核心能力:
├── tool_executor (工具执行器)
├── builtin_tools/ (40+工具)
│   ├── file_ops.rs
│   ├── search.rs
│   ├── bash.rs
│   └── ...
├── memory_system/ (分层记忆)
│   ├── working.rs
│   ├── session.rs
│   ├── project.rs
│   └── long_term.rs
└── query_engine (代码查询)

Priority 2 - 扩展能力:
├── process_manager (进程管理)
├── sandbox_runtime (沙箱)
├── lsp_client (LSP客户端)
├── retriever_engine (检索)
├── vector_store (向量存储)
└── document_loaders/ (文档加载)
```

**依赖关系**：
```
Layer 3 依赖 Layer 2
→ Terminal 3 需要 Terminal 2 提供 agent_runtime 和 tool_registry 接口
→ 可以先定义 builtin_tools 接口，并行开发各工具
```

---

### Terminal 4: 产品接口开发者

**负责层级**：Layer 4 + Layer 5

**工作目录**：
```
rust/layer4/
rust/sh-core/src/lib.rs (Python绑定)
cli/
python/continuum_sdk/
tests/e2e/
```

**任务队列**：
```
Priority 1 - CLI产品:
├── cli/src/commands/ (子命令实现)
├── cli/src/tui/ (TUI界面)
├── cli/src/output/ (输出处理)
└── cli/tests/ (集成测试)

Priority 2 - Python SDK:
├── python/continuum_sdk/agent/
├── python/continuum_sdk/tools/
├── python/continuum_sdk/workflow/
└── python/tests/

Priority 3 - 集成层:
├── mcp_bridge (MCP协议)
├── plugin_loader (插件加载)
├── worktree_manager (工作树)
└── channel_gateway (渠道网关)
```

---

## 三、并行开发时间线

### 第一阶段：基础设施（可完全并行）

```
时间: Day 1-3

Terminal 1: Layer 0 + Layer 1 全部模块
Terminal 2: 定义 Layer 2 所有接口（trait定义）
Terminal 3: 定义 Layer 3 所有接口
Terminal 4: CLI 框架搭建，命令解析
Terminal 0: 文档维护，进度跟踪

关键点：所有接口定义在第一天完成，后续实现可并行
```

### 第二阶段：核心实现（有依赖关系）

```
时间: Day 4-7

Terminal 1: 完善测试，修复 bug
Terminal 2: 实现 Layer 2 核心逻辑
          ↓ 等待 Terminal 2 完成
Terminal 3: 开始 Layer 3 实现
Terminal 4: CLI TUI 开发
Terminal 0: 代码审查，合并 Terminal 1 成果

关键点：Terminal 2 完成后，Terminal 3 才能真正开始实现
```

### 第三阶段：产品集成

```
时间: Day 8-10

Terminal 1: 支持其他终端的 bug 修复
Terminal 2: 支持 Terminal 3 的集成需求
Terminal 3: 完成 Layer 3 所有模块
Terminal 4: Python SDK + CLI 最终集成
Terminal 0: 最终合并，解决冲突，发布

关键点：所有代码合并到主分支
```

---

## 四、代码隔离策略

### 文件级隔离（推荐）

```
原则：每个终端独占一组文件，不跨终端修改

Terminal 0: docs/*, README.md, Cargo.toml
Terminal 1: rust/layer0/*, rust/layer1/*
Terminal 2: rust/layer2/*
Terminal 3: rust/layer3/*
Terminal 4: rust/layer4/*, rust/sh-core/*, cli/*, python/*
```

### 接口先行策略

```rust
// Terminal 2 先定义接口（Day 1）
// 文件: rust/layer2/src/session_manager.rs

/// 会话管理器接口
pub trait SessionManagerTrait {
    fn create_session(&self) -> Result<Session>;
    fn get_session(&self, id: &str) -> Result<Option<Session>>;
    fn save_session(&self, session: &Session) -> Result<()>;
    fn delete_session(&self, id: &str) -> Result<()>;
}

// Terminal 3 可以立即开始使用这个接口
// 不需要等 Terminal 2 实现具体逻辑
```

### 共享类型定义

```
创建专门的 types crate 或使用 layer0 定义共享类型

rust/layer0/src/types.rs:
├── SessionId
├── AgentId
├── TaskId
├── Result<T>
└── Error

所有终端共享这些类型，避免重复定义
```

---

## 五、同步与通信机制

### 每日同步点

```
时间: 每天固定时间（如 9:00, 14:00, 19:00）

流程:
1. 各终端报告完成情况
2. 主控终端更新进度文档
3. 讨论架构问题
4. 分配下一阶段任务

输出: 更新 docs/DEVELOPMENT_PROGRESS.md
```

### Git 分支策略

```
main                    # 稳定发布
├── develop             # 开发集成
│   ├── feature/layer0  # Terminal 1
│   ├── feature/layer1  # Terminal 1
│   ├── feature/layer2  # Terminal 2
│   ├── feature/layer3  # Terminal 3
│   └── feature/cli     # Terminal 4
└── docs                # Terminal 0
```

### 合并时间窗口

```
Terminal 0 负责:
├── 每4小时合并一次 feature 分支到 develop
├── 解决合并冲突
├── 运行测试确保不破坏
└── 通知相关终端冲突情况
```

---

## 六、质量保证流程

### 自检清单（每个终端提交前）

```
□ clippy 无警告
□ fmt 格式化通过
□ 单元测试通过
□ 文档注释完整
□ 无 TODO/unimplemented!
□ 文件不超过 500 行
```

### 代码审查流程

```
Terminal 0 审查流程:

1. 检查架构符合性
   - 是否遵守层级依赖
   - 是否符合设计规范

2. 检查代码质量
   - 单文件单职责
   - 无创世文件
   - 接口最小化

3. 检查测试覆盖
   - 单元测试存在
   - 边界条件测试

4. 合并或退回
   - 通过：合并到 develop
   - 不通过：说明问题，退回修改
```

---

## 七、具体分工方案

### 当前阶段（Layer 0-1 完成）

```
立即开始:

Terminal 1:
├── 完善 Layer 0-1 测试覆盖
├── 修复潜在 bug
└── 开始设计 Layer 2 接口（等待 Terminal 2）

Terminal 2:
├── 定义所有 Layer 2 trait 接口
├── 设计 session_manager 结构
├── 设计 checkpoint_system 结构
└── 规划迁移 Python 代码策略

Terminal 3:
├── 定义所有 Layer 3 trait 接口
├── 设计 builtin_tools 接口
├── 规划 40+ 工具分类
└── 设计 memory_system 结构

Terminal 4:
├── 完成 CLI 子命令框架
├── 设计 TUI 界面原型
├── 规划 Python SDK API
└── 搭建测试框架

Terminal 0:
├── 维护进度文档
├── 协调各终端进度
└── 准备代码审查流程
```

### 下一阶段（Layer 2 开发）

```
Terminal 1:
├── 支持其他终端的问题修复
└── 开发新增安全模块（secrets-manager）

Terminal 2:
├── 实现 session_manager（迁移 Python）
├── 实现 checkpoint_system（迁移 Python）
├── 实现 agent_runtime
├── 实现 workflow_engine
└── 实现 tool_registry

Terminal 3:
├── 等待 Terminal 2 接口稳定
├── 先行开发 builtin_tools（工具实现不依赖 Layer 2）
└── 准备 memory_system 实现

Terminal 4:
├── CLI 基本功能开发
├── 简单 Python SDK 绑定
└── 准备集成测试

Terminal 0:
├── 定期合并
├── 解决冲突
└── 更新进度
```

---

## 八、效率最大化技巧

### 任务拆分原则

```
✅ 好的任务拆分:
├── 单个文件或紧密相关的一组文件
├── 完成时间 < 4小时
├── 无外部依赖或依赖已就绪
└── 明确的输入输出

❌ 不好的任务拆分:
├── 涉及多个不相关文件
├── 完成时间不确定
├── 强依赖其他终端当前正在开发的功能
└── 边界不清晰
```

### 阻塞处理

```
当被阻塞时:

1. 立即通知主控终端
2. 切换到备用任务（每个终端应准备2-3个备选）
3. 继续推进不依赖阻塞项的工作
4. 等待阻塞解除后继续

示例:
Terminal 3 被 Terminal 2 的 agent_runtime 阻塞
→ 切换到开发 builtin_tools（不依赖 agent_runtime）
→ 或完善接口定义
→ 或编写测试用例
```

### 工作量估算

```
每个终端每日工作量基准:

Rust 核心模块: ~300-500 行高质量代码/天
接口定义: ~10-15 个 trait/天
测试编写: ~20-30 个测试用例/天
文档编写: ~1000-2000 行/天
代码审查: ~1000-2000 行/天

Continuum 总工作量估算:
├── Rust 代码: ~15000 行
├── 测试代码: ~5000 行
├── 文档: ~5000 行
├── 合计: ~25000 行
└── 5终端并行: ~10-15 天完成
```

---

## 九、输出物清单

### 每日输出

```
Terminal 0:
├── 更新进度文档
├── 合并代码
└── 问题记录

Terminal 1-4:
├── 功能代码
├── 单元测试
├── 文档注释
└── 自检报告
```

### 阶段输出

```
Phase 1 完成:
├── Layer 0-1 全部代码 + 测试
├── 所有接口定义
├── CLI 框架
└── 进度文档

Phase 2 完成:
├── Layer 2 全部代码 + 测试
├── Layer 3 部分代码
├── CLI 基本功能
└── 进度文档

Phase 3 完成:
├── 全部代码 + 测试
├── CLI 产品可用
├── Python SDK 可用
├── 完整文档
└── 发布准备
```

---

## 十、总结

### 成功要素

```
1. 接口先行 - 第一天定义所有接口，后续并行实现
2. 文件隔离 - 每个终端独占文件，避免冲突
3. 定期同步 - 每日固定时间同步进度
4. 主控协调 - Terminal 0 负责合并和冲突解决
5. 质量优先 - 不赶进度，保证代码质量
```

### 一句话原则

> **每个终端专注自己的层级，通过接口通信，主控终端负责协调合并。**

---

**文档状态**: v1.0 完成
**适用项目**: Continuum 多终端并行开发
