# Continuum 项目目录结构

> 版本: v1.0
> 日期: 2026-05-10

---

## 一、顶层目录结构

```
continuum/
├── rust/                      # Rust 核心代码
│   ├── Cargo.toml            # Workspace 配置
│   ├── Cargo.lock            # 依赖锁定
│   ├── layer0/               # Security Gateway
│   ├── layer1/               # Foundation
│   ├── layer2/               # Core
│   ├── layer3/               # Capabilities
│   ├── layer4/               # Integration
│   └── sh-core/              # 核心 crate（汇总）
│
├── python/                    # Python SDK 代码
│   ├── pyproject.toml        # Python 包配置
│   ├── continuum_sdk/     # SDK 源码
│   └── tests/                # Python 测试
│
├── cli/                       # CLI 产品
│   ├── Cargo.toml            # CLI crate 配置
│   ├── src/                  # CLI 源码
│   └── tests/                # CLI 测试
│
├── docs/                      # 文档
│   ├── ARCHITECTURE_V4.md    # 架构设计
│   ├── PROJECT_STRUCTURE.md # 本文档
│   └── ...                   # 其他文档
│
├── tests/                     # 集成测试
│   ├── integration/          # 集成测试
│   ├── e2e/                  # 端到端测试
│   └── benchmarks/           # 性能测试
│
├── examples/                  # 示例代码
│   ├── basic/                # 基础示例
│   ├── advanced/             # 高级示例
│   └── plugins/              # 插件示例
│
├── tools/                     # 开发工具
│   ├── scripts/              # 脚本
│   └── generators/           # 代码生成器
│
├── .github/                   # GitHub 配置
│   ├── workflows/            # CI/CD
│   └── ISSUE_TEMPLATE/       # Issue 模板
│
├── Cargo.toml                 # Workspace 根配置
├── pyproject.toml             # Python 根配置
├── README.md                  # 项目说明
└── LICENSE                    # 许可证
```

---

## 二、Rust 核心目录结构

```
rust/
├── Cargo.toml                          # Workspace 配置
│
├── layer0/                             # Security Gateway (4模块)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── input_validator.rs         # 输入验证
│       ├── pii_scrubber.rs            # PII清洗
│       ├── access_controller.rs       # 访问控制
│       └── rate_limiter.rs            # 速率限制
│
├── layer1/                             # Foundation (10模块)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── llm_client.rs              # LLM客户端
│       ├── embeddings.rs              # 嵌入模型
│       ├── storage_engine.rs          # 存储引擎
│       ├── streaming.rs               # 流式处理
│       ├── config_manager.rs          # 配置管理
│       ├── event_bus.rs               # 事件总线
│       ├── observability.rs           # 可观测性
│       ├── cost_tracker.rs            # 成本追踪
│       ├── cache_manager.rs           # 缓存管理
│       └── error_handler.rs           # 错误处理
│
├── layer2/                             # Core (8模块)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── agent_runtime.rs           # Agent运行时
│       ├── session_manager.rs         # 会话管理
│       ├── tool_registry.rs           # 工具注册
│       ├── workflow_engine.rs         # 工作流引擎
│       ├── hook_system.rs             # Hook系统
│       ├── checkpoint_system.rs       # 检查点系统
│       ├── tasks.rs                    # 任务管理
│       └── prompts.rs                  # 提示词管理
│
├── layer3/                             # Capabilities (15模块)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── tool_executor.rs           # 工具执行器
│       ├── builtin_tools/             # 内置工具集
│       │   ├── mod.rs
│       │   ├── file_ops.rs
│       │   ├── search.rs
│       │   ├── code.rs
│       │   └── ...
│       ├── skills.rs                  # Skills系统
│       ├── memory_system/             # 分层记忆
│       │   ├── mod.rs
│       │   ├── working.rs
│       │   ├── session.rs
│       │   ├── project.rs
│       │   └── long_term.rs
│       ├── retriever_engine.rs        # 检索引擎
│       ├── query_engine.rs            # 查询引擎
│       ├── output_parsers.rs          # 输出解析
│       ├── guard_rails.rs             # 边界约束
│       ├── example_selectors.rs       # 示例选择
│       ├── process_manager.rs        # 进程管理
│       ├── sandbox_runtime.rs        # 沙箱运行时
│       ├── lsp_client.rs             # LSP客户端
│       ├── document_loaders/         # 文档加载器
│       │   ├── mod.rs
│       │   ├── pdf.rs
│       │   ├── markdown.rs
│       │   └── ...
│       ├── text_splitters.rs         # 文本分割器
│       └── vector_store.rs           # 向量存储
│
├── layer4/                             # Integration (5模块)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── channel_gateway.rs        # 渠道网关
│       ├── plugin_loader.rs          # 插件加载器
│       ├── mcp_bridge.rs             # MCP桥接
│       ├── worktree_manager.rs       # 工作树管理
│       └── audit_logger.rs           # 审计日志
│
└── sh-core/                            # 核心 crate（汇总导出）
    ├── Cargo.toml
    └── src/
        └── lib.rs                     # 重导出所有 layer 模块
```

---

## 三、Python SDK 目录结构

```
python/
├── pyproject.toml                     # Python 包配置
├── continuum_sdk/
│   ├── __init__.py                   # SDK入口
│   ├── agent/                       # Agent API
│   │   ├── __init__.py
│   │   ├── runtime.py              # 运行时API
│   │   └── session.py              # 会话API
│   ├── tools/                       # 工具API
│   │   ├── __init__.py
│   │   ├── builtin.py              # 内置工具
│   │   └── custom.py               # 自定义工具
│   ├── memory/                      # 记忆API
│   │   ├── __init__.py
│   │   └── layers.py               # 分层记忆
│   ├── workflow/                     # 工作流API
│   │   ├── __init__.py
│   │   ├── dag.py                  # DAG定义
│   │   └── nodes.py                # 节点类型
│   ├── observability/                # 可观测性API
│   │   ├── __init__.py
│   │   ├── tracing.py
│   │   └── metrics.py
│   └── _internal/                   # 内部模块（PyO3绑定）
│       ├── __init__.py
│       └── bindings.py              # Rust绑定
│
└── tests/
    ├── test_agent.py
    ├── test_tools.py
    ├── test_memory.py
    └── test_workflow.py
```

---

## 四、CLI 产品目录结构

```
cli/
├── Cargo.toml                         # CLI crate 配置
├── src/
│   ├── main.rs                       # 入口点
│   ├── tui/                          # TUI界面
│   │   ├── mod.rs
│   │   ├── app.rs                   # 应用状态
│   │   ├── ui.rs                    # UI渲染
│   │   └── event.rs                 # 事件处理
│   ├── commands/                     # 子命令
│   │   ├── mod.rs
│   │   ├── run.rs                   # continuum run
│   │   ├── config.rs                # continuum config
│   │   ├── session.rs              # continuum session
│   │   └── tools.rs                # continuum tools
│   ├── output/                       # 输出处理
│   │   ├── mod.rs
│   │   ├── streaming.rs            # 流式输出
│   │   └── format.rs               # 格式化
│   └── config/                       # 配置处理
│       ├── mod.rs
│       └── loader.rs
│
└── tests/
    ├── integration_tests.rs
    └── e2e_tests.rs
```

---

## 五、测试目录结构

```
tests/
├── integration/                       # 集成测试
│   ├── test_agent_lifecycle.rs
│   ├── test_session_persistence.rs
│   ├── test_workflow_execution.rs
│   └── test_tool_execution.rs
│
├── e2e/                               # 端到端测试
│   ├── test_cli_basic.py
│   ├── test_cli_session.py
│   ├── test_sdk_integration.py
│   └── fixtures/
│       ├── sample_project/
│       └── test_configs/
│
└── benchmarks/                        # 性能测试
    ├── benchmark_agent_throughput.rs
    ├── benchmark_session_concurrency.rs
    └── results/                      # 基准结果
```

---

## 六、示例目录结构

```
examples/
├── basic/
│   ├── hello_agent.py               # 最简Agent示例
│   ├── session_example.py           # 会话示例
│   └── tool_usage.py                # 工具使用示例
│
├── advanced/
│   ├── multi_agent_workflow.py      # 多Agent工作流
│   ├── custom_memory.py             # 自定义记忆
│   ├── plugin_development.py        # 插件开发
│   └── mcp_integration.py          # MCP集成
│
└── plugins/
    ├── example_plugin/              # 示例插件
    │   ├── Cargo.toml
    │   └── src/
    └── plugin_template/             # 插件模板
```

---

## 七、文档目录结构

```
docs/
├── ARCHITECTURE_V4.md               # 完整架构设计
├── PROJECT_STRUCTURE.md             # 本文档
├── SUPER_PROJECT_VISION.md          # 项目愿景
├── DIFFERENTIATION_STRATEGY.md      # 差异化策略
├── COST_CONTROL_DESIGN.md           # 成本控制设计
│
├── api/                              # API文档
│   ├── rust_api.md                  # Rust API
│   └── python_api.md               # Python API
│
├── guides/                           # 使用指南
│   ├── getting_started.md           # 快速开始
│   ├── cli_usage.md                 # CLI使用
│   ├── sdk_development.md           # SDK开发
│   └── plugin_development.md        # 插件开发
│
└── internals/                        # 内部文档
    ├── security.md                  # 安全设计
    ├── performance.md               # 性能优化
    └── testing.md                   # 测试策略
```

---

## 八、模块依赖关系图

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI (Layer 5)                           │
│                           ↓                                     │
│                 Layer 4 (Integration)                           │
│                           ↓                                     │
│                 Layer 3 (Capabilities)                          │
│                           ↓                                     │
│                    Layer 2 (Core)                                │
│                           ↓                                     │
│                 Layer 1 (Foundation)                             │
│                           ↓                                     │
│                Layer 0 (Security Gateway)                       │
└─────────────────────────────────────────────────────────────────┘

Python SDK 通过 PyO3 绑定调用 Rust sh-core crate
```

---

## 九、下一步

1. 初始化 Rust workspace
2. 创建各 layer crate
3. 迁移 Python 实现到 Rust
4. 实现 CLI 产品
5. 实现 Python SDK
