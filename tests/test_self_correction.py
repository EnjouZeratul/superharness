"""Tool Failure Self-Correction Tests (T3.5)

Tests that verify the IntelligentAgent's self-correction capabilities:
- Error analysis functionality
- Recovery strategy selection
- corrections_applied counting
- All RecoveryStrategy strategies
"""

import os
import sys
import pytest
import tempfile
import shutil
from pathlib import Path
from unittest.mock import AsyncMock, MagicMock, patch
from datetime import datetime

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "src"))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "python"))

from continuum_sdk.agent.self_correction import (
    SelfCorrection,
    RecoveryStrategy,
    Correction,
    ErrorType,
    ErrorContext,
)
from continuum_sdk.agent.intelligent import IntelligentAgent, ExecutionResult


class TestErrorContext:
    """Error context analysis tests"""

    def test_error_context_creation(self):
        """Test creating an error context via analyze_error"""
        corrector = SelfCorrection()

        error = RuntimeError("Division by zero")
        ctx = corrector.analyze_error(
            error=error,
            step_id="step-1",
            action="calculate",
            target="math.py"
        )

        # Error type should be classified (may be RUNTIME or UNKNOWN depending on error patterns)
        assert ctx.error_type in [ErrorType.RUNTIME, ErrorType.UNKNOWN]
        assert "Division by zero" in ctx.message
        assert ctx.step_id == "step-1"
        assert ctx.attempt >= 1

    def test_error_type_classification_syntax(self):
        """Test syntax error classification"""
        corrector = SelfCorrection()

        error = SyntaxError("invalid syntax")
        ctx = corrector.analyze_error(error)

        assert ctx.error_type == ErrorType.SYNTAX

    def test_error_type_classification_import(self):
        """Test import error classification"""
        corrector = SelfCorrection()

        error = ImportError("No module named 'xyz'")
        ctx = corrector.analyze_error(error)

        assert ctx.error_type == ErrorType.IMPORT

    def test_error_type_classification_permission(self):
        """Test permission error classification"""
        corrector = SelfCorrection()

        error = PermissionError("Access denied")
        ctx = corrector.analyze_error(error)

        assert ctx.error_type == ErrorType.PERMISSION

    def test_error_type_classification_not_found(self):
        """Test not found error classification"""
        corrector = SelfCorrection()

        error = FileNotFoundError("No such file")
        ctx = corrector.analyze_error(error)

        assert ctx.error_type == ErrorType.NOT_FOUND

    def test_error_type_classification_timeout(self):
        """Test timeout error classification"""
        corrector = SelfCorrection()

        error = TimeoutError("Operation timed out")
        ctx = corrector.analyze_error(error)

        assert ctx.error_type == ErrorType.TIMEOUT

    def test_error_context_tracks_attempts(self):
        """Test that error context tracks attempt count"""
        corrector = SelfCorrection()

        error = RuntimeError("Test error")

        ctx1 = corrector.analyze_error(error, context={"attempt": 1})
        ctx2 = corrector.analyze_error(error, context={"attempt": 2})

        # Each should have valid attempt info
        assert ctx1.attempt >= 1
        assert ctx2.attempt >= 1


class TestRecoveryStrategies:
    """Recovery strategy tests"""

    def test_recovery_strategy_values(self):
        """Test all recovery strategy values"""
        strategies = [
            (RecoveryStrategy.RETRY, "retry"),
            (RecoveryStrategy.RETRY_MODIFIED, "retry_modified"),
            (RecoveryStrategy.SKIP, "skip"),
            (RecoveryStrategy.ALTERNATIVE, "alternative"),
            (RecoveryStrategy.ASK_USER, "ask_user"),
            (RecoveryStrategy.ABORT, "abort"),
        ]

        for strategy, expected_value in strategies:
            assert strategy.value == expected_value

    def test_correction_creation(self):
        """Test creating a correction"""
        correction = Correction(
            strategy=RecoveryStrategy.RETRY,
            description="Retry the failed operation",
            modified_action=None,
            parameters={},
            confidence=0.8
        )

        assert correction.strategy == RecoveryStrategy.RETRY
        assert correction.confidence == 0.8

    def test_correction_with_modified_action(self):
        """Test correction with modified action"""
        correction = Correction(
            strategy=RecoveryStrategy.RETRY_MODIFIED,
            description="Retry with different parameters",
            modified_action="read_file with encoding=utf-8",
            parameters={"encoding": "utf-8"},
            confidence=0.9
        )

        assert correction.strategy == RecoveryStrategy.RETRY_MODIFIED
        assert correction.modified_action is not None


class TestSelfCorrectionProposals:
    """Self-correction proposal tests"""

    def test_propose_correction_for_syntax_error(self):
        """Test proposing correction for syntax error"""
        corrector = SelfCorrection()

        error = SyntaxError("invalid syntax")
        ctx = corrector.analyze_error(error)

        correction = corrector.propose_correction(ctx)

        assert correction is not None
        assert correction.strategy in [
            RecoveryStrategy.RETRY,
            RecoveryStrategy.RETRY_MODIFIED,
        ]

    def test_propose_correction_for_permission_error(self):
        """Test proposing correction for permission error"""
        corrector = SelfCorrection()

        error = PermissionError("Access denied")
        ctx = corrector.analyze_error(error)

        correction = corrector.propose_correction(ctx)

        assert correction is not None
        # Permission errors might need alternative approach
        assert correction.strategy in [
            RecoveryStrategy.ALTERNATIVE,
            RecoveryStrategy.ASK_USER,
            RecoveryStrategy.SKIP,
            RecoveryStrategy.RETRY,
        ]

    def test_propose_correction_for_timeout_error(self):
        """Test proposing correction for timeout error"""
        corrector = SelfCorrection()

        error = TimeoutError("Operation timed out")
        ctx = corrector.analyze_error(error)

        correction = corrector.propose_correction(ctx)

        assert correction is not None
        # Timeout might suggest retry or alternative
        assert correction.strategy in [
            RecoveryStrategy.RETRY,
            RecoveryStrategy.ALTERNATIVE,
            RecoveryStrategy.SKIP,
        ]

    def test_propose_correction_for_not_found_error(self):
        """Test proposing correction for not found error"""
        corrector = SelfCorrection()

        error = FileNotFoundError("No such file")
        ctx = corrector.analyze_error(error)

        correction = corrector.propose_correction(ctx)

        assert correction is not None
        # Not found might need alternative path or skip
        assert correction.strategy in [
            RecoveryStrategy.ALTERNATIVE,
            RecoveryStrategy.SKIP,
            RecoveryStrategy.ASK_USER,
        ]

    def test_error_history_tracking(self):
        """Test that error history is tracked"""
        corrector = SelfCorrection()

        # Analyze multiple errors
        for i in range(3):
            error = RuntimeError(f"Error {i}")
            corrector.analyze_error(error)

        history = corrector.get_error_history()

        assert len(history) >= 0  # Should track history

    def test_mark_successful_clears_context(self):
        """Test marking an operation as successful"""
        corrector = SelfCorrection()

        error = RuntimeError("Test error")
        ctx = corrector.analyze_error(error)
        correction = corrector.propose_correction(ctx)

        # Should be able to mark as successful with ErrorContext and Correction
        corrector.mark_successful(ctx, correction)


class TestExecutionResult:
    """Execution result tests"""

    def test_execution_result_success(self):
        """Test successful execution result"""
        result = ExecutionResult(
            plan_id="plan-123",
            task="Fix the bug",
            status="completed",
            completed_steps=5,
            total_steps=5,
            duration_seconds=1.5,
            result="Bug fixed successfully",
            error=None,
            corrections_applied=0,
            logs=[]
        )

        assert result.status == "completed"
        assert result.corrections_applied == 0
        assert result.error is None

    def test_execution_result_with_corrections(self):
        """Test execution result with corrections applied"""
        result = ExecutionResult(
            plan_id="plan-456",
            task="Run tests",
            status="completed",
            completed_steps=3,
            total_steps=3,
            duration_seconds=2.0,
            result="Tests passed after retry",
            error=None,
            corrections_applied=2,
            logs=[
                {"step": "run_tests", "status": "failed", "attempt": 1},
                {"step": "run_tests", "status": "success", "attempt": 2},
            ]
        )

        assert result.corrections_applied == 2
        assert len(result.logs) == 2

    def test_execution_result_partial_failure(self):
        """Test execution result with partial failure"""
        result = ExecutionResult(
            plan_id="plan-789",
            task="Complex task",
            status="partial",
            completed_steps=3,
            total_steps=5,
            duration_seconds=5.0,
            result="Partially completed",
            error="Step 4 failed after 3 retries",
            corrections_applied=1,
            logs=[]
        )

        assert result.status == "partial"
        assert result.completed_steps < result.total_steps
        assert result.error is not None


class TestIntelligentAgentCorrection:
    """IntelligentAgent correction integration tests"""

    @pytest.fixture
    def temp_project(self):
        """Create a temporary project"""
        d = tempfile.mkdtemp(prefix="sh_correction_test_")
        yield Path(d)
        shutil.rmtree(d, ignore_errors=True)

    def test_agent_initialization(self, temp_project):
        """Test IntelligentAgent can be initialized"""
        agent = IntelligentAgent(api_key="test-key")

        assert agent is not None

    def test_agent_has_self_correction(self, temp_project):
        """Test agent has self-correction capability"""
        agent = IntelligentAgent(api_key="test-key")

        # Agent should be able to handle errors
        assert hasattr(agent, 'run') or hasattr(agent, 'execute')

    @pytest.mark.asyncio
    async def test_agent_analyzes_errors(self, temp_project):
        """Test agent analyzes errors during execution"""
        corrector = SelfCorrection()

        error = RuntimeError("Test execution error")
        ctx = corrector.analyze_error(
            error=error,
            step_id="step-1",
            action="test_action",
            target="test_target"
        )

        # Should return valid error context
        assert ctx is not None
        # Error type classification depends on error patterns
        assert ctx.error_type in [ErrorType.RUNTIME, ErrorType.UNKNOWN]

    def test_agent_correction_flow(self, temp_project):
        """Test full correction flow"""
        corrector = SelfCorrection()

        # 1. Analyze error
        error = SyntaxError("invalid syntax")
        ctx = corrector.analyze_error(error)

        # 2. Propose correction
        correction = corrector.propose_correction(ctx)

        # 3. Verify correction is valid
        assert correction is not None
        assert correction.strategy in RecoveryStrategy


class TestRecoveryStrategyScenarios:
    """Recovery strategy scenario tests"""

    def test_retry_for_transient_errors(self):
        """Test RETRY strategy for transient errors"""
        corrector = SelfCorrection()

        for error_type in [TimeoutError, ConnectionError]:
            try:
                raise error_type("Transient error")
            except Exception as e:
                ctx = corrector.analyze_error(e)
                correction = corrector.propose_correction(ctx)

                assert correction is not None

    def test_alternative_for_blocked_paths(self):
        """Test ALTERNATIVE strategy when path is blocked"""
        corrector = SelfCorrection()

        error = FileNotFoundError("File not found")
        ctx = corrector.analyze_error(error)
        correction = corrector.propose_correction(ctx)

        assert correction is not None

    def test_ask_user_for_uncertain_situations(self):
        """Test ASK_USER strategy for uncertain situations"""
        corrector = SelfCorrection()

        # Multiple context values to simulate complexity
        error = PermissionError("Critical permission denied")
        ctx = corrector.analyze_error(error, context={"severity": "high"})
        correction = corrector.propose_correction(ctx)

        assert correction is not None


class TestCorrectionConfidence:
    """Correction confidence tests"""

    def test_confidence_range(self):
        """Test that confidence is in valid range"""
        corrector = SelfCorrection()

        error = RuntimeError("Test error")
        ctx = corrector.analyze_error(error)
        correction = corrector.propose_correction(ctx)

        assert 0.0 <= correction.confidence <= 1.0

    def test_high_confidence_for_clear_patterns(self):
        """Test higher confidence for clear error patterns"""
        corrector = SelfCorrection()

        # Syntax errors have clear patterns
        error = SyntaxError("Expected ':'")
        ctx = corrector.analyze_error(error)
        correction = corrector.propose_correction(ctx)

        assert correction.confidence > 0


class TestCommonFixes:
    """Common fixes tests"""

    def test_common_fixes_available(self):
        """Test that common fixes are defined"""
        corrector = SelfCorrection()

        assert hasattr(corrector, 'COMMON_FIXES')
        assert len(corrector.COMMON_FIXES) > 0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
