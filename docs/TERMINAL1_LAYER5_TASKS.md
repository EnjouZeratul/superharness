# Terminal 1 任务清单 - Layer 5 SDK

> 分配时间: 2026-05-11
> 负责产品: Python SDK + CLI 支持
> 角色: SDK 主力开发者 + CLI 集成支持
> 预计时间: 6-8小时

---

## ⚠️ 重要规则

```
1. 只做本文档列出的任务，不做其他终端的任务
2. 遇到需要等待的依赖项，暂停并通知 Terminal 0
3. 完成每个任务后更新本文档状态
4. CLI 集成完成后立即通知 Terminal 2
```

---

## 🚨 执行顺序警告

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚠️  请按顺序执行，通知不能遗漏！                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  任务执行: T1.1 → T1.2 → T1.3 → T1.4 → T1.5 → T1.6            │
│            ✅ 全部可立即开始                                    │
│                                                                 │
│  ──────────────── 关键通知点 ────────────────                   │
│                                                                 │
│  ✉️  T1.1 完成后 → 通知 Terminal 2: "MCP 集成可用"              │
│  ✉️  T1.2 完成后 → 通知 Terminal 2: "审计集成可用"              │
│  ✉️  T1.3-T1.5 完成后 → 通知 Terminal 3: "SDK 基础 API 可用"    │
│                                                                 │
│  ⏸️  T1.7 需等待 Terminal 2 完成 CLI 全部功能                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 任务清单

### ✅ 可立即开始（无依赖）

#### T1.1: CLI MCP 集成指导
- [ ] 创建 `cli/src/integration/mcp.rs`
- [ ] 封装 mcp_bridge 为 CLI 易用接口
- [ ] 提供使用示例给 Terminal 2
- [ ] **完成后立即通知 Terminal 2**: "MCP 集成可用"
- [ ] 预计时间: 1小时

#### T1.2: CLI 审计集成指导
- [ ] 创建 `cli/src/integration/audit.rs`
- [ ] 封装 audit_logger 为 CLI 易用接口
- [ ] 提供使用示例给 Terminal 2
- [ ] **完成后立即通知 Terminal 2**: "审计集成可用"
- [ ] 预计时间: 0.5小时

#### T1.3: Python SDK PyO3 绑定更新
- [ ] 更新 `rust/sh-core/src/lib.rs`
- [ ] 添加 Agent/Session 基础绑定
- [ ] 确保编译通过
- [ ] 预计时间: 2小时

#### T1.4: Python SDK Agent API
- [ ] 创建 `python/superharness_sdk/agent/__init__.py`
- [ ] 创建 `python/superharness_sdk/agent/runtime.py`
- [ ] 实现 Agent 类基础 API
- [ ] 预计时间: 1.5小时

#### T1.5: Python SDK Session API
- [ ] 创建 `python/superharness_sdk/agent/session.py`
- [ ] 实现 Session 类 API
- [ ] 预计时间: 1小时

#### T1.6: Python SDK Config API
- [ ] 创建 `python/superharness_sdk/config/__init__.py`
- [ ] 创建 `python/superharness_sdk/config/loader.py`
- [ ] 实现配置加载 API
- [ ] 预计时间: 1小时

---

### ⏸️ 需要等待

#### T1.7: CLI 最终测试
- [ ] 等待 Terminal 2 完成 CLI 全部功能
- [ ] 进行 CLI 端到端测试
- [ ] **触发条件**: Terminal 2 通知 "CLI 功能完成"
- [ ] 预计时间: 1小时

---

## 依赖关系图

```
Terminal 1 任务:
├── T1.1-T1.6: ✅ 可立即开始（无依赖）
│   │
│   ├── T1.1 完成后 → 通知 Terminal 2 可开始 T2.7
│   ├── T1.2 完成后 → 通知 Terminal 2 可开始 T2.8
│   └── T1.3-T1.5 完成后 → 通知 Terminal 3 可开始 T3.1-T3.3
│
└── T1.7 最终测试: ⏸️ 等待 Terminal 2
    └── Terminal 2 任务: T2.1-T2.8
```

---

## ⚡ 关键通知点

```
完成 T1.1 后立即通知:
┌────────────────────────────────────────────┐
│  📢 通知 Terminal 2:                       │
│  "MCP 集成可用，可以开始 T2.7"               │
└────────────────────────────────────────────┘

完成 T1.2 后立即通知:
┌────────────────────────────────────────────┐
│  📢 通知 Terminal 2:                       │
│  "审计集成可用，可以开始 T2.8"              │
└────────────────────────────────────────────┘

完成 T1.3-T1.5 后立即通知:
┌────────────────────────────────────────────┐
│  📢 通知 Terminal 3:                       │
│  "SDK 基础 API 可用，可以开始 T3.1-T3.3"    │
└────────────────────────────────────────────┘
```

---

## 工作目录

```
CLI 集成支持:
cli/src/integration/
├── mod.rs
├── mcp.rs
└── audit.rs

Python SDK:
python/superharness_sdk/
├── __init__.py
├── agent/
│   ├── __init__.py
│   ├── runtime.py
│   └── session.py
├── config/
│   ├── __init__.py
│   └── loader.py
└── _internal/
    └── bindings.py

Rust 绑定:
rust/sh-core/src/lib.rs
```

---

## 自检清单

```
□ mcp_bridge CLI 封装可用
□ audit_logger CLI 封装可用
□ sh-core 编译通过
□ Python SDK 可导入
□ Agent/Session API 可用
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. CLI 子命令实现 → Terminal 2
2. CLI TUI 界面 → Terminal 2
3. Python SDK Tool API → Terminal 3
4. Python SDK Memory API → Terminal 3
5. 示例代码 → Terminal 3
```

---

## 完成标准

- [ ] MCP/审计 CLI 集成指导完成
- [ ] 已通知 Terminal 2 (T1.1/T1.2)
- [ ] 已通知 Terminal 3 (T1.3-T1.5)
- [ ] Python SDK Agent/Session API 可用
- [ ] 更新本文档状态为完成

---

## 状态更新

完成后通知 Terminal 0：
- "Terminal 1 完成 T1.1-T1.6"
- T1.1/T1.2 已通知 Terminal 2
- T1.3-T1.5 已通知 Terminal 3
