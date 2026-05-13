# Terminal 1 任务清单 - Python SDK 完善

> 分配时间: 2026-05-11
> 擅长方向: Python SDK、API 设计
> 前置条件: Layer 5 基础开发已完成

---

## 🎯 擅长匹配

```
Terminal 1 擅长: Python SDK、集成工作
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
│  ✅ 全部可立即开始，无依赖                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  T1.1 SDK Quick Start 精简 (优先级最高)                         │
│       目标: 9步 → 3步                                           │
│                                                                 │
│  T1.2 SDK API 文档完善                                          │
│       目标: 5分钟内跑通第一个例子                                │
│                                                                 │
│  T1.3 Python SDK 测试                                           │
│       为 Python 代码补充测试                                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 任务清单

### T1.1: SDK Quick Start 精简 ⭐ 最高优先级
- [ ] 审查当前 `examples/basic/` 示例
- [ ] 参考目标: SmolAgents (2步启动)
- [ ] 重新设计 `superharness_sdk/__init__.py` 导出
- [ ] 创建一键启动示例 `hello_world.py`
- [ ] 目标达成:
  ```python
  # 目标: 3步以内
  from superharness import Agent
  agent = Agent()
  agent.run("hello")
  ```
- [ ] 预计时间: 2小时

### T1.2: SDK API 文档完善
- [ ] 为所有公开 API 添加完整类型提示
- [ ] 添加 docstring + 使用示例
- [ ] 更新 `examples/README.md`
- [ ] 确保 `help(Agent)` 显示有用信息
- [ ] 预计时间: 1.5小时

### T1.3: Python SDK 测试
- [ ] 创建 `python/tests/` 目录
- [ ] 编写 SDK 单元测试
  - [ ] `test_agent.py`
  - [ ] `test_session.py`
  - [ ] `test_tools.py`
  - [ ] `test_memory.py`
- [ ] 运行 `pytest` 确保通过
- [ ] 预计时间: 2小时

---

## 工作目录

```
python/superharness_sdk/
├── __init__.py          ← T1.1 导出优化
├── agent.py             ← T1.2 文档
├── session.py           ← T1.2 文档
├── tools/               ← T1.2 文档
├── memory/              ← T1.2 文档
└── workflow/            ← T1.2 文档

python/tests/
├── test_agent.py        ← T1.3
├── test_session.py      ← T1.3
├── test_tools.py        ← T1.3
└── test_memory.py       ← T1.3

examples/basic/
├── hello_world.py       ← T1.1 一键示例
└── README.md            ← T1.2
```

---

## 自检清单

```
□ Quick Start ≤ 3 步
□ pip install 后可直接运行
□ 所有公开 API 有类型提示
□ 所有公开 API 有 docstring
□ pytest 全部通过
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. Rust LLM client 实现 → Terminal 2
2. 安全审查 → Terminal 2
3. Rust 测试 → Terminal 2
4. 集成测试设计 → Terminal 3
5. E2E 测试场景 → Terminal 3
```

---

## 完成标准

- [ ] Quick Start 精简完成
- [ ] API 文档完善
- [ ] Python 测试通过
- [ ] 更新本文档状态为完成

---

## 状态更新

完成后通知 Terminal 0：
- "Terminal 1 完成 Python SDK 完善"
