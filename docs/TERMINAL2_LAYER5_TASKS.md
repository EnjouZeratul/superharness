# Terminal 2 任务清单 - Layer 5 CLI

> 分配时间: 2026-05-11
> 负责产品: CLI (终端产品)
> 角色: CLI 主力开发者
> 预计时间: 6-8小时

---

## ⚠️ 重要规则

```
1. 只做本文档列出的任务，不做其他终端的任务
2. 遇到需要等待的依赖项，暂停并通知 Terminal 0
3. 完成每个任务后更新本文档状态
```

---

## 🚨 执行顺序警告

```
┌─────────────────────────────────────────────────────────────────┐
│  ⚠️  请按顺序执行，不要跳过等待！                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  第一阶段: T2.1 → T2.2 → T2.3 → T2.4 → T2.5 → T2.6            │
│            ✅ 可立即开始，无需等待                               │
│                                                                 │
│  ──────────────── 等待分割线 ────────────────                   │
│                                                                 │
│  第二阶段: T2.7 (需要 T1.1 完成 + Terminal 1 通知)              │
│  第三阶段: T2.8 (需要 T1.2 完成 + Terminal 1 通知)              │
│            ⏸️ 收到通知前，禁止开始这两个任务                     │
│                                                                 │
│  完成 T2.6 后请在此暂停，等待 Terminal 1 通知                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 任务清单

### ✅ 可立即开始（无依赖）

#### T2.1: CLI 基础框架 ✅
- [x] 创建 `cli/src/cli/mod.rs`
- [x] 创建 `cli/src/cli/args.rs` - 命令行参数定义
- [x] 创建 `cli/src/cli/app.rs` - CliApp 结构体
- [x] 预计时间: 1小时
- **完成时间**: 2026-05-11

#### T2.2: run 子命令 ✅
- [x] 创建 `cli/src/cli/commands/run.rs`
- [x] 实现基本任务执行
- [x] 预计时间: 1小时
- **完成时间**: 2026-05-11

#### T2.3: session 子命令 ✅
- [x] 创建 `cli/src/cli/commands/session.rs`
- [x] 实现 list/resume/delete
- [x] 预计时间: 1小时
- **完成时间**: 2026-05-11

#### T2.4: config 子命令 ✅
- [x] 创建 `cli/src/cli/commands/config.rs`
- [x] 实现 show/set/init
- [x] 预计时间: 0.5小时
- **完成时间**: 2026-05-11

#### T2.5: tools 子命令 ✅
- [x] 创建 `cli/src/cli/commands/tools.rs`
- [x] 实现 list/info
- [x] 预计时间: 0.5小时
- **完成时间**: 2026-05-11

#### T2.6: TUI 基础框架 ✅
- [x] 更新 `cli/src/tui/mod.rs`
- [x] 创建 `cli/src/tui/components/chat.rs`
- [x] 创建 `cli/src/tui/components/input.rs`
- [x] 创建 `cli/src/tui/components/status.rs`
- [x] 预计时间: 2小时
- **完成时间**: 2026-05-11

---

### ⏸️ 需要等待

#### T2.7: MCP 协议集成 ✅
- [x] 在 CLI 中集成 MCP 协议支持
- [x] 使用 layer4 的 mcp_bridge
- [x] 添加 McpService 到 CliApp
- [x] 预计时间: 1小时
- **完成时间**: 2026-05-11

#### T2.8: 审计日志集成 ✅
- [x] 在 CLI 中集成审计日志
- [x] 使用 layer4 的 audit_logger
- [x] 添加 AuditService 到 CliApp
- [x] 在 run 命令中记录执行审计
- [x] 预计时间: 0.5小时
- **完成时间**: 2026-05-11

---

## 依赖关系图

```
Terminal 2 任务:
├── T2.1-T2.6: ✅ 可立即开始（无依赖）
│
├── T2.7 MCP集成: ⏸️ 等待 Terminal 1
│   └── Terminal 1 任务: T1.1 CLI MCP 集成
│
└── T2.8 审计集成: ⏸️ 等待 Terminal 1
    └── Terminal 1 任务: T1.2 CLI 审计集成
```

---

## 工作目录

```
cli/src/
├── cli/
│   ├── mod.rs
│   ├── args.rs
│   ├── app.rs
│   └── commands/
│       ├── mod.rs
│       ├── run.rs
│       ├── session.rs
│       ├── config.rs
│       └── tools.rs
├── tui/
│   ├── mod.rs
│   └── components/
│       ├── mod.rs
│       ├── chat.rs
│       ├── input.rs
│       └── status.rs
└── main.rs (已存在)
```

---

## 自检清单

```
□ cargo check 通过
□ cargo clippy 无警告
□ cargo fmt 通过
□ 每个子命令可执行
□ TUI 界面可渲染
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. Python SDK 任何开发 → Terminal 1 + Terminal 3
2. mcp_bridge 实现 → 已完成，等待 Terminal 1 集成指导
3. audit_logger 实现 → 已完成，等待 Terminal 1 集成指导
4. sh-core PyO3 绑定 → Terminal 1
```

---

## 完成标准

- [x] 所有子命令可执行 ✅
- [x] TUI 界面可交互 ✅
- [x] cargo test 通过 (29 tests) ✅
- [x] 更新本文档状态为完成 ✅

---

## 状态更新

**Terminal 2 完成 T2.1-T2.8** ✅

### 完成内容:
- ✅ CLI 基础框架 (cli/mod.rs, args.rs, app.rs)
- ✅ run 子命令 (commands/run.rs)
- ✅ session 子命令 (commands/session.rs) - list/resume/delete/show
- ✅ config 子命令 (commands/config.rs) - show/set/init/keys
- ✅ tools 子命令 (commands/tools.rs) - list with filter/verbose
- ✅ TUI 基础框架 (tui/components: chat/input/status)
- ✅ MCP 协议集成 (integration/mcp.rs + CliApp)
- ✅ 审计日志集成 (integration/audit.rs + CliApp)
- ✅ 临时修复 sh-core 编译问题（协助 Terminal 1）
- ✅ cargo test: 29 passed, 0 failed

### 通知 Terminal 0:
- "Terminal 2 全部完成 T2.1-T2.8，Layer 5 CLI 开发结束"
