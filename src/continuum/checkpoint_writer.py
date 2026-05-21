"""
CheckpointWriter - Atomic Checkpoint Persistence for Continuum

This module provides atomic write guarantees for checkpoint persistence,
ensuring crash safety across Windows and Linux platforms.

Key Features:
- Atomic write via temp file + rename
- Cross-platform support (Windows atomic replace workaround)
- SHA-256 checksum validation
- Write verification with read-back
- Crash recovery with corruption detection

Usage:
    from continuum.checkpoint_writer import CheckpointWriter

    writer = CheckpointWriter()
    await writer.save_checkpoint(checkpoint_data, "session_123")

    # Load with validation
    checkpoint, error = await writer.load_checkpoint("session_123")
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import tempfile
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any, Optional, Tuple

import asyncio
from pydantic import BaseModel, Field, field_validator


# ============================================================================
# Constants and Configuration
# ============================================================================

CHECKPOINT_VERSION = "1.0"
TEMP_FILE_PREFIX = ".tmp_checkpoint_"
BACKUP_FILE_SUFFIX = ".backup"
CHECKSUM_FIELD = "_checksum"
VERSION_FIELD = "_version"


# ============================================================================
# Error Definitions
# ============================================================================

class CheckpointError(Exception):
    """Base exception for checkpoint operations."""
    pass


class CheckpointWriteError(CheckpointError):
    """Failed to write checkpoint."""
    pass


class CheckpointValidationError(CheckpointError):
    """Checkpoint validation failed."""
    pass


class CheckpointCorruptedError(CheckpointError):
    """Checkpoint file is corrupted."""
    pass


class CheckpointNotFoundError(CheckpointError):
    """Checkpoint file not found."""
    pass


# ============================================================================
# Pydantic Models for Validation
# ============================================================================

class CheckpointData(BaseModel):
    """Pydantic model for checkpoint data validation."""

    checkpoint_id: str = Field(default_factory=lambda: str(uuid.uuid4())[:8])
    session_id: str
    created_at: datetime = Field(default_factory=datetime.now)
    trigger: str = "manual"
    iteration: int = Field(ge=0, default=0)
    messages: list[dict] = Field(default_factory=list)
    tool_calls_pending: list[dict] = Field(default_factory=list)
    tool_results: dict = Field(default_factory=dict)
    tokens_used: int = Field(ge=0, default=0)
    cost_estimate: float = Field(ge=0, default=0.0)
    resume_hint: Optional[str] = None

    @field_validator('messages')
    @classmethod
    def validate_messages(cls, v: list) -> list:
        """Validate message format."""
        for i, msg in enumerate(v):
            if not isinstance(msg, dict):
                raise ValueError(f"Message {i} must be a dict")
            if 'role' not in msg:
                raise ValueError(f"Message {i} missing 'role' field")
            if msg['role'] not in ['system', 'user', 'assistant', 'tool']:
                raise ValueError(f"Message {i} has invalid role: {msg['role']}")
        return v


# ============================================================================
# Checksum Utilities
# ============================================================================

class ChecksumUtils:
    """Utilities for checksum computation and verification."""

    @staticmethod
    def compute_checksum(data: dict) -> str:
        """Compute SHA-256 checksum for checkpoint data.

        Args:
            data: Dictionary data to checksum

        Returns:
            Hexadecimal checksum string
        """
        # Create canonical JSON (sorted keys, no whitespace)
        canonical_json = json.dumps(
            data,
            sort_keys=True,
            ensure_ascii=False,
            separators=(',', ':')
        )
        return hashlib.sha256(canonical_json.encode('utf-8')).hexdigest()

    @staticmethod
    def add_checksum(data: dict) -> dict:
        """Add checksum to checkpoint data.

        Args:
            data: Checkpoint data dictionary

        Returns:
            Copy of data with checksum and version added
        """
        data_copy = data.copy()
        # Remove existing checksum for fresh computation
        data_copy.pop(CHECKSUM_FIELD, None)
        data_copy.pop(VERSION_FIELD, None)

        checksum = ChecksumUtils.compute_checksum(data_copy)
        data_copy[CHECKSUM_FIELD] = checksum
        data_copy[VERSION_FIELD] = CHECKPOINT_VERSION
        return data_copy

    @staticmethod
    def verify_checksum(data: dict) -> Tuple[bool, Optional[str]]:
        """Verify checksum in checkpoint data.

        Args:
            data: Checkpoint data dictionary with checksum

        Returns:
            Tuple of (is_valid, error_message)
        """
        if CHECKSUM_FIELD not in data:
            return False, "Missing checksum field"

        if VERSION_FIELD not in data:
            return False, "Missing version field"

        if data[VERSION_FIELD] != CHECKPOINT_VERSION:
            return False, f"Version mismatch: expected {CHECKPOINT_VERSION}, got {data[VERSION_FIELD]}"

        data_copy = data.copy()
        expected_checksum = data_copy.pop(CHECKSUM_FIELD)
        data_copy.pop(VERSION_FIELD)

        actual_checksum = ChecksumUtils.compute_checksum(data_copy)
        if expected_checksum != actual_checksum:
            return False, f"Checksum mismatch: expected {expected_checksum[:16]}..., got {actual_checksum[:16]}..."

        return True, None


# ============================================================================
# Atomic File Operations
# ============================================================================

class AtomicFileWriter:
    """Cross-platform atomic file writer.

    Uses temp file + rename pattern for atomicity.
    On Windows, uses os.replace() which is atomic on the same filesystem.
    On Unix, os.rename() is atomic on the same filesystem.
    """

    def __init__(self, sync_on_write: bool = True, verify_write: bool = True):
        """Initialize atomic file writer.

        Args:
            sync_on_write: Whether to call fsync after write (default True)
            verify_write: Whether to verify write by reading back (default True)
        """
        self.sync_on_write = sync_on_write
        self.verify_write = verify_write

    def write_atomic(
        self,
        filepath: Path,
        content: str,
        encoding: str = 'utf-8'
    ) -> Tuple[bool, Optional[str]]:
        """Write content to file atomically.

        Args:
            filepath: Target file path
            content: Content to write
            encoding: File encoding (default utf-8)

        Returns:
            Tuple of (success, error_message)
        """
        filepath = Path(filepath)
        parent_dir = filepath.parent

        # Ensure parent directory exists
        parent_dir.mkdir(parents=True, exist_ok=True)

        # Generate temp file in same directory (for same-filesystem rename)
        temp_filename = f"{TEMP_FILE_PREFIX}{uuid.uuid4().hex}"
        temp_path = parent_dir / temp_filename

        try:
            # Write to temp file
            with open(temp_path, 'w', encoding=encoding) as f:
                f.write(content)
                # Ensure data is written to disk
                if self.sync_on_write:
                    f.flush()
                    os.fsync(f.fileno())

            # Verify write by reading back
            if self.verify_write:
                with open(temp_path, 'r', encoding=encoding) as f:
                    written_content = f.read()
                    if written_content != content:
                        raise CheckpointWriteError(
                            f"Write verification failed: content mismatch "
                            f"(expected {len(content)} bytes, got {len(written_content)})"
                        )

            # Atomic rename (works on both Windows and Unix)
            # On Windows, os.replace() is atomic on NTFS
            # On Unix, os.replace() uses rename() which is atomic
            os.replace(temp_path, filepath)

            # Sync directory to ensure rename is persisted
            # On Windows, directory sync via os.open() is not supported
            # Windows NTFS guarantees atomicity of os.replace() without explicit sync
            if self.sync_on_write and os.name != 'nt':
                # Unix: sync directory to ensure rename is persisted
                if hasattr(os, 'O_DIRECTORY'):
                    parent_fd = os.open(parent_dir, os.O_RDONLY | os.O_DIRECTORY)
                    try:
                        os.fsync(parent_fd)
                    finally:
                        os.close(parent_fd)

            return True, None

        except Exception as e:
            # Clean up temp file on failure
            self._safe_unlink(temp_path)
            return False, str(e)

    def _safe_unlink(self, path: Path):
        """Safely unlink a file, ignoring errors."""
        try:
            path.unlink(missing_ok=True)
        except Exception:
            pass


# ============================================================================
# Checkpoint Writer
# ============================================================================

class CheckpointWriter:
    """Atomic checkpoint writer with crash safety guarantees.

    Features:
    - Atomic write via temp file + rename
    - SHA-256 checksum validation
    - Write verification with read-back
    - Automatic backup of previous checkpoint
    - Crash recovery with corruption detection

    Example:
        writer = CheckpointWriter("~/.continuum/sessions")

        # Save checkpoint
        success, error = writer.save_checkpoint(
            {"session_id": "abc123", "messages": [...]},
            "abc123",
            trigger="periodic"
        )

        # Load checkpoint
        checkpoint, error = writer.load_checkpoint("abc123")
    """

    def __init__(
        self,
        storage_path: str = "~/.continuum/sessions",
        max_backups: int = 3,
        sync_on_write: bool = True,
        verify_write: bool = True
    ):
        """Initialize checkpoint writer.

        Args:
            storage_path: Base storage path for checkpoints
            max_backups: Maximum number of backup files to keep
            sync_on_write: Whether to call fsync after write
            verify_write: Whether to verify write by reading back
        """
        self.storage_path = Path(storage_path).expanduser()
        self.max_backups = max_backups
        self.atomic_writer = AtomicFileWriter(sync_on_write, verify_write)

        # Ensure storage directory exists
        self.storage_path.mkdir(parents=True, exist_ok=True)

    def save_checkpoint(
        self,
        checkpoint_data: dict,
        session_id: str,
        trigger: str = "manual"
    ) -> Tuple[bool, Optional[str]]:
        """Save checkpoint atomically with checksum.

        Args:
            checkpoint_data: Checkpoint data dictionary
            session_id: Session identifier
            trigger: Trigger reason for checkpoint

        Returns:
            Tuple of (success, error_message)
        """
        # Validate checkpoint structure
        try:
            # Add metadata
            checkpoint_data['session_id'] = session_id
            checkpoint_data['trigger'] = trigger
            if 'created_at' not in checkpoint_data:
                checkpoint_data['created_at'] = datetime.now().isoformat()
            if 'checkpoint_id' not in checkpoint_data:
                checkpoint_data['checkpoint_id'] = str(uuid.uuid4())[:8]

            # Validate with Pydantic
            validated = CheckpointData(**checkpoint_data)
            data_to_save = validated.model_dump(mode='json')

        except Exception as e:
            return False, f"Validation error: {e}"

        # Add checksum
        data_with_checksum = ChecksumUtils.add_checksum(data_to_save)

        # Serialize to JSON
        try:
            json_content = json.dumps(
                data_with_checksum,
                ensure_ascii=False,
                indent=2
            )
        except Exception as e:
            return False, f"JSON serialization error: {e}"

        # Determine file paths
        session_dir = self.storage_path / session_id / "checkpoints"
        session_dir.mkdir(parents=True, exist_ok=True)

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        checkpoint_id = data_with_checksum.get('checkpoint_id', uuid.uuid4().hex[:8])
        filename = f"cp_{timestamp}_{checkpoint_id}.json"
        filepath = session_dir / filename

        # Backup existing checkpoint if present
        latest_path = session_dir / "latest.json"
        self._backup_checkpoint(latest_path)

        # Write checkpoint atomically
        success, error = self.atomic_writer.write_atomic(filepath, json_content)
        if not success:
            return False, f"Atomic write failed: {error}"

        # Update latest symlink/copy
        success, error = self._update_latest(session_dir, filepath)
        if not success:
            # Non-fatal: checkpoint saved, but latest update failed
            pass

        # Prune old backups
        self._prune_backups(session_dir)

        return True, None

    def load_checkpoint(
        self,
        session_id: str,
        checkpoint_path: Optional[Path] = None
    ) -> Tuple[Optional[dict], Optional[str]]:
        """Load checkpoint with validation and recovery.

        Args:
            session_id: Session identifier
            checkpoint_path: Specific checkpoint path (optional, uses latest if not provided)

        Returns:
            Tuple of (checkpoint_data, error_message)
        """
        session_dir = self.storage_path / session_id / "checkpoints"

        # Determine which checkpoint to load
        if checkpoint_path:
            filepath = Path(checkpoint_path)
        else:
            # Try latest first
            latest_path = session_dir / "latest.json"
            if latest_path.exists():
                filepath = latest_path
            else:
                # Find most recent checkpoint
                checkpoints = sorted(
                    session_dir.glob("cp_*.json"),
                    key=lambda p: p.stat().st_mtime,
                    reverse=True
                )
                if not checkpoints:
                    return None, "No checkpoints found"
                filepath = checkpoints[0]

        if not filepath.exists():
            return None, f"Checkpoint file not found: {filepath}"

        # Read file
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
                data = json.loads(content)
        except json.JSONDecodeError as e:
            # Try recovery from backup
            return self._recover_from_backup(filepath)
        except Exception as e:
            return None, f"Read error: {e}"

        # Verify checksum
        is_valid, error = ChecksumUtils.verify_checksum(data)
        if not is_valid:
            # Try recovery from backup
            return self._recover_from_backup(filepath)

        return data, None

    def _backup_checkpoint(self, filepath: Path):
        """Create backup of existing checkpoint."""
        if not filepath.exists():
            return

        try:
            backup_path = filepath.with_suffix(f"{filepath.suffix}{BACKUP_FILE_SUFFIX}")
            shutil.copy2(filepath, backup_path)
        except Exception:
            pass

    def _recover_from_backup(
        self,
        filepath: Path
    ) -> Tuple[Optional[dict], Optional[str]]:
        """Attempt to recover checkpoint from backup files.

        Tries all available backups in order of recency.
        """
        session_dir = filepath.parent

        # Find all backup files
        backups = sorted(
            session_dir.glob(f"*{BACKUP_FILE_SUFFIX}"),
            key=lambda p: p.stat().st_mtime,
            reverse=True
        )

        for backup_path in backups:
            try:
                with open(backup_path, 'r', encoding='utf-8') as f:
                    data = json.load(f)

                # Verify checksum
                is_valid, error = ChecksumUtils.verify_checksum(data)
                if is_valid:
                    return data, None

            except Exception:
                continue

        return None, "No valid backup found for recovery"

    def _update_latest(
        self,
        session_dir: Path,
        filepath: Path
    ) -> Tuple[bool, Optional[str]]:
        """Update latest checkpoint reference.

        On Unix: uses symlink (atomic)
        On Windows: uses copy (non-atomic but acceptable)
        """
        latest_path = session_dir / "latest.json"

        try:
            if os.name == 'nt':
                # Windows: use copy (symlink requires admin)
                shutil.copy2(filepath, latest_path)
            else:
                # Unix: use atomic symlink
                temp_link = session_dir / f".tmp_latest_{uuid.uuid4().hex}"
                temp_link.symlink_to(filepath.name)
                temp_link.rename(latest_path)

            return True, None

        except Exception as e:
            return False, str(e)

    def _prune_backups(self, session_dir: Path):
        """Prune old backup files to limit storage."""
        backups = sorted(
            session_dir.glob(f"*{BACKUP_FILE_SUFFIX}"),
            key=lambda p: p.stat().st_mtime,
            reverse=True
        )

        # Keep only max_backups most recent
        for old_backup in backups[self.max_backups:]:
            try:
                old_backup.unlink()
            except Exception:
                pass

    def verify_checkpoint_integrity(
        self,
        filepath: Path
    ) -> Tuple[bool, Optional[str]]:
        """Verify checkpoint file integrity without full loading.

        Args:
            filepath: Path to checkpoint file

        Returns:
            Tuple of (is_valid, error_message)
        """
        if not filepath.exists():
            return False, "File does not exist"

        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
                data = json.loads(content)

            # Verify checksum
            return ChecksumUtils.verify_checksum(data)

        except json.JSONDecodeError as e:
            return False, f"Invalid JSON: {e}"
        except Exception as e:
            return False, f"Read error: {e}"

    def list_checkpoints(
        self,
        session_id: str
    ) -> list[Tuple[Path, datetime, bool]]:
        """List all checkpoints for a session with integrity status.

        Args:
            session_id: Session identifier

        Returns:
            List of tuples: (path, modified_time, is_valid)
        """
        session_dir = self.storage_path / session_id / "checkpoints"
        if not session_dir.exists():
            return []

        checkpoints = []
        for cp_file in sorted(session_dir.glob("cp_*.json")):
            try:
                mtime = datetime.fromtimestamp(cp_file.stat().st_mtime)
                is_valid, _ = self.verify_checkpoint_integrity(cp_file)
                checkpoints.append((cp_file, mtime, is_valid))
            except Exception:
                continue

        return sorted(checkpoints, key=lambda x: x[1], reverse=True)


# ============================================================================
# Crash Recovery Utilities
# ============================================================================

class CrashRecovery:
    """Crash recovery utilities for checkpoint restoration."""

    def __init__(self, writer: CheckpointWriter):
        """Initialize crash recovery.

        Args:
            writer: CheckpointWriter instance
        """
        self.writer = writer

    def detect_unclean_shutdown(self) -> Optional[dict]:
        """Detect if previous session ended uncleanly.

        Returns:
            Crash info dict if unclean shutdown detected, None otherwise
        """
        # Check for sessions in active state
        for session_dir in self.writer.storage_path.iterdir():
            if not session_dir.is_dir():
                continue

            session_meta = session_dir / "session_meta.json"
            if not session_meta.exists():
                continue

            try:
                with open(session_meta, 'r', encoding='utf-8') as f:
                    meta = json.load(f)

                # Check for active session without proper termination
                if meta.get("is_active", False) and meta.get("termination_reason") is None:
                    return {
                        "session_id": meta.get("session_id"),
                        "last_activity": meta.get("last_updated"),
                        "last_iteration": meta.get("last_iteration", 0)
                    }

            except Exception:
                continue

        return None

    def recover_session(
        self,
        session_id: str
    ) -> Tuple[Optional[dict], Optional[str]]:
        """Recover session from latest valid checkpoint.

        Args:
            session_id: Session to recover

        Returns:
            Tuple of (checkpoint_data, error_message)
        """
        checkpoints = self.writer.list_checkpoints(session_id)

        if not checkpoints:
            return None, "No checkpoints available for recovery"

        # Try checkpoints in order of recency
        for cp_path, mtime, is_valid in checkpoints:
            if not is_valid:
                continue

            data, error = self.writer.load_checkpoint(session_id, cp_path)
            if data:
                return data, None

        # Try backup recovery as last resort
        session_dir = self.writer.storage_path / session_id / "checkpoints"
        data, error = self.writer._recover_from_backup(session_dir / "latest.json")
        if data:
            return data, "Recovered from backup"

        return None, "All recovery attempts failed"


# ============================================================================
# Async Wrapper for CheckpointWriter
# ============================================================================

class AsyncCheckpointWriter:
    """Async wrapper for CheckpointWriter.

    Provides async interface for use in async contexts.
    """

    def __init__(self, *args, **kwargs):
        """Initialize async checkpoint writer."""
        self._sync_writer = CheckpointWriter(*args, **kwargs)
        self._executor = None

    async def save_checkpoint(
        self,
        checkpoint_data: dict,
        session_id: str,
        trigger: str = "manual"
    ) -> Tuple[bool, Optional[str]]:
        """Save checkpoint asynchronously."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            None,
            lambda: self._sync_writer.save_checkpoint(
                checkpoint_data,
                session_id,
                trigger
            )
        )

    async def load_checkpoint(
        self,
        session_id: str,
        checkpoint_path: Optional[Path] = None
    ) -> Tuple[Optional[dict], Optional[str]]:
        """Load checkpoint asynchronously."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            None,
            lambda: self._sync_writer.load_checkpoint(session_id, checkpoint_path)
        )

    async def verify_checkpoint_integrity(
        self,
        filepath: Path
    ) -> Tuple[bool, Optional[str]]:
        """Verify checkpoint integrity asynchronously."""
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            None,
            lambda: self._sync_writer.verify_checkpoint_integrity(filepath)
        )

    @property
    def sync_writer(self) -> CheckpointWriter:
        """Access underlying sync writer."""
        return self._sync_writer
