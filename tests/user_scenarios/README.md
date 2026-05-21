# 用户测试场景设计

> 版本: v1.0.0
> 更新时间: 2026-05-21
> 状态: 待 T1/T2 完成后执行

---

## 概述

本目录包含 Continuum 用户测试场景设计，用于验证 Agent 的自主任务完成能力、Git 工作流、MCP 工具调用、会话恢复等核心功能。

---

## 场景列表

| 场景 | 文件 | 依赖 | 描述 |
|------|------|------|------|
| 场景1 | `scenario_1_fizzbuzz_fix.py` | T1 P1.2 | Agent 自主修复 FizzBuzz bug |
| 场景2 | `scenario_2_git_workflow.py` | T2 P1.3 | Git 工作流完整验证 |
| 场景3 | `scenario_3_mcp_tools.py` | T2 P1.4 | MCP 工具调用验证 |
| 场景4 | `scenario_4_session_recovery.py` | 无 | 会话中断恢复验证 |
| 场景5 | `scenario_5_multi_turn_context.py` | T1 P1.2 | 多轮对话上下文保持 |

---

## 测试执行流程

### 1. 环境准备

```bash
# 安装依赖
pip install continuum[dev]

# 配置 API Key
export ANTHROPIC_API_KEY=your-key

# 运行场景测试
python tests/user_scenarios/run_all_scenarios.py
```

### 2. 单场景执行

```bash
# 执行单个场景
python tests/user_scenarios/scenario_1_fizzbuzz_fix.py

# 带详细输出
python tests/user_scenarios/scenario_1_fizzbuzz_fix.py --verbose
```

### 3. 结果收集

测试结果保存在 `tests/user_scenarios/results/` 目录：
- `scenario_1_result.json`
- `scenario_2_result.json`
- ...

---

## 验收标准

每个场景需满足以下标准：

### 基本标准

1. **成功率**: 核心路径成功率 ≥ 95%
2. **时间限制**: 单场景执行时间 ≤ 5 分钟
3. **无崩溃**: 不应出现程序崩溃或异常退出
4. **日志完整**: 执行过程日志记录完整

### 进阶标准

1. **自主性**: Agent 能自主完成任务，无需用户干预
2. **准确性**: 任务结果准确无误
3. **恢复性**: 遇到错误能自动恢复或明确提示用户
4. **效率**: Token 使用合理，成本可控

---

## 边界条件覆盖

每个场景包含以下边界测试：

| 类型 | 描述 |
|------|------|
| 空输入 | 空字符串、空文件、空目录 |
| 超长输入 | 超长文本、大量文件、复杂任务 |
| 特殊字符 | Unicode、控制字符、路径注入 |
| 并发操作 | 多文件同时操作、多任务并发 |
| 网络异常 | API 错误、超时、断连 |
| 权限问题 | 无权限文件、锁定文件、只读目录 |

---

## 错误恢复验证

每个场景包含以下错误恢复测试：

| 错误类型 | 预期行为 |
|----------|----------|
| 工具执行失败 | 自动重试或降级处理 |
| API 调用失败 | 三层恢复机制触发 |
| 会话中断 | 自动保存检查点 |
| 用户中断 | 优雅退出，状态保存 |

---

## 目录结构

```
tests/user_scenarios/
├── README.md                    # 本文档
├── run_all_scenarios.py         # 执行所有场景
├── scenario_1_fizzbuzz_fix.py   # 场景1: FizzBuzz bug 修复
├── scenario_2_git_workflow.py   # 场景2: Git 工作流
├── scenario_3_mcp_tools.py      # 场景3: MCP 工具调用
├── scenario_4_session_recovery.py # 场景4: 会话恢复
├── scenario_5_multi_turn_context.py # 场景5: 多轮对话
├── fixtures/                    # 测试数据
│   ├── fizzbuzz_buggy.py        # 有 bug 的 FizzBuzz
│   ├── sample_project/          # 示例项目
│   └── mcp_test_server.py       # MCP 测试服务器
└── results/                     # 测试结果
    ├── scenario_1_result.json
    ├── scenario_2_result.json
    └── ...
```

---

## 待办事项

- [ ] T1 完成 Agent 智能增强后执行场景1、5
- [ ] T2 完成 Git 集成后执行场景2
- [ ] T2 完成 MCP 支持后执行场景3
- [ ] 场景4 可立即执行（无依赖）

---

*Continuum User Scenarios - Ensuring Quality Through Real Testing*