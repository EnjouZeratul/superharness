"""
Intelligent Agent

Agent with task planning, self-correction, and progress tracking.

Features:
    - Task decomposition: Break complex tasks into steps
    - Execution planning: Generate visible, controllable plans
    - Self-correction: Analyze errors and auto-recover
    - Progress tracking: Real-time progress display

Example:
    >>> from continuum_sdk.agent import IntelligentAgent
    >>> agent = IntelligentAgent()
    >>> plan = await agent.plan("fix bug in auth.py")
    >>> result = await agent.execute_plan(plan)
"""

import asyncio
import json
from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional, List, Dict, Any, Callable, Union, AsyncIterator
from enum import Enum

from .planner import Planner, Plan, Step, StepType, StepStatus
from .self_correction import SelfCorrection, ErrorContext, Correction, RecoveryStrategy
from .progress import ProgressTracker, ProgressEvent, StepLogger
from ..llm import BaseLlmClient, LlmClient, ChatResponse, Message as LlmMessage


class AgentMode(Enum):
    """Agent execution mode."""
    AUTONOMOUS = "autonomous"     # Execute without confirmation
    INTERACTIVE = "interactive"  # Ask for confirmation
    STEP_BY_STEP = "step_by_step"  # Pause after each step


@dataclass
class ExecutionResult:
    """Result of plan execution."""
    plan_id: str
    task: str
    status: str
    completed_steps: int
    total_steps: int
    duration_seconds: float
    result: Optional[str] = None
    error: Optional[str] = None
    corrections_applied: int = 0
    logs: List[Dict[str, Any]] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "plan_id": self.plan_id,
            "task": self.task,
            "status": self.status,
            "completed_steps": self.completed_steps,
            "total_steps": self.total_steps,
            "duration_seconds": self.duration_seconds,
            "result": self.result,
            "error": self.error,
            "corrections_applied": self.corrections_applied,
        }


class IntelligentAgent:
    """
    Agent with planning and self-correction capabilities.

    This agent can:
    - Decompose complex tasks into executable steps
    - Generate visible execution plans
    - Execute steps with automatic error recovery
    - Track and report progress in real-time

    Example:
        >>> agent = IntelligentAgent(api_key="your-key")
        >>> plan = await agent.plan("Fix the login bug in auth.py")
        >>> print(plan.to_dict())  # View the plan
        >>> result = await agent.execute(plan)
        >>> print(f"Completed {result.completed_steps}/{result.total_steps} steps")
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        provider: str = "anthropic",
        model: Optional[str] = None,
        base_url: Optional[str] = None,
        mode: AgentMode = AgentMode.INTERACTIVE,
        max_retries: int = 3,
        on_progress: Optional[Callable[[ProgressEvent], None]] = None,
    ):
        """
        Initialize intelligent agent.

        Args:
            api_key: API key for LLM (uses env var if not provided)
            provider: LLM provider (anthropic, openai, gemini)
            model: Model to use
            base_url: Optional custom base URL
            mode: Execution mode
            max_retries: Max retries per step
            on_progress: Progress callback
        """
        self.api_key = api_key
        self.provider = provider
        self.model = model
        self.base_url = base_url
        self.mode = mode
        self.max_retries = max_retries

        # Initialize components
        self._llm_client: Optional[BaseLlmClient] = None
        self.planner = Planner()
        self.correction = SelfCorrection()
        self.tracker = ProgressTracker()
        self.logger = StepLogger()

        # State
        self.current_plan: Optional[Plan] = None
        self.context: Dict[str, Any] = {}

        # Register progress callback
        if on_progress:
            self.tracker.on_progress(on_progress)

    def _get_llm_client(self) -> BaseLlmClient:
        """Get or create LLM client."""
        if self._llm_client is None:
            if not self.api_key:
                import os
                self.api_key = (
                    os.environ.get("CONTINUUM_API_KEY")
                    or os.environ.get("CONTINUUM_API_KEY")
                    or os.environ.get("ANTHROPIC_API_KEY")
                )

            if not self.api_key:
                raise ValueError("API key required. Set CONTINUUM_API_KEY or pass api_key parameter.")

            self._llm_client = LlmClient.for_provider(
                provider=self.provider,
                api_key=self.api_key,
                base_url=self.base_url,
                model=self.model,
            )

        return self._llm_client

    async def plan(self, task: str, context: Optional[Dict[str, Any]] = None) -> Plan:
        """
        Create execution plan for a task.

        Args:
            task: Task description
            context: Additional context

        Returns:
            Plan with steps
        """
        # Set LLM client for planner
        self.planner.llm_client = self._get_llm_client()
        self.correction.llm_client = self._get_llm_client()

        # Create plan
        plan = await self.planner.plan(task, context)
        self.current_plan = plan

        return plan

    async def execute(
        self,
        plan: Optional[Plan] = None,
        task: Optional[str] = None,
        on_step_start: Optional[Callable[[Step], bool]] = None,
        on_step_complete: Optional[Callable[[Step], None]] = None,
        on_error: Optional[Callable[[Step, ErrorContext], bool]] = None,
    ) -> ExecutionResult:
        """
        Execute a plan.

        Args:
            plan: Plan to execute (uses current plan if not provided)
            task: Task to plan and execute (creates new plan)
            on_step_start: Called before each step, return False to skip
            on_step_complete: Called after each step completes
            on_error: Called on error, return False to abort

        Returns:
            ExecutionResult
        """
        # Create plan if task provided
        if task:
            plan = await self.plan(task)
        elif plan is None:
            plan = self.current_plan

        if not plan:
            raise ValueError("No plan to execute")

        start_time = datetime.now()
        self.tracker.start(len(plan.steps))

        # Track corrections
        corrections_applied = 0

        # Execute steps
        while True:
            # Get next pending steps
            pending = plan.get_pending_steps()

            if not pending:
                # Check if all done
                progress = plan.get_progress()
                if progress["progress_percent"] >= 100:
                    break

                # Check for failed steps that might block
                failed_blocking = [
                    s for s in plan.steps
                    if s.status == StepStatus.FAILED
                ]
                if failed_blocking:
                    break

                # Wait for running steps
                await asyncio.sleep(0.1)
                continue

            # Execute first pending step
            step = pending[0]

            # Check callback
            if on_step_start:
                try:
                    if not on_step_start(step):
                        step.status = StepStatus.SKIPPED
                        self.tracker.update_step(step.id, "skipped", step.description)
                        continue
                except Exception:
                    pass

            # Interactive mode: ask for confirmation
            if self.mode == AgentMode.INTERACTIVE:
                # In real implementation, would prompt user
                pass

            # Execute step
            step.status = StepStatus.RUNNING
            self.tracker.update_step(step.id, "running", step.description)

            try:
                result = await self._execute_step(step)

                step.result = result
                step.status = StepStatus.COMPLETED
                self.tracker.update_step(step.id, "completed", step.description)
                self.logger.log(step.id, "completed", result or "")

                if on_step_complete:
                    try:
                        on_step_complete(step)
                    except Exception:
                        pass

            except Exception as e:
                # Analyze error
                error_ctx = self.correction.analyze_error(
                    error=e,
                    step_id=step.id,
                    action=step.action,
                    target=step.target,
                )

                self.logger.log(step.id, "error", str(e), {"error_type": error_ctx.error_type.value})

                # Propose correction
                correction = self.correction.propose_correction(error_ctx, self.context)

                # Handle based on strategy
                if correction.strategy == RecoveryStrategy.RETRY:
                    if step.retry_count < step.max_retries:
                        step.retry_count += 1
                        step.status = StepStatus.RETRYING
                        self.logger.log(step.id, "retrying", f"Attempt {step.retry_count}")
                        continue

                elif correction.strategy == RecoveryStrategy.RETRY_MODIFIED:
                    if step.retry_count < step.max_retries and correction.modified_action:
                        step.retry_count += 1
                        step.action = correction.modified_action
                        step.status = StepStatus.RETRYING
                        self.logger.log(step.id, "retrying", f"Modified: {correction.description}")
                        corrections_applied += 1
                        continue

                elif correction.strategy == RecoveryStrategy.SKIP:
                    step.status = StepStatus.SKIPPED
                    self.tracker.update_step(step.id, "skipped", step.description)
                    continue

                elif correction.strategy == RecoveryStrategy.ASK_USER:
                    # Callback for user input
                    if on_error:
                        try:
                            if not on_error(step, error_ctx):
                                step.status = StepStatus.FAILED
                                step.error = str(e)
                                break  # Abort
                        except Exception:
                            pass

                # Mark as failed
                step.status = StepStatus.FAILED
                step.error = str(e)
                self.tracker.update_step(step.id, "failed", step.description)

                if on_error:
                    try:
                        on_error(step, error_ctx)
                    except Exception:
                        pass

                # Check if should abort
                if correction.strategy == RecoveryStrategy.ABORT:
                    break

        # Calculate result
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()

        progress = plan.get_progress()
        status = "completed" if progress["progress_percent"] >= 100 else "partial"

        return ExecutionResult(
            plan_id=plan.id,
            task=plan.task,
            status=status,
            completed_steps=progress["completed"],
            total_steps=progress["total"],
            duration_seconds=duration,
            corrections_applied=corrections_applied,
            logs=self.logger.to_dict(),
        )

    async def _execute_step(self, step: Step) -> str:
        """
        Execute a single step.

        Args:
            step: Step to execute

        Returns:
            Execution result
        """
        action = step.action.lower()

        # Import tools
        from ..tools import BashTool, ReadTool, WriteTool, EditTool, GrepTool, GlobTool

        # Execute based on step type
        if step.type == StepType.SEARCH:
            grep = GrepTool()
            pattern = self._extract_pattern(step.action)
            result = grep.search(pattern or step.action, path=".")

        elif step.type == StepType.READ:
            reader = ReadTool()
            target = step.target or self._extract_file(step.action)
            if target:
                result = reader.read(target)
            else:
                result = await self._llm_analyze(step)

        elif step.type == StepType.EDIT:
            editor = EditTool()
            # Would need specific old/new strings
            result = await self._llm_edit(step)

        elif step.type == StepType.TEST:
            bash = BashTool()
            result = bash.run("pytest tests/ -v --tb=short")

        elif step.type == StepType.ANALYZE:
            result = await self._llm_analyze(step)

        elif step.type == StepType.VERIFY:
            result = await self._llm_verify(step)

        else:
            # Generic LLM execution
            result = await self._llm_execute(step)

        return result.content if hasattr(result, 'content') else str(result)

    async def _llm_analyze(self, step: Step) -> str:
        """Use LLM for analysis."""
        client = self._get_llm_client()

        system_prompt = f"""You are analyzing code as part of a task execution.
Current task: {self.current_plan.task if self.current_plan else step.description}
Current step: {step.description}

Provide a concise analysis focusing on:
1. What you found
2. What needs to be done
3. Any issues or risks"""

        messages = [LLMMessage.user(step.action)]

        response = await client.chat(
            messages=messages,
            system_prompt=system_prompt,
            temperature=0.3,
        )

        return response.content

    async def _llm_edit(self, step: Step) -> str:
        """Use LLM to plan an edit."""
        # In real implementation, would analyze and generate edit
        client = self._get_llm_client()

        system_prompt = "You are planning code edits. Provide specific file changes needed."

        messages = [LLMMessage.user(step.action)]

        response = await client.chat(
            messages=messages,
            system_prompt=system_prompt,
            temperature=0.2,
        )

        return response.content

    async def _llm_verify(self, step: Step) -> str:
        """Verify changes."""
        client = self._get_llm_client()

        system_prompt = "Verify that the changes resolved the issue. Check for regressions."

        messages = [LLMMessage.user(step.action)]

        response = await client.chat(
            messages=messages,
            system_prompt=system_prompt,
            temperature=0.2,
        )

        return response.content

    async def _llm_execute(self, step: Step) -> str:
        """Generic LLM execution."""
        client = self._get_llm_client()

        messages = [LLMMessage.user(step.action)]

        response = await client.chat(messages=messages, temperature=0.3)
        return response.content

    def _extract_pattern(self, action: str) -> Optional[str]:
        """Extract search pattern from action."""
        # Simple extraction: look for quoted strings or key terms
        import re
        match = re.search(r'["\']([^"\']+)["\']', action)
        if match:
            return match.group(1)

        # Look for "for X" or "find X" patterns
        match = re.search(r'(?:for|find|search)\s+(\w+)', action, re.IGNORECASE)
        if match:
            return match.group(1)

        return None

    def _extract_file(self, action: str) -> Optional[str]:
        """Extract file path from action."""
        import re
        # Look for file paths
        match = re.search(r'[\w/.-]+\.\w+', action)
        if match:
            return match.group(0)
        return None

    async def run(self, task: str, **kwargs) -> ExecutionResult:
        """
        One-shot: plan and execute a task.

        Args:
            task: Task description
            **kwargs: Additional arguments

        Returns:
            ExecutionResult
        """
        plan = await self.plan(task)
        return await self.execute(plan, **kwargs)

    def get_progress(self) -> Dict[str, Any]:
        """Get current progress."""
        return self.tracker.get_progress()

    def get_progress_text(self) -> str:
        """Get human-readable progress."""
        return self.tracker.get_progress_text()

    def get_plan_summary(self) -> Optional[str]:
        """Get summary of current plan."""
        if not self.current_plan:
            return None

        lines = [f"Plan: {self.current_plan.task}", ""]
        for step in self.current_plan.steps:
            status_icon = {
                StepStatus.PENDING: "○",
                StepStatus.RUNNING: "◐",
                StepStatus.COMPLETED: "●",
                StepStatus.FAILED: "✗",
                StepStatus.SKIPPED: "◌",
            }.get(step.status, "?")

            lines.append(f"  {status_icon} [{step.id}] {step.description}")

        progress = self.current_plan.get_progress()
        lines.append("")
        lines.append(f"Progress: {progress['completed']}/{progress['total']} ({progress['progress_percent']:.0f}%)")

        return "\n".join(lines)
