# Terminal 3 任务清单 - Layer 5 SDK 支持

> 分配时间: 2026-05-11
> 负责产品: Python SDK 支持
> 角色: SDK API 开发者
> 预计时间: 3-4小时

---

## ⚠️ 重要规则

```
1. 只做本文档列出的任务，不做其他终端的任务
2. 遇到需要等待的依赖项，暂停并通知 Terminal 0
3. 完成每个任务后更新本文档状态
4. 等待 Terminal 1 完成 SDK 基础后再开发 API
```

---

## 任务清单

### ✅ 已完成

#### T3.1: Python SDK Tool API
- [x] 等待 Terminal 1 完成 T1.3-T1.5
- [x] 创建 `python/superharness_sdk/tools/__init__.py`
- [x] 创建 `python/superharness_sdk/tools/builtin.py`
- [x] 创建 `python/superharness_sdk/tools/custom.py`
- [x] 实现 Tool 注册和执行 API
- [x] **触发条件**: Terminal 1 通知 "SDK 基础 API 可用"
- [x] 预计时间: 1.5小时

#### T3.2: Python SDK Memory API
- [x] 等待 Terminal 1 完成 T1.3-T1.5
- [x] 创建 `python/superharness_sdk/memory/__init__.py`
- [x] 创建 `python/superharness_sdk/memory/layers.py`
- [x] 实现分层记忆 API
- [x] **触发条件**: Terminal 1 通知 "SDK 基础 API 可用"
- [x] 预计时间: 1小时

#### T3.3: Python SDK Workflow API
- [x] 等待 Terminal 1 完成 T1.3-T1.5
- [x] 创建 `python/superharness_sdk/workflow/__init__.py`
- [x] 创建 `python/superharness_sdk/workflow/dag.py`
- [x] 实现 Workflow API
- [x] **触发条件**: Terminal 1 通知 "SDK 基础 API 可用"
- [x] 预计时间: 1小时

#### T3.4: 示例代码
- [x] 等待 T3.1-T3.3 完成
- [x] 创建 `examples/basic/hello_agent.py`
- [x] 创建 `examples/basic/session_example.py`
- [x] 创建 `examples/advanced/custom_tools.py`
- [x] 创建 `examples/advanced/workflow.py`
- [x] **触发条件**: T3.1-T3.3 全部完成
- [x] 预计时间: 1小时

---

## 依赖关系图

```
Terminal 3 任务:
│
├── T3.1 Tool API: ✅ 完成
│   └── Terminal 1 任务: T1.3-T1.5 (SDK 基础) ✅
│
├── T3.2 Memory API: ✅ 完成
│   └── Terminal 1 任务: T1.3-T1.5 (SDK 基础) ✅
│
├── T3.3 Workflow API: ✅ 完成
│   └── Terminal 1 任务: T1.3-T1.5 (SDK 基础) ✅
│
└── T3.4 示例代码: ✅ 完成
    └── 本终端任务: T3.1-T3.3 ✅
```

---

## 触发条件

```
开始工作的信号:

Terminal 1 通知:
┌────────────────────────────────────────────┐
│  📢 Terminal 1 通知:                       │
│  "SDK 基础 API 可用，可以开始 Tool/Memory"  │
│                                            │
│  → Terminal 3 开始 T3.1, T3.2, T3.3       │
└────────────────────────────────────────────┘
```

---

## 工作目录

```
python/superharness_sdk/
├── tools/
│   ├── __init__.py
│   ├── builtin.py
│   └── custom.py
├── memory/
│   ├── __init__.py
│   └── layers.py
└── workflow/
    ├── __init__.py
    └── dag.py

examples/
├── basic/
│   ├── hello_agent.py
│   └── session_example.py
└── advanced/
    ├── custom_tools.py
    └── workflow.py
```

---

## 自检清单

```
□ Tool API 可导入使用
□ Memory API 可导入使用
□ Workflow API 可导入使用
□ 所有示例代码可运行
□ 类型提示完整
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. CLI 任何开发 → Terminal 2
2. MCP/审计集成 → Terminal 1
3. SDK Agent/Session API → Terminal 1
4. SDK Config API → Terminal 1
5. sh-core PyO3 绑定 → Terminal 1
```

---

## 完成标准

- [x] Tool API 可用
- [x] Memory API 可用
- [x] Workflow API 可用
- [x] 所有示例代码可运行
- [x] 更新本文档状态为完成

---

## 状态更新

等待 Terminal 1 通知期间：
- ✅ 已完成 API 设计草稿 (`docs/API_DESIGN_DRAFT.md`)
- ✅ 包含 Tool API、Memory API、Workflow API 设计
- ✅ 包含 4 个示例代码草稿

Terminal 1 完成后：
- ✅ T3.1 Tool API 完成 (tools/__init__.py, builtin.py, custom.py)
- ✅ T3.2 Memory API 完成 (memory/__init__.py, layers.py)
- ✅ T3.3 Workflow API 完成 (workflow/__init__.py, dag.py)
- ✅ T3.4 示例代码完成 (basic/, advanced/)
- ✅ SDK __init__.py 已更新导出
- ✅ examples/README.md 已创建

**Terminal 3 完成 T3.1-T3.4**
