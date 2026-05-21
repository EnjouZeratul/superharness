# Continuum

[![CI](https://github.com/continuum-ai/continuum/actions/workflows/ci.yml/badge.svg)](https://github.com/continuum-ai/continuum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10+-blue.svg)](https://www.python.org/)

**Continuum 是简洁可靠的 Agent 运行时。**

Rust 核心性能 + Python 友好接口 + 完整 Agent 能力

---

## 产品定位

Continuum 是两个同等重要的产品：

| 产品 | 描述 | 类比 |
|------|------|------|
| **CLI** | 终端 Agent 产品 | Claude Code + Aider + OpenClaw CLI |
| **Python SDK** | AI 应用开发框架 | LangChain + LangGraph + SmolAgents |

---

## 核心特性

### 安全优先
- Layer 0 安全网关：所有外部输入必须经过验证
- PII 数据清洗、访问控制、速率限制

### 性能核心
- Rust 实现核心引擎（70% 模块）
- 并发安全的会话管理
- 高效的检查点系统

### 开发友好
- Python 薄层接口，简洁易用
- 完整的类型提示
- 详尽的文档

### 可观测性
- 内置成本追踪
- 实时 Token 统计
- 执行过程回放

---

## 快速开始

### CLI 使用

```bash
# 安装
cargo install continuum

# 运行任务
continuum run "分析这个项目结构"

# 启动 TUI 模式
continuum tui

# 管理会话
continuum session list
continuum session resume <session_id>
```

### Python SDK 使用

```python
from continuum import Agent, SessionManager

# 创建 Agent
agent = Agent(
    model="claude-sonnet-4-6",
    tools=["file_read", "file_write", "bash"]
)

# 运行任务
result = await agent.run("帮我重构这个函数")

# 会话管理
session = SessionManager()
session.save("my_session")
session.load("my_session")
```

---

## 架构

### 六层架构

```
Layer 5: Interface     → CLI + Python SDK
Layer 4: Integration   → MCP, Plugin, Worktree
Layer 3: Capabilities  → Tools, Memory, Query Engine
Layer 2: Core          → Agent Runtime, Session, Checkpoint
Layer 1: Foundation    → LLM Client, Storage, Cost Tracker
Layer 0: Security      → Input Validator, PII Scrubber, Access Control
```

### 依赖规则

- Layer N 只能依赖 Layer N-1
- 无循环依赖
- 安全网关是所有外部输入的入口

详细架构请参阅 [ARCHITECTURE_V4.md](docs/ARCHITECTURE_V4.md)

---

## 项目结构

```
continuum/
├── rust/              # Rust 核心代码
│   ├── layer0/       # Security Gateway
│   ├── layer1/       # Foundation
│   ├── layer2/       # Core Engine
│   ├── layer3/       # Capabilities
│   ├── layer4/       # Integration
│   ├── sh-core/      # 核心 crate (纯 Rust)
│   └── sh-python/    # Python 绑定
├── cli/              # CLI 产品
├── python/           # Python SDK
├── docs/             # 文档
└── tests/            # 测试
```

详细结构请参阅 [PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md)

---

## 开发

### 构建

```bash
# 构建 Rust 核心
cargo build --release

# 构建 Python 包
maturin develop

# 运行测试
cargo test
pytest
```

### 代码规范

请参阅 [DESIGN_PHILOSOPHY.md](docs/DESIGN_PHILOSOPHY.md)

核心原则：
- 单文件单职责
- 无创世文件
- 复用可复用接口

---

## 文档

| 文档 | 描述 |
|------|------|
| [ARCHITECTURE_V4.md](docs/ARCHITECTURE_V4.md) | 完整架构设计 |
| [PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) | 项目目录结构 |
| [DESIGN_PHILOSOPHY.md](docs/DESIGN_PHILOSOPHY.md) | 设计理念与代码规范 |
| [SUPER_PROJECT_VISION.md](docs/SUPER_PROJECT_VISION.md) | 项目愿景 |
| [DIFFERENTIATION_STRATEGY.md](docs/DIFFERENTIATION_STRATEGY.md) | 差异化竞争策略 |

---

## 竞品能力对标

### Claude Code
- ✅ 40+ 内置工具
- ✅ Query Engine
- ✅ LSP Client
- ✅ Worktree Manager
- ✅ Cost Tracking
- ✅ 流式输出
- ✅ 会话持久化

### OpenClaw
- ✅ Plugin SDK
- ✅ Channel Gateway
- ✅ Process Management
- ✅ Sandbox Runtime
- ✅ Audit Logger

### LangChain/LangGraph
- ✅ Workflow Engine
- ✅ Memory System
- ✅ Tool Registry
- ✅ Retrieval Engine
- ✅ Output Parsers

---

## 开发状态

🚧 **正在开发中**

当前进度：
- [x] 架构设计
- [x] Layer 0 安全网关
- [x] Layer 1 基础模块
- [ ] Layer 2 核心引擎
- [ ] Layer 3 能力扩展
- [ ] Layer 4 集成模块
- [ ] CLI 产品
- [ ] Python SDK

---

## License

MIT
