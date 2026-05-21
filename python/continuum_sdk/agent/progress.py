"""
Progress Tracker

Real-time task progress tracking and reporting.

Features:
    - Progress calculation
    - Status updates
    - ETA estimation
    - Event callbacks
"""

import time
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Optional, List, Dict, Any, Callable
from enum import Enum
import json


class ProgressState(Enum):
    """Progress state."""
    IDLE = "idle"
    RUNNING = "running"
    PAUSED = "paused"
    COMPLETED = "completed"
    FAILED = "failed"


@dataclass
class ProgressEvent:
    """Progress event data."""
    step_id: str
    step_description: str
    status: str
    progress_percent: float
    elapsed_time: float
    estimated_remaining: Optional[float]
    message: Optional[str] = None
    timestamp: datetime = field(default_factory=datetime.now)


class ProgressTracker:
    """
    Real-time progress tracker.

    Tracks step execution, calculates progress, estimates remaining time.

    Example:
        >>> tracker = ProgressTracker()
        >>> tracker.start(total_steps=5)
        >>> tracker.update_step("s1", "completed")
        >>> print(tracker.get_progress_text())
        '[1/5] 20% complete'
    """

    def __init__(self):
        self.total_steps: int = 0
        self.completed_steps: int = 0
        self.failed_steps: int = 0
        self.skipped_steps: int = 0
        self.current_step: Optional[str] = None
        self.state: ProgressState = ProgressState.IDLE
        self.start_time: Optional[datetime] = None
        self.end_time: Optional[datetime] = None
        self.step_times: Dict[str, float] = {}
        self.callbacks: List[Callable[[ProgressEvent], None]] = []
        self.events: List[ProgressEvent] = []

    def start(self, total_steps: int) -> None:
        """Start tracking."""
        self.total_steps = total_steps
        self.completed_steps = 0
        self.failed_steps = 0
        self.skipped_steps = 0
        self.state = ProgressState.RUNNING
        self.start_time = datetime.now()
        self.end_time = None
        self.events = []

        self._notify(ProgressEvent(
            step_id="",
            step_description="Started",
            status="started",
            progress_percent=0,
            elapsed_time=0,
            estimated_remaining=None,
        ))

    def update_step(
        self,
        step_id: str,
        status: str,
        description: Optional[str] = None,
        message: Optional[str] = None,
    ) -> None:
        """Update step status."""
        self.current_step = step_id

        if status in ("completed", "done"):
            self.completed_steps += 1
            self.step_times[step_id] = time.time()
        elif status == "failed":
            self.failed_steps += 1
        elif status == "skipped":
            self.skipped_steps += 1

        # Check if done
        if self.completed_steps + self.failed_steps + self.skipped_steps >= self.total_steps:
            self.state = ProgressState.COMPLETED if self.failed_steps == 0 else ProgressState.FAILED
            self.end_time = datetime.now()

        # Calculate progress
        progress = self.get_progress()
        elapsed = self.get_elapsed_time()
        remaining = self.estimate_remaining()

        event = ProgressEvent(
            step_id=step_id,
            step_description=description or step_id,
            status=status,
            progress_percent=progress["percent"],
            elapsed_time=elapsed,
            estimated_remaining=remaining,
            message=message,
        )

        self.events.append(event)
        self._notify(event)

    def pause(self) -> None:
        """Pause tracking."""
        self.state = ProgressState.PAUSED

    def resume(self) -> None:
        """Resume tracking."""
        if self.state == ProgressState.PAUSED:
            self.state = ProgressState.RUNNING

    def get_progress(self) -> Dict[str, Any]:
        """Get current progress details."""
        done = self.completed_steps + self.skipped_steps
        total = self.total_steps

        return {
            "total_steps": total,
            "completed": self.completed_steps,
            "failed": self.failed_steps,
            "skipped": self.skipped_steps,
            "pending": total - done - self.failed_steps,
            "percent": (done / total * 100) if total > 0 else 0,
            "state": self.state.value,
            "current_step": self.current_step,
        }

    def get_elapsed_time(self) -> float:
        """Get elapsed time in seconds."""
        if not self.start_time:
            return 0

        end = self.end_time or datetime.now()
        return (end - self.start_time).total_seconds()

    def estimate_remaining(self) -> Optional[float]:
        """Estimate remaining time in seconds."""
        if self.completed_steps == 0:
            return None

        elapsed = self.get_elapsed_time()
        avg_time_per_step = elapsed / self.completed_steps
        remaining_steps = self.total_steps - self.completed_steps - self.skipped_steps

        return avg_time_per_step * remaining_steps

    def get_progress_text(self) -> str:
        """Get human-readable progress text."""
        progress = self.get_progress()
        elapsed = self.get_elapsed_time()
        remaining = self.estimate_remaining()

        parts = [
            f"[{self.completed_steps + self.skipped_steps}/{self.total_steps}]",
            f"{progress['percent']:.0f}%",
            f"in {self._format_time(elapsed)}",
        ]

        if remaining:
            parts.append(f"ETA: {self._format_time(remaining)}")

        return " ".join(parts)

    def get_status_bar(self, width: int = 40) -> str:
        """Get progress bar string."""
        progress = self.get_progress()
        percent = progress["percent"]

        filled = int(width * percent / 100)
        empty = width - filled

        bar = "█" * filled + "░" * empty
        return f"[{bar}] {percent:.0f}%"

    def _format_time(self, seconds: float) -> str:
        """Format time in human-readable form."""
        if seconds < 60:
            return f"{seconds:.0f}s"
        elif seconds < 3600:
            minutes = int(seconds / 60)
            secs = int(seconds % 60)
            return f"{minutes}m {secs}s"
        else:
            hours = int(seconds / 3600)
            minutes = int((seconds % 3600) / 60)
            return f"{hours}h {minutes}m"

    def on_progress(self, callback: Callable[[ProgressEvent], None]) -> None:
        """Register progress callback."""
        self.callbacks.append(callback)

    def _notify(self, event: ProgressEvent) -> None:
        """Notify all callbacks."""
        for callback in self.callbacks:
            try:
                callback(event)
            except Exception:
                pass

    def to_dict(self) -> Dict[str, Any]:
        """Export progress as dict."""
        return {
            "state": self.state.value,
            "total_steps": self.total_steps,
            "completed_steps": self.completed_steps,
            "failed_steps": self.failed_steps,
            "skipped_steps": self.skipped_steps,
            "current_step": self.current_step,
            "elapsed_time": self.get_elapsed_time(),
            "estimated_remaining": self.estimate_remaining(),
            "progress_percent": self.get_progress()["percent"],
            "events": [
                {
                    "step_id": e.step_id,
                    "status": e.status,
                    "progress": e.progress_percent,
                    "time": e.timestamp.isoformat(),
                }
                for e in self.events
            ],
        }


class StepLogger:
    """
    Logs step execution details.

    Example:
        >>> logger = StepLogger()
        >>> logger.log("s1", "started", "Searching for bug...")
        >>> logger.log("s1", "completed", "Found 3 files")
    """

    def __init__(self):
        self.logs: List[Dict[str, Any]] = []

    def log(
        self,
        step_id: str,
        status: str,
        message: str,
        details: Optional[Dict[str, Any]] = None,
    ) -> None:
        """Log step event."""
        entry = {
            "step_id": step_id,
            "status": status,
            "message": message,
            "timestamp": datetime.now().isoformat(),
            "details": details or {},
        }
        self.logs.append(entry)

    def get_step_logs(self, step_id: str) -> List[Dict[str, Any]]:
        """Get all logs for a step."""
        return [l for l in self.logs if l["step_id"] == step_id]

    def get_recent_logs(self, count: int = 10) -> List[Dict[str, Any]]:
        """Get most recent logs."""
        return self.logs[-count:]

    def to_dict(self) -> List[Dict[str, Any]]:
        """Export logs."""
        return self.logs.copy()
