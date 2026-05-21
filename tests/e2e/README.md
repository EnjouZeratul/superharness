# E2E 测试场景

本目录包含 Continuum 的端到端测试场景。

## 场景概述

| 场景 | 文件 | 描述 |
|------|------|------|
| 简单问答 | `scenario_qa.py` | 基本问答流程，无工具 |
| 多轮对话 | `scenario_conversation.py` | 上下文保持测试 |
| 工具调用 | `scenario_toolcalling.py` | Agent 工具使用 |
| 会话恢复 | `scenario_session_recovery.py` | 检查点和回滚 |

## 运行方式

```bash
# 运行所有 E2E 测试
pytest tests/e2e/ -v

# 运行特定场景
pytest tests/e2e/scenarios/scenario_qa.py -v

# 运行带标记的测试
pytest -m e2e tests/e2e/
```

## 场景设计原则

1. **真实性**: 场景模拟真实用户行为
2. **完整性**: 从开始到结束的完整流程
3. **可验证**: 有明确的预期结果
4. **独立性**: 场景之间不依赖

## 添加新场景

1. 创建 `scenario_xxx.py`
2. 定义 `ScenarioXXX` 类
3. 实现 `run()` 和 `validate()`
4. 创建对应的 `TestScenarioXXX` 测试类
5. 添加 `pytestmark = pytest.mark.e2e`

## 预期结果验证

每个场景应验证:
- 响应非空
- 响应相关
- 工具正确调用
- 时间约束满足
- 错误正确处理