# Continuum

[![CI](https://github.com/EnjouZeratul/continuum/actions/workflows/ci.yml/badge.svg)](https://github.com/EnjouZeratul/continuum/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10+-blue.svg)](https://www.python.org/)
[![crates.io](https://img.shields.io/crates/v/continuum.svg)](https://crates.io/crates/continuum)
[![PyPI](https://img.shields.io/pypi/v/continuum-agent-sdk.svg)](https://pypi.org/project/continuum-agent-sdk/)

**Continuum is a concise and reliable Agent runtime.**

Rust core performance + Python friendly interface + full Agent capabilities

[中文版](#中文版)

---

## Product Positioning

Continuum is two equally important products:

| Product | Description | Analogy |
|---------|-------------|---------|
| **CLI** | Terminal Agent product | Claude Code + Aider + OpenClaw CLI |
| **Python SDK** | AI application development framework | LangChain + LangGraph + SmolAgents |

---

## Core Features

### Security First
- Layer 0 Security Gateway: All external inputs must be validated
- PII data scrubbing, access control, rate limiting

### Performance Core
- Rust core engine (70% modules)
- Concurrent-safe session management
- Efficient checkpoint system

### Developer Friendly
- Python thin-layer interface, simple and intuitive
- Complete type hints
- Comprehensive documentation

### Observability
- Built-in cost tracking
- Real-time Token statistics
- Execution process replay

---

## Quick Start

### Environment Setup (Recommended)

```bash
# Create conda environment (reproducible)
conda env create -f environment.yml
conda activate continuum

# Or use pip directly
pip install -e ./python[dev]
```

### CLI Usage

```bash
# Install
cargo install continuum

# Run a task
continuum run "Analyze this project structure"

# Start TUI mode
continuum tui

# Manage sessions
continuum session list
continuum session resume <session_id>
```

### Python SDK Usage

```python
from continuum_sdk import Agent, Session

# Create Agent (auto-configures from environment)
agent = Agent()

# Run a task
result = agent.run("Help me refactor this function")

# With explicit configuration
agent = Agent(model="claude-sonnet-4-6", provider="anthropic")

# Session management
session = Session()
session.add_user_message("Hello")
session.save("my_session")
session = Session.load("my_session")
```

---

## Architecture

### Six-Layer Architecture

```
Layer 5: Interface     → CLI + Python SDK
Layer 4: Integration   → MCP, Plugin, Worktree
Layer 3: Capabilities  → Tools, Memory, Query Engine
Layer 2: Core          → Agent Runtime, Session, Checkpoint
Layer 1: Foundation    → LLM Client, Storage, Cost Tracker
Layer 0: Security      → Input Validator, PII Scrubber, Access Control
```

### Dependency Rules

- Layer N can only depend on Layer N-1
- No circular dependencies
- Security gateway is the entry point for all external inputs

See [ARCHITECTURE_V4.md](docs/ARCHITECTURE_V4.md) for details.

---

## Project Structure

```
continuum/
├── rust/              # Rust core code
│   ├── layer0/       # Security Gateway
│   ├── layer1/       # Foundation
│   ├── layer2/       # Core Engine
│   ├── layer3/       # Capabilities
│   ├── layer4/       # Integration
│   ├── sh-core/      # Core crate (pure Rust)
│   └── sh-python/    # Python bindings
├── cli/              # CLI product
├── python/           # Python SDK
├── docs/             # Documentation
└── tests/            # Tests
```

See [PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) for details.

---

## Development

### Build

```bash
# Activate environment
conda activate continuum

# Build Rust core
cargo build --release

# Build Python package
maturin develop

# Run tests
cargo test
pytest
```

### Code Standards

See [DESIGN_PHILOSOPHY.md](docs/DESIGN_PHILOSOPHY.md)

Core principles:
- Single file, single responsibility
- No genesis files
- Reuse reusable interfaces

---

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE_V4.md](docs/ARCHITECTURE_V4.md) | Complete architecture design |
| [PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) | Project directory structure |
| [DESIGN_PHILOSOPHY.md](docs/DESIGN_PHILOSOPHY.md) | Design philosophy and code standards |
| [SUPER_PROJECT_VISION.md](docs/SUPER_PROJECT_VISION.md) | Project vision |
| [DIFFERENTIATION_STRATEGY.md](docs/DIFFERENTIATION_STRATEGY.md) | Differentiation strategy |

---

## Competitive Benchmark

### Claude Code
- ✅ 40+ built-in tools
- ✅ Query Engine
- ✅ LSP Client
- ✅ Worktree Manager
- ✅ Cost Tracking
- ✅ Streaming output
- ✅ Session persistence

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

## Package Registration

Package names have been reserved to prevent squatting:

| Platform | Package Name | Version | Status |
|----------|--------------|---------|--------|
| crates.io | [`continuum`](https://crates.io/crates/continuum) | v0.1.0 | ✅ Reserved |
| PyPI | [`continuum-agent-sdk`](https://pypi.org/project/continuum-agent-sdk/) | v1.0.0 | ✅ Reserved |

```bash
# Install Python SDK
pip install continuum-agent-sdk

# Add Rust dependency
cargo add continuum
```

> Note: These are placeholder packages. Full implementation will be released soon.

---

## Development Status

🚧 **In Development**

Current progress:
- [x] Architecture design
- [x] Layer 0 Security Gateway
- [x] Layer 1 Foundation modules
- [ ] Layer 2 Core Engine
- [ ] Layer 3 Capabilities
- [ ] Layer 4 Integration
- [ ] CLI product
- [ ] Python SDK

---

## License

MIT

---

<h2 id="中文版">中文版</h2>

<details>
<summary>点击展开中文版</summary>

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
from continuum_sdk import Agent, Session

# 创建 Agent (自动从环境配置)
agent = Agent()

# 运行任务
result = agent.run("帮我重构这个函数")

# 指定配置
agent = Agent(model="claude-sonnet-4-6", provider="anthropic")

# 会话管理
session = Session()
session.add_user_message("你好")
session.save("my_session")
session = Session.load("my_session")
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

## 包名注册

包名已预留，防止被抢注：

| 平台 | 包名 | 版本 | 状态 |
|------|------|------|------|
| crates.io | [`continuum`](https://crates.io/crates/continuum) | v0.1.0 | ✅ 已注册 |
| PyPI | [`continuum-agent-sdk`](https://pypi.org/project/continuum-agent-sdk/) | v1.0.0 | ✅ 已注册 |

```bash
# 安装 Python SDK
pip install continuum-agent-sdk

# 添加 Rust 依赖
cargo add continuum
```

> 注：当前为占位包，完整实现将在后续版本发布。

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

</details>