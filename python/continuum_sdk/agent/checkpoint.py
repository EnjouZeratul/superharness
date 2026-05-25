"""Checkpoint Interface Layer

Python wrapper for Rust checkpoint functionality.

Features:
    - Session state persistence: Save complete session state
    - Crash recovery support: Resume after unexpected termination
    - Atomic checkpoint writes: Safe persistence operations
    - Checkpoint listing and deletion: Manage checkpoint lifecycle
    - Integrity verification: Ensure checkpoint consistency

Use Cases:
    - Long-running tasks: Save progress periodically
    - Crash recovery: Resume after process termination
    - Testing: Capture and restore agent state
    - Debugging: Inspect session state at specific points

Quick Start:
    >>> from continuum_sdk.agent.checkpoint import CheckpointClient
    >>>
    >>> client = CheckpointClient()
    >>>
    >>> # Save checkpoint
    >>> checkpoint_id = client.save("session-001", {
    ...     "messages": [{"role": "user", "content": "Hello"}],
    ...     "state": {"step": 1}
    ... })
    >>> print(f"Saved checkpoint: {checkpoint_id}")
    >>>
    >>> # Load checkpoint
    >>> data = client.load("session-001")
    >>> print(data["messages"])

Checkpoint Lifecycle:
    >>> # Create multiple checkpoints
    >>> cp1 = client.save("session-001", {"iteration": 1})
    >>> cp2 = client.save("session-001", {"iteration": 2})
    >>> cp3 = client.save("session-001", {"iteration": 3})
    >>>
    >>> # List all checkpoints
    >>> checkpoints = client.list("session-001")
    >>> for cp in checkpoints:
    ...     print(f"{cp.checkpoint_id}: iteration {cp.iteration}")
    >>>
    >>> # Load latest
    >>> latest = client.load_latest("session-001")
    >>>
    >>> # Delete old checkpoints
    >>> client.delete("session-001", cp1)

Crash Recovery Pattern:
    >>> import os
    >>>
    >>> # Check for existing checkpoint on startup
    >>> def resume_or_start(session_id):
    ...     client = CheckpointClient()
    ...     existing = client.load_latest(session_id)
    ...     if existing:
    ...         print(f"Resuming from checkpoint: {existing['checkpoint_id']}")
    ...         return existing
    ...     return {"iteration": 0, "messages": []}
    >>>
    >>> # Save progress periodically
    >>> def save_progress(session_id, state, iteration):
    ...     client = CheckpointClient()
    ...     client.save(session_id, {
    ...         **state,
    ...         "iteration": iteration,
    ...         "timestamp": datetime.now().isoformat()
    ...     })

Checkpoint Metadata:
    >>> @dataclass
    >>> class CheckpointMeta:
    ...     checkpoint_id: str      # Unique identifier
    ...     session_id: str         # Session this belongs to
    ...     created_at: datetime    # When created
    ...     trigger: str            # Why it was created (manual, periodic, error)
    ...     iteration: int          # Execution iteration at save time

Storage:
    Checkpoints are stored in:
    - Default: ~/.continuum/checkpoints/{session_id}/{checkpoint_id}.json
    - Custom: Specify storage_path in CheckpointClient constructor

Performance:
    - Atomic writes: Checkpoint integrity guaranteed
    - Compression: Large states are compressed
    - Incremental: Only changed state is stored (with Rust binding)

See Also:
    SessionManager: Higher-level session management
    CheckpointMeta: Checkpoint metadata structure
"""

import json
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any

# Import Rust binding
try:
    from sh_python import CheckpointSystem as RustCheckpointSystem
    HAS_RUST_BINDING = True
except ImportError:
    HAS_RUST_BINDING = False
    # Define placeholder for type annotation
    class RustCheckpointSystem:
        pass


@dataclass
class CheckpointMeta:
    """Checkpoint metadata."""
    checkpoint_id: str
    session_id: str
    created_at: datetime
    trigger: str
    iteration: int

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "CheckpointMeta":
        return cls(
            checkpoint_id=data.get("checkpoint_id", ""),
            session_id=data.get("session_id", ""),
            created_at=datetime.fromisoformat(data.get("created_at", datetime.now().isoformat())),
            trigger=data.get("trigger", "manual"),
            iteration=data.get("iteration", 0),
        )


class CheckpointClient:
    """Python wrapper for Rust CheckpointSystem.

    Provides checkpoint save/load operations for session persistence
    and crash recovery.

    Example:
        >>> client = CheckpointClient()
        >>> checkpoint_id = client.save("my-session", {"state": "active"})
        >>> data = client.load("my-session")
        >>> print(data)
    """

    _system: RustCheckpointSystem | None = None

    def __init__(self, storage_path: Path | None = None):
        """Initialize checkpoint client.

        Args:
            storage_path: Directory for checkpoint storage.
                         Default: ~/.continuum/checkpoints/
        """
        if HAS_RUST_BINDING:
            path_str = str(storage_path) if storage_path else None
            self._system = RustCheckpointSystem(path_str)
        else:
            self._storage_path = storage_path or Path.home() / ".continuum" / "checkpoints"
            self._storage_path.mkdir(parents=True, exist_ok=True)

    def _check_binding(self) -> None:
        """Check if Rust binding is available."""
        if not self._system:
            raise RuntimeError(
                "CheckpointClient requires Rust binding. "
                "Ensure sh_python.pyd is in the package directory."
            )

    def save(self, session_id: str, state: dict[str, Any]) -> str:
        """Save checkpoint for session.

        Args:
            session_id: Session identifier
            state: Session state to persist (will be JSON serialized)

        Returns:
            Checkpoint ID
        """
        self._check_binding()
        data_json = json.dumps(state)
        return self._system.save(session_id, data_json)

    def load(
        self,
        session_id: str,
        checkpoint_id: str | None = None
    ) -> dict[str, Any] | None:
        """Load checkpoint for session.

        Args:
            session_id: Session identifier
            checkpoint_id: Specific checkpoint ID (optional, loads latest if None)

        Returns:
            Loaded state, or None if not found
        """
        self._check_binding()
        result = self._system.load(session_id, checkpoint_id)
        if result:
            try:
                return json.loads(result)
            except json.JSONDecodeError:
                return {"raw": result}
        return None

    def list(self, session_id: str) -> list[str]:
        """List all checkpoints for session.

        Args:
            session_id: Session identifier

        Returns:
            List of checkpoint IDs
        """
        self._check_binding()
        return self._system.list(session_id)

    def delete(self, session_id: str, checkpoint_id: str) -> bool:
        """Delete specific checkpoint.

        Args:
            session_id: Session identifier
            checkpoint_id: Checkpoint to delete

        Returns:
            True if deleted successfully
        """
        self._check_binding()
        return self._system.delete(session_id, checkpoint_id)

    def has_checkpoints(self, session_id: str) -> bool:
        """Check if session has any checkpoints.

        Args:
            session_id: Session identifier

        Returns:
            True if checkpoints exist
        """
        return len(self.list(session_id)) > 0

    def clear_session(self, session_id: str) -> int:
        """Delete all checkpoints for session.

        Args:
            session_id: Session identifier

        Returns:
            Number of checkpoints deleted
        """
        checkpoints = self.list(session_id)
        deleted = 0
        for cp_id in checkpoints:
            if self.delete(session_id, cp_id):
                deleted += 1
        return deleted
