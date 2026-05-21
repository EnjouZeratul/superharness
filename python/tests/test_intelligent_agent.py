"""
Intelligent Agent Tests

Tests for task planning, self-correction, and progress tracking.
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch

from continuum_sdk.agent import (
    Planner,
    Plan,
    Step,
    StepType,
    StepStatus,
    SelfCorrection,
    ErrorContext,
    ErrorType,
    Correction,
    RecoveryStrategy,
    ProgressTracker,
    ProgressEvent,
    ProgressState,
    StepLogger,
    IntelligentAgent,
    AgentMode,
    ExecutionResult,
)


# ==================== Planner Tests ====================

class TestPlanner:
    """Planner tests"""

    def test_plan_fix_bug(self):
        """Test planning for bug fix task"""
        planner = Planner()
        plan = asyncio.run(planner.plan("fix bug in auth.py"))
        assert plan is not None
        assert len(plan.steps) > 0
        assert any(s.type == StepType.SEARCH for s in plan.steps)
        assert any(s.type == StepType.EDIT for s in plan.steps)
        assert any(s.type == StepType.TEST for s in plan.steps)

    def test_plan_add_feature(self):
        """Test planning for feature addition"""
        planner = Planner()
        plan = asyncio.run(planner.plan("add feature to login"))
        assert plan is not None
        assert len(plan.steps) > 0

    def test_plan_has_dependencies(self):
        """Test that steps have proper dependencies"""
        planner = Planner()
        plan = asyncio.run(planner.plan("fix bug in parser"))
        # Steps should be sequential (each depends on previous)
        for i in range(1, len(plan.steps)):
            assert plan.steps[i].dependencies  # Has at least one dependency

    def test_plan_progress(self):
        """Test plan progress calculation"""
        planner = Planner()
        plan = asyncio.run(planner.plan("fix bug"))
        progress = plan.get_progress()
        assert progress["total"] > 0
        assert progress["completed"] == 0
        assert progress["progress_percent"] == 0

    def test_plan_step_completion(self):
        """Test step completion updates progress"""
        planner = Planner()
        plan = asyncio.run(planner.plan("fix bug"))
        # Complete first step
        plan.steps[0].status = StepStatus.COMPLETED
        progress = plan.get_progress()
        assert progress["completed"] == 1

    def test_plan_get_pending_steps(self):
        """Test getting pending steps"""
        planner = Planner()
        plan = asyncio.run(planner.plan("fix bug"))
        # First step should be pending (no dependencies)
        pending = plan.get_pending_steps()
        assert len(pending) > 0
        # First step has no dependencies, should be available
        assert pending[0].id == "s1"

    def test_plan_to_dict(self):
        """Test plan serialization"""
        planner = Planner()
        plan = asyncio.run(planner.plan("fix bug"))
        data = plan.to_dict()
        assert "id" in data
        assert "task" in data
        assert "steps" in data
        assert "progress" in data


class TestStep:
    """Step tests"""

    def test_step_creation(self):
        """Test step creation"""
        step = Step(
            id="s1",
            type=StepType.SEARCH,
            description="Search for bug",
            action="search codebase",
        )
        assert step.id == "s1"
        assert step.type == StepType.SEARCH
        assert step.status == StepStatus.PENDING

    def test_step_to_dict(self):
        """Test step serialization"""
        step = Step(
            id="s1",
            type=StepType.EDIT,
            description="Edit file",
            action="edit auth.py",
            target="auth.py",
        )
        data = step.to_dict()
        assert data["id"] == "s1"
        assert data["type"] == "edit"
        assert data["target"] == "auth.py"


# ==================== Self-Correction Tests ====================

class TestSelfCorrection:
    """SelfCorrection tests"""

    def test_classify_import_error(self):
        """Test classifying import error"""
        correction = SelfCorrection()
        error = ImportError("No module named 'foo'")
        error_ctx = correction.analyze_error(error)
        assert error_ctx.error_type == ErrorType.IMPORT

    def test_classify_file_not_found(self):
        """Test classifying file not found error"""
        correction = SelfCorrection()
        error = FileNotFoundError("File not found: config.py")
        error_ctx = correction.analyze_error(error)
        assert error_ctx.error_type == ErrorType.NOT_FOUND

    def test_classify_type_error(self):
        """Test classifying type error"""
        correction = SelfCorrection()
        error = TypeError("'NoneType' object has no attribute 'x'")
        error_ctx = correction.analyze_error(error)
        assert error_ctx.error_type == ErrorType.TYPE

    def test_classify_syntax_error(self):
        """Test classifying syntax error"""
        correction = SelfCorrection()
        error = SyntaxError("invalid syntax")
        error_ctx = correction.analyze_error(error)
        assert error_ctx.error_type == ErrorType.SYNTAX

    def test_propose_correction_retry(self):
        """Test proposing retry for transient error"""
        correction = SelfCorrection()
        error = ConnectionError("Connection refused")
        error_ctx = correction.analyze_error(error)
        proposal = correction.propose_correction(error_ctx)
        assert proposal.strategy in (RecoveryStrategy.RETRY, RecoveryStrategy.RETRY_MODIFIED)

    def test_propose_correction_ask_user_after_retries(self):
        """Test asking user after too many retries"""
        correction = SelfCorrection()
        error = ValueError("Invalid value")
        error_ctx = correction.analyze_error(error, step_id="s1")
        error_ctx.attempt = 5  # Too many attempts
        proposal = correction.propose_correction(error_ctx)
        assert proposal.strategy == RecoveryStrategy.ASK_USER

    def test_pattern_based_correction(self):
        """Test pattern-based correction for import error"""
        correction = SelfCorrection()
        error = ImportError("No module named 'requests'")
        error_ctx = correction.analyze_error(error)
        proposal = correction.propose_correction(error_ctx)
        assert proposal.strategy == RecoveryStrategy.RETRY_MODIFIED
        assert "requests" in proposal.modified_action or "install" in proposal.description.lower()

    def test_error_history(self):
        """Test error history tracking"""
        correction = SelfCorrection()
        error1 = ImportError("No module 'foo'")
        error2 = TypeError("Wrong type")
        correction.analyze_error(error1)
        correction.analyze_error(error2)
        assert len(correction.get_error_history()) == 2

    def test_mark_successful(self):
        """Test marking correction as successful"""
        correction = SelfCorrection()
        error = ImportError("No module 'bar'")
        error_ctx = correction.analyze_error(error)
        proposal = correction.propose_correction(error_ctx)
        correction.mark_successful(error_ctx, proposal)
        # Same error should return saved correction
        error_ctx2 = correction.analyze_error(ImportError("No module 'bar'"))
        proposal2 = correction.propose_correction(error_ctx2)
        assert proposal2.strategy == proposal.strategy


class TestErrorContext:
    """ErrorContext tests"""

    def test_error_context_creation(self):
        """Test error context creation"""
        ctx = ErrorContext(
            error_type=ErrorType.IMPORT,
            message="No module named 'foo'",
            step_id="s1",
        )
        assert ctx.error_type == ErrorType.IMPORT
        assert ctx.step_id == "s1"

    def test_error_context_to_dict(self):
        """Test error context serialization"""
        ctx = ErrorContext(
            error_type=ErrorType.RUNTIME,
            message="Runtime error",
        )
        data = ctx.to_dict()
        assert data["error_type"] == "runtime"
        assert data["message"] == "Runtime error"


# ==================== Progress Tracker Tests ====================

class TestProgressTracker:
    """ProgressTracker tests"""

    def test_start_tracking(self):
        """Test starting tracking"""
        tracker = ProgressTracker()
        tracker.start(5)
        assert tracker.state == ProgressState.RUNNING
        assert tracker.total_steps == 5

    def test_update_step_completed(self):
        """Test step completion"""
        tracker = ProgressTracker()
        tracker.start(3)
        tracker.update_step("s1", "completed", "Step 1")
        assert tracker.completed_steps == 1

    def test_progress_calculation(self):
        """Test progress percentage"""
        tracker = ProgressTracker()
        tracker.start(4)
        tracker.update_step("s1", "completed", "Step 1")
        tracker.update_step("s2", "completed", "Step 2")
        progress = tracker.get_progress()
        assert progress["percent"] == 50.0

    def test_progress_text(self):
        """Test progress text format"""
        tracker = ProgressTracker()
        tracker.start(5)
        tracker.update_step("s1", "completed", "Step 1")
        text = tracker.get_progress_text()
        assert "1/5" in text
        assert "20%" in text

    def test_progress_bar(self):
        """Test progress bar"""
        tracker = ProgressTracker()
        tracker.start(10)
        tracker.update_step("s1", "completed")
        bar = tracker.get_status_bar()
        assert "10%" in bar

    def test_progress_callback(self):
        """Test progress callback"""
        events = []
        tracker = ProgressTracker()
        tracker.on_progress(lambda e: events.append(e))
        tracker.start(2)
        tracker.update_step("s1", "completed", "Step 1")
        assert len(events) == 2  # start + step update

    def test_estimate_remaining(self):
        """Test remaining time estimation"""
        tracker = ProgressTracker()
        tracker.start(4)
        tracker.update_step("s1", "completed")
        # Can estimate after first completion
        remaining = tracker.estimate_remaining()
        assert remaining is not None

    def test_auto_complete(self):
        """Test auto completion when all steps done"""
        tracker = ProgressTracker()
        tracker.start(2)
        tracker.update_step("s1", "completed")
        tracker.update_step("s2", "completed")
        assert tracker.state == ProgressState.COMPLETED


class TestStepLogger:
    """StepLogger tests"""

    def test_log_step(self):
        """Test logging step"""
        logger = StepLogger()
        logger.log("s1", "started", "Starting step 1")
        logger.log("s1", "completed", "Done")
        assert len(logger.logs) == 2

    def test_get_step_logs(self):
        """Test getting logs for specific step"""
        logger = StepLogger()
        logger.log("s1", "started", "Step 1")
        logger.log("s2", "started", "Step 2")
        logger.log("s1", "completed", "Done")
        s1_logs = logger.get_step_logs("s1")
        assert len(s1_logs) == 2

    def test_get_recent_logs(self):
        """Test getting recent logs"""
        logger = StepLogger()
        for i in range(20):
            logger.log(f"s{i}", "started", f"Step {i}")
        recent = logger.get_recent_logs(5)
        assert len(recent) == 5


# ==================== Intelligent Agent Tests ====================

class TestIntelligentAgent:
    """IntelligentAgent tests"""

    def test_agent_creation(self):
        """Test agent creation"""
        agent = IntelligentAgent(api_key="test-key")
        assert agent.mode == AgentMode.INTERACTIVE
        assert agent.planner is not None
        assert agent.correction is not None
        assert agent.tracker is not None

    def test_agent_with_mode(self):
        """Test agent with different mode"""
        agent = IntelligentAgent(api_key="test-key", mode=AgentMode.AUTONOMOUS)
        assert agent.mode == AgentMode.AUTONOMOUS

    def test_plan_without_llm(self):
        """Test planning without LLM (pattern-based)"""
        agent = IntelligentAgent(api_key="test-key")
        plan = asyncio.run(agent.plan("fix bug in auth.py"))
        assert plan is not None
        assert len(plan.steps) > 0

    def test_get_plan_summary(self):
        """Test plan summary display"""
        agent = IntelligentAgent(api_key="test-key")
        asyncio.run(agent.plan("fix bug"))
        summary = agent.get_plan_summary()
        assert summary is not None
        assert "fix bug" in summary
        assert "s1" in summary

    def test_execution_result(self):
        """Test execution result creation"""
        result = ExecutionResult(
            plan_id="p1",
            task="fix bug",
            status="completed",
            completed_steps=5,
            total_steps=5,
            duration_seconds=10.5,
        )
        assert result.status == "completed"
        data = result.to_dict()
        assert data["plan_id"] == "p1"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])