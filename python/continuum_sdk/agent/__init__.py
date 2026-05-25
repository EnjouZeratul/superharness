"""
Agent Module

Provides Agent classes for interacting with Continuum.

Classes:
    - Agent: Basic agent with LLM integration
    - IntelligentAgent: Agent with planning and self-correction
    - Session: Conversation session management

Components:
    - Planner: Task decomposition and planning
    - SelfCorrection: Error analysis and recovery
    - ProgressTracker: Real-time progress tracking
"""

from .runtime import Agent, AgentConfig, AgentState
from .session import Session, Message, MessageRole
from .planner import (
    Planner,
    Plan,
    Step,
    StepType,
    StepStatus,
)
from .self_correction import (
    SelfCorrection,
    ErrorContext,
    ErrorType,
    Correction,
    RecoveryStrategy,
)
from .progress import (
    ProgressTracker,
    ProgressEvent,
    ProgressState,
    StepLogger,
)
from .intelligent import IntelligentAgent, AgentMode, ExecutionResult
from .task_completion import (
    TaskCompletionDetector,
    CompletionStatus,
    CompletionMarker,
    TaskRecord,
)
from .checkpoint import (
    CheckpointClient,
    CheckpointMeta,
)

__all__ = [
    # Core Agent
    "Agent",
    "AgentConfig",
    "AgentState",
    "Session",
    "Message",
    "MessageRole",
    # Intelligent Agent
    "IntelligentAgent",
    "AgentMode",
    "ExecutionResult",
    # Task Completion
    "TaskCompletionDetector",
    "CompletionStatus",
    "CompletionMarker",
    "TaskRecord",
    # Checkpoint
    "CheckpointClient",
    "CheckpointMeta",
    # Planner
    "Planner",
    "Plan",
    "Step",
    "StepType",
    "StepStatus",
    # Self-Correction
    "SelfCorrection",
    "ErrorContext",
    "ErrorType",
    "Correction",
    "RecoveryStrategy",
    # Progress
    "ProgressTracker",
    "ProgressEvent",
    "ProgressState",
    "StepLogger",
]