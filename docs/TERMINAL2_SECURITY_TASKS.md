# Terminal 2 任务清单 - Rust 核心完善与安全审查

> 分配时间: 2026-05-11
> 擅长方向: Rust CLI、底层调试
> 前置条件: Layer 5 全部完成 ✅

---

## 🎯 擅长匹配

```
Terminal 2 擅长: Rust CLI、底层调试 (曾修复 sh-core 编译问题)
本次任务: ✅ 完全匹配
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
│  ✅ 全部可立即开始，可并行                                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  T2.1 OpenAI API 实现 (Rust) ─┐                                 │
│  T2.2 Gemini API 实现 (Rust) ─┼─ 可并行                         │
│  T2.3 安全审查 ───────────────┘                                 │
│                                                                 │
│  T2.4 Rust 测试补充                                              │
│       完成后需通知 Terminal 3 可开始集成测试                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 任务清单

### T2.1: OpenAI API 实现 (Rust) ✅
- [x] 打开 `rust/layer1/src/llm_client.rs`
- [x] 实现 `send_openai()` 方法
- [x] 使用 OpenAI Chat Completions API 格式
- [x] 添加单元测试
- [x] 预计时间: 1.5小时
- **完成时间**: 2026-05-11

### T2.2: Gemini API 实现 (Rust) ✅
- [x] 打开 `rust/layer1/src/llm_client.rs`
- [x] 实现 `send_gemini()` 方法
- [x] 使用 Google Generative AI API 格式
- [x] 添加单元测试
- [x] 预计时间: 1.5小时
- **完成时间**: 2026-05-11

### T2.3: 安全审查 ✅
- [x] **Clippy 全面检查**
  - [x] `cargo clippy --all-targets --all-features`
  - [x] 仅有低优先级警告（未使用导入/字段）
- [x] **Unsafe 代码审查**
  - [x] 无 unsafe 代码块
- [x] 输出报告到 `docs/security/audit-report.md`
- [x] 预计时间: 2小时
- **完成时间**: 2026-05-11
- **注意**: cargo audit 因网络问题无法获取 advisory database

### T2.4: Rust 测试补充 ✅
- [x] 补充 `llm_client` 测试 (新增 8 个测试)
- [x] 运行测试确保通过
- [x] 测试结果: 219 passed (Layer 0-2, 4, CLI)
- [x] 预计时间: 1.5小时
- **完成时间**: 2026-05-11

---

## 工作目录

```
rust/layer1/src/
└── llm_client.rs       ← T2.1, T2.2

rust/                   ← T2.3 安全审查

docs/security/
└── audit-report.md     ← T2.3 输出
```

---

## 自检清单

```
□ OpenAI API 可调用
□ Gemini API 可调用
□ cargo audit 无漏洞
□ cargo clippy 零警告
□ cargo test 全部通过
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. Python SDK 精简 → Terminal 1
2. Python SDK 测试 → Terminal 1
3. 集成测试设计 → Terminal 3
4. E2E 测试场景 → Terminal 3
5. 示例代码验证 → Terminal 3
```

---

## ⚡ 关键通知点

```
完成 T2.4 后立即通知:
┌────────────────────────────────────────────┐
│  📢 通知 Terminal 3:                       │
│  "Rust 核心测试完成，可开始集成测试"         │
└────────────────────────────────────────────┘
```

---

## 完成标准

- [ ] OpenAI API 实现 + 测试通过
- [ ] Gemini API 实现 + 测试通过
- [ ] 安全审查报告生成
- [ ] Rust 测试补充完成
- [ ] 更新本文档状态为完成

---

## 状态更新

**Terminal 2 完成 Rust 核心完善与安全审查** ✅

### 完成内容:
- ✅ OpenAI API 实现 (send_openai)
- ✅ Gemini API 实现 (send_gemini)
- ✅ 安全审查报告 (docs/security/audit-report.md)
- ✅ Rust 测试补充 (219 tests passed)
- ✅ 无 unsafe 代码
- ✅ Clippy 检查通过（仅低优先级警告）

### 测试统计:
| Layer | Tests |
|-------|-------|
| Layer 0 | 51 |
| Layer 1 | 28 |
| Layer 2 | 53 |
| Layer 4 | 58 |
| CLI | 29 |
| **Total** | **219** |

### 通知 Terminal 3:
- "Rust 核心测试完成，可开始集成测试"
