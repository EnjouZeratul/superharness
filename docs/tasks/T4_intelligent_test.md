# T4 任务：intelligent.py 测试（测试专家）

## 基本信息

| 项目 | 内容 |
|------|------|
| 终端 | T4（测试专家） |
| 模块 | `python/continuum_sdk/agent/intelligent.py` |
| 当前覆盖率 | 28% |
| 目标覆盖率 | 70% |
| 优先级 | P1（最高业务价值） |
| 参考规范 | `docs/TESTING_STANDARDS.md` |

---

## 角色定位

**T4 是测试专家，职责：**

1. ✅ 完成 `intelligent.py` 核心模块测试
2. ✅ Review 所有终端的测试质量
3. ✅ 确保无占位测试、无漏洞
4. ✅ 验证测试覆盖率达标

---

## 任务目标

为智能 Agent 核心模块编写完整测试，这是系统的核心业务逻辑。

---

## 需测试的功能

### 1. 任务规划 (Planner)
- [ ] 任务分解逻辑
- [ ] Step 生成
- [ ] 依赖关系建立
- [ ] 规划结果验证

### 2. 执行引擎
- [ ] `execute_plan()` 主流程
- [ ] Step 执行状态转换
- [ ] 进度跟踪
- [ ] 执行结果收集

### 3. 自校正机制 (SelfCorrection)
- [ ] 错误分类（ImportError, FileNotFoundError, etc）
- [ ] 恢复策略选择
- [ ] 重试逻辑
- [ ] 错误历史记录

### 4. 进度跟踪 (ProgressTracker)
- [ ] 进度计算
- [ ] 事件发布
- [ ] 日志记录
- [ ] 进度条显示

### 5. Agent 模式
- [ ] AUTONOMOUS 模式
- [ ] INTERACTIVE 模式
- [ ] STEP_BY_STEP 模式

### 6. 边界条件
- [ ] 空任务处理
- [ ] 无步骤规划
- [ ] 执行中途失败
- [ ] 并发执行

---

## 测试要求

### 测试专家标准

```python
# ✅ 专家级测试示例
import pytest
from continuum_sdk.agent import IntelligentAgent, AgentMode
from continuum_sdk.agent.planner import Plan, Step, StepType

class TestIntelligentAgentPlanning:
    """测试智能Agent规划功能"""
    
    def test_plan_fix_bug_generates_correct_steps(self):
        """测试bug修复任务生成正确的步骤"""
        agent = IntelligentAgent(mode=AgentMode.AUTONOMOUS)
        
        plan = agent.plan("Fix the null pointer bug in auth.py")
        
        # 验证规划存在
        assert plan is not None
        assert len(plan.steps) > 0
        
        # 验证步骤类型
        step_types = [s.type for s in plan.steps]
        assert StepType.ANALYZE in step_types
        
        # 验证步骤描述不为空
        for step in plan.steps:
            assert step.description, "Step description should not be empty"
            assert step.id, "Step should have an ID"

    def test_plan_add_feature_includes_verify_step(self):
        """测试功能添加任务包含验证步骤"""
        agent = IntelligentAgent(mode=AgentMode.AUTONOMOUS)
        
        plan = agent.plan("Add logging to user service")
        
        # 验证有验证步骤
        verify_steps = [s for s in plan.steps if s.type == StepType.VERIFY]
        assert len(verify_steps) > 0, "Plan should include VERIFY step"
    
    @pytest.mark.asyncio
    async def test_execute_plan_updates_progress(self):
        """测试计划执行更新进度"""
        agent = IntelligentAgent(mode=AgentMode.AUTONOMOUS)
        
        plan = agent.plan("Test task")
        tracker = ProgressTracker()
        
        # Mock 执行
        with patch.object(agent, '_execute_step', new_callable=AsyncMock):
            result = await agent.execute_plan(plan)
        
        assert result.status in ["completed", "failed"]
        assert result.completed_steps >= 0


class TestSelfCorrection:
    """测试自校正机制"""
    
    def test_classify_import_error(self):
        """测试ImportError分类"""
        correction = SelfCorrection()
        
        error = ImportError("No module named 'nonexistent'")
        ctx = correction.analyze_error(error)
        
        assert ctx.error_type == ErrorType.IMPORT
        assert ctx.retry_count == 0
    
    def test_propose_correction_for_connection_error(self):
        """测试连接错误的修正建议"""
        correction = SelfCorrection()
        
        error = ConnectionError("Connection refused")
        ctx = correction.analyze_error(error)
        proposal = correction.propose_correction(ctx)
        
        assert proposal.strategy in [RecoveryStrategy.RETRY, RecoveryStrategy.RETRY_MODIFIED]
    
    def test_max_retries_asks_user(self):
        """测试超过最大重试次数后询问用户"""
        correction = SelfCorrection(max_retries=3)
        
        # 模拟已重试3次
        ctx = ErrorContext(
            error=ConnectionError("Failed"),
            error_type=ErrorType.NETWORK,
            retry_count=3
        )
        proposal = correction.propose_correction(ctx)
        
        assert proposal.strategy == RecoveryStrategy.ASK_USER


class TestProgressTracker:
    """测试进度跟踪"""
    
    def test_progress_calculation(self):
        """测试进度计算"""
        tracker = ProgressTracker()
        tracker.start(total_steps=10)
        
        tracker.update_step_completed(1)
        
        assert tracker.progress == 0.1
        
        tracker.update_step_completed(2)
        
        assert tracker.progress == 0.2
    
    def test_progress_text_display(self):
        """测试进度文本显示"""
        tracker = ProgressTracker()
        tracker.start(total_steps=5)
        
        for i in range(1, 4):
            tracker.update_step_completed(i)
        
        text = tracker.get_progress_text()
        
        assert "3/5" in text or "60%" in text
```

---

## 测试文件位置

```
python/tests/test_intelligent_agent.py  # 已存在，需扩展
python/tests/test_self_correction.py    # 可能需要新建
python/tests/test_progress.py           # 可能需要新建
```

---

## Review 职责

T4 完成后，需 Review 其他终端的测试：

| 终端 | 模块 | Review 内容 |
|------|------|-------------|
| T1 | builtin.py | 工具执行测试是否完整 |
| T2 | dag.py | 算法测试是否覆盖边界 |
| T3 | client.py | Mock 是否正确，无真实调用 |

**Review 检查清单：**
- [ ] 无 `assert True` / `pass` / 无断言
- [ ] 测试名称描述清晰
- [ ] 覆盖成功+失败+边界
- [ ] Mock 使用正确
- [ ] 无测试间依赖

---

## 验收标准

1. `intelligent.py` 覆盖率 ≥ 70%
2. 所有公开方法均有测试
3. 自校正机制完整测试
4. 进度跟踪完整测试
5. Review 其他终端测试通过
6. CI 通过

---

## 提交方式

```bash
git checkout -b test/intelligent-coverage
git add python/tests/test_intelligent*.py python/tests/test_self_correction.py python/tests/test_progress.py
git commit -m "test: add comprehensive tests for intelligent agent core"
git push origin test/intelligent-coverage
# 创建 PR
```
