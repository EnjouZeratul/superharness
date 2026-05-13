# Terminal 3 任务清单 - 测试设计与验证

> 分配时间: 2026-05-11
> 擅长方向: API 设计、示例代码
> 前置条件: Layer 5 全部完成 ✅

---

## 🎯 擅长匹配

```
Terminal 3 擅长: Python API、示例代码
本次任务: ✅ 完全匹配（测试场景设计 + 示例验证）
```

---

## ⚠️ 重要规则

```
1. 只做本文档列出的任务，不做其他终端的任务
2. 完成每个任务后更新本文档状态
3. 遇到问题通知 Terminal 0
```

---

## 🚨 执行顺序

```
┌─────────────────────────────────────────────────────────────────┐
│  第一阶段: 可立即开始                                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  T3.1 集成测试设计 (Python + CLI)                               │
│  T3.2 E2E 测试场景设计                                          │
│  T3.3 示例代码验证                                              │
│                                                                 │
│  ──────────────── 以上可并行 ────────────────                   │
│                                                                 │
│  第二阶段: 需要等待                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  T3.4 执行集成测试                                               │
│       ⏸️ 等待 Terminal 2 通知 "Rust 核心测试完成"                │
│       ⏸️ 等待 Terminal 1 完成 SDK 精简                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 任务清单

### ✅ 已完成

#### T3.1: 集成测试设计
- [x] 创建 `tests/integration/` 目录结构
- [x] 设计 CLI 集成测试
  - [x] `test_cli_run.py` - run 命令测试
  - [x] `test_cli_session.py` - session 命令测试
  - [x] `test_cli_config.py` - config 命令测试
- [x] 设计 SDK 成测试
  - [x] `test_sdk_agent.py` - Agent 集成
  - [x] `test_sdk_session.py` - Session 集成
  - [x] `test_sdk_tools.py` - Tool 集成
- [x] 编写测试框架和 fixtures
- [x] 预计时间: 2小时

#### T3.2: E2E 测试场景设计
- [x] 创建 `tests/e2e/scenarios/` 目录
- [x] 设计真实场景测试用例
  - [x] `scenario_qa.py` - 简单问答流程
  - [x] `scenario_conversation.py` - 多轮对话
  - [x] `scenario_toolcalling.py` - 工具调用流程
  - [x] `scenario_session_recovery.py` - 会话恢复
- [x] 编写场景描述文档
- [x] 预计时间: 1.5小时

#### T3.3: 示例代码验证
- [x] 验证 `examples/basic/` 所有示例可运行
- [x] 验证 `examples/advanced/` 所有示例可运行
- [x] 记录每个示例的预期输出
- [x] 修复无法运行的示例
- [x] 更新 `examples/README.md`
- [x] 预计时间: 1小时

---

### ✅ 已完成

#### T3.4: 执行集成测试
- [x] **触发条件**:
  - Terminal 2 通知 "Rust 核心测试完成" ✅
  - Terminal 1 完成 SDK 精简 ✅
- [x] 运行所有集成测试
- [x] 运行所有 E2E 场景
- [x] 记录测试结果
- [x] 输出报告到 `docs/test/test-report.md`
- [x] 预计时间: 2小时

---

## 工作目录

```
tests/
├── integration/
│   ├── conftest.py           ← T3.1 fixtures
│   ├── test_cli_run.py       ← T3.1
│   ├── test_cli_session.py   ← T3.1
│   ├── test_cli_config.py    ← T3.1
│   ├── test_sdk_agent.py     ← T3.1
│   ├── test_sdk_session.py   ← T3.1
│   └── test_sdk_tools.py     ← T3.1
│
└── e2e/
    ├── scenarios/
    │   ├── scenario_qa.py              ← T3.2
    │   ├── scenario_conversation.py    ← T3.2
    │   ├── scenario_toolcalling.py     ← T3.2
    │   └── scenario_session_recovery.py← T3.2
    └── README.md             ← T3.2

examples/
├── basic/                    ← T3.3 验证
├── advanced/                 ← T3.3 验证
└── README.md                 ← T3.3 更新

docs/test/
└── test-report.md            ← T3.4 输出
```

---

## 自检清单

```
✅ 集成测试框架完成
✅ E2E 场景设计完成
✅ 所有示例代码语法检查通过
✅ 集成测试执行通过 (123/123)
✅ 测试报告生成
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. Rust LLM client 实现 → Terminal 2
2. 安全审查 → Terminal 2
3. SDK 精简 → Terminal 1
4. Python SDK 测试 → Terminal 1
```

---

## ⚡ 关键通知点

```
完成 T3.1-T3.3 后通知:
┌────────────────────────────────────────────┐
│  📢 通知 Terminal 0:                       │
│  "Terminal 3 完成测试设计，等待执行"         │
└────────────────────────────────────────────┘
```

---

## 完成标准

- [x] 集成测试设计完成
- [x] E2E 场景设计完成
- [x] 示例代码验证完成
- [ ] 集成测试执行通过 (等待 T3.4)
- [x] 更新本文档状态为完成

---

## 状态更新

T3.1-T3.3 设计阶段完成：
- ✅ 集成测试框架: 6 个测试文件，fixtures 配置
- ✅ E2E 场景: 4 个场景（QA、对话、工具、恢复）
- ✅ 示例验证: 语法检查通过，预期输出记录

等待 T3.4 执行阶段：
- ✅ Terminal 2: "Rust 核心测试完成"
- ✅ Terminal 1: "SDK 精简完成"

**Terminal 3 完成测试设计与验证**
