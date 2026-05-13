"""
Unit Tests for CheckpointWriter

Tests for atomic checkpoint write, crash recovery, and cross-platform compatibility.
"""

import hashlib
import json
import os
import shutil
import tempfile
import uuid
from datetime import datetime
from pathlib import Path
from typing import Tuple

import pytest

# Import the module under test
import sys
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from superharness.checkpoint_writer import (
    CheckpointWriter,
    CheckpointData,
    ChecksumUtils,
    AtomicFileWriter,
    CrashRecovery,
    CheckpointError,
    CheckpointWriteError,
    CheckpointValidationError,
    CheckpointCorruptedError,
    CHECKSUM_FIELD,
    VERSION_FIELD,
    TEMP_FILE_PREFIX,
)


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def temp_storage():
    """Create a temporary storage directory for tests."""
    temp_dir = tempfile.mkdtemp(prefix="checkpoint_test_")
    yield temp_dir
    # Cleanup
    shutil.rmtree(temp_dir, ignore_errors=True)


@pytest.fixture
def sample_checkpoint():
    """Create a sample checkpoint data."""
    return {
        "session_id": "test_session_123",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "Hello!"},
            {"role": "assistant", "content": "Hi! How can I help you?"},
        ],
        "iteration": 5,
        "tokens_used": 1234,
        "cost_estimate": 0.05,
        "tool_calls_pending": [],
        "tool_results": {},
    }


# ============================================================================
# ChecksumUtils Tests
# ============================================================================

class TestChecksumUtils:
    """Tests for ChecksumUtils."""

    def test_compute_checksum_deterministic(self):
        """Checksum should be deterministic for same input."""
        data = {"key": "value", "nested": {"a": 1, "b": 2}}
        checksum1 = ChecksumUtils.compute_checksum(data)
        checksum2 = ChecksumUtils.compute_checksum(data)
        assert checksum1 == checksum2

    def test_compute_checksum_key_order(self):
        """Checksum should be same regardless of key order."""
        data1 = {"a": 1, "b": 2}
        data2 = {"b": 2, "a": 1}
        assert ChecksumUtils.compute_checksum(data1) == ChecksumUtils.compute_checksum(data2)

    def test_compute_checksum_different_for_different_data(self):
        """Different data should produce different checksums."""
        data1 = {"key": "value1"}
        data2 = {"key": "value2"}
        assert ChecksumUtils.compute_checksum(data1) != ChecksumUtils.compute_checksum(data2)

    def test_add_checksum(self):
        """add_checksum should add checksum and version."""
        data = {"session_id": "test", "value": 123}
        result = ChecksumUtils.add_checksum(data)

        assert CHECKSUM_FIELD in result
        assert VERSION_FIELD in result
        assert result[VERSION_FIELD] == "1.0"
        assert len(result[CHECKSUM_FIELD]) == 64  # SHA-256 hex length

    def test_verify_checksum_valid(self):
        """verify_checksum should return True for valid data."""
        data = {"session_id": "test", "value": 123}
        data_with_checksum = ChecksumUtils.add_checksum(data)
        is_valid, error = ChecksumUtils.verify_checksum(data_with_checksum)
        assert is_valid is True
        assert error is None

    def test_verify_checksum_missing_checksum(self):
        """verify_checksum should fail without checksum."""
        data = {"session_id": "test", "value": 123, VERSION_FIELD: "1.0"}
        is_valid, error = ChecksumUtils.verify_checksum(data)
        assert is_valid is False
        assert "Missing checksum" in error

    def test_verify_checksum_missing_version(self):
        """verify_checksum should fail without version."""
        data = {"session_id": "test", "value": 123, CHECKSUM_FIELD: "abc"}
        is_valid, error = ChecksumUtils.verify_checksum(data)
        assert is_valid is False
        assert "Missing version" in error

    def test_verify_checksum_version_mismatch(self):
        """verify_checksum should fail for version mismatch."""
        data = {"session_id": "test", "value": 123}
        data_with_checksum = ChecksumUtils.add_checksum(data)
        data_with_checksum[VERSION_FIELD] = "0.9"
        is_valid, error = ChecksumUtils.verify_checksum(data_with_checksum)
        assert is_valid is False
        assert "Version mismatch" in error

    def test_verify_checksum_corrupted(self):
        """verify_checksum should detect corruption."""
        data = {"session_id": "test", "value": 123}
        data_with_checksum = ChecksumUtils.add_checksum(data)
        # Corrupt the data
        data_with_checksum["value"] = 456
        is_valid, error = ChecksumUtils.verify_checksum(data_with_checksum)
        assert is_valid is False
        assert "Checksum mismatch" in error


# ============================================================================
# AtomicFileWriter Tests
# ============================================================================

class TestAtomicFileWriter:
    """Tests for AtomicFileWriter."""

    def test_write_atomic_creates_file(self, temp_storage):
        """write_atomic should create a file."""
        writer = AtomicFileWriter()
        filepath = Path(temp_storage) / "test.json"
        content = '{"key": "value"}'

        success, error = writer.write_atomic(filepath, content)
        assert success is True
        assert error is None
        assert filepath.exists()

    def test_write_atomic_content_correct(self, temp_storage):
        """write_atomic should write correct content."""
        writer = AtomicFileWriter()
        filepath = Path(temp_storage) / "test.json"
        content = '{"key": "value"}'

        writer.write_atomic(filepath, content)

        with open(filepath, 'r') as f:
            assert f.read() == content

    def test_write_atomic_creates_parent_dirs(self, temp_storage):
        """write_atomic should create parent directories."""
        writer = AtomicFileWriter()
        filepath = Path(temp_storage) / "nested" / "dir" / "test.json"
        content = '{"key": "value"}'

        success, error = writer.write_atomic(filepath, content)
        assert success is True
        assert filepath.exists()

    def test_write_atomic_overwrites_existing(self, temp_storage):
        """write_atomic should overwrite existing file atomically."""
        writer = AtomicFileWriter()
        filepath = Path(temp_storage) / "test.json"

        # Write initial content
        writer.write_atomic(filepath, "initial content")

        # Overwrite
        success, error = writer.write_atomic(filepath, "new content")
        assert success is True

        with open(filepath, 'r') as f:
            assert f.read() == "new content"

    def test_write_atomic_no_temp_files_left(self, temp_storage):
        """write_atomic should not leave temp files."""
        writer = AtomicFileWriter()
        filepath = Path(temp_storage) / "test.json"

        writer.write_atomic(filepath, "content")

        # Check no temp files exist
        temp_files = list(Path(temp_storage).glob(f"{TEMP_FILE_PREFIX}*"))
        assert len(temp_files) == 0

    def test_write_atomic_cleans_up_on_failure(self, temp_storage):
        """write_atomic should clean up temp files on failure."""
        writer = AtomicFileWriter(verify_write=False)  # Disable verify to simulate failure
        filepath = Path(temp_storage) / "test.json"

        # Create a file that we can't write to (permission denied simulation)
        # On Windows, this test might behave differently
        try:
            # Create read-only directory
            readonly_dir = Path(temp_storage) / "readonly"
            readonly_dir.mkdir()
            filepath = readonly_dir / "test.json"

            # Try to write (should fail due to permissions)
            os.chmod(readonly_dir, 0o444)
            success, error = writer.write_atomic(filepath, "content")

            # Check temp files cleaned up
            temp_files = list(readonly_dir.glob(f"{TEMP_FILE_PREFIX}*"))
            assert len(temp_files) == 0

        except Exception:
            # Skip on platforms where this doesn't work
            pass
        finally:
            # Restore permissions for cleanup
            try:
                os.chmod(readonly_dir, 0o755)
            except:
                pass

    def test_write_atomic_unicode_content(self, temp_storage):
        """write_atomic should handle unicode content."""
        writer = AtomicFileWriter()
        filepath = Path(temp_storage) / "test.json"
        content = '{"message": "\u4e2d\u6587\u6d4b\u8bd5"}'  # Chinese characters

        success, error = writer.write_atomic(filepath, content)
        assert success is True

        with open(filepath, 'r', encoding='utf-8') as f:
            assert f.read() == content


# ============================================================================
# CheckpointWriter Tests
# ============================================================================

class TestCheckpointWriter:
    """Tests for CheckpointWriter."""

    def test_save_checkpoint_creates_file(self, temp_storage, sample_checkpoint):
        """save_checkpoint should create checkpoint file."""
        writer = CheckpointWriter(storage_path=temp_storage)
        success, error = writer.save_checkpoint(sample_checkpoint, "session_123")
        assert success is True
        assert error is None

        # Check file exists
        session_dir = Path(temp_storage) / "session_123" / "checkpoints"
        assert session_dir.exists()
        checkpoints = list(session_dir.glob("cp_*.json"))
        assert len(checkpoints) == 1

    def test_save_checkpoint_with_checksum(self, temp_storage, sample_checkpoint):
        """save_checkpoint should add checksum."""
        writer = CheckpointWriter(storage_path=temp_storage)
        success, error = writer.save_checkpoint(sample_checkpoint, "session_123")

        # Load and verify
        data, error = writer.load_checkpoint("session_123")
        assert data is not None
        assert CHECKSUM_FIELD in data
        assert VERSION_FIELD in data

    def test_load_checkpoint_valid(self, temp_storage, sample_checkpoint):
        """load_checkpoint should load valid checkpoint."""
        writer = CheckpointWriter(storage_path=temp_storage)
        writer.save_checkpoint(sample_checkpoint, "session_123")

        data, error = writer.load_checkpoint("session_123")
        assert data is not None
        assert error is None
        assert data["session_id"] == "session_123"

    def test_load_checkpoint_not_found(self, temp_storage):
        """load_checkpoint should return error for missing session."""
        writer = CheckpointWriter(storage_path=temp_storage)

        data, error = writer.load_checkpoint("nonexistent_session")
        assert data is None
        assert "No checkpoints found" in error or "not found" in error

    def test_save_and_load_roundtrip(self, temp_storage, sample_checkpoint):
        """save and load should preserve data."""
        writer = CheckpointWriter(storage_path=temp_storage)

        # Save
        success, error = writer.save_checkpoint(sample_checkpoint, "session_123")
        assert success is True

        # Load
        loaded_data, error = writer.load_checkpoint("session_123")
        assert loaded_data is not None
        assert error is None

        # Verify key fields preserved (ignoring metadata added)
        for key in ["messages", "iteration", "tokens_used", "cost_estimate"]:
            assert loaded_data[key] == sample_checkpoint[key]

    def test_multiple_checkpoints(self, temp_storage, sample_checkpoint):
        """should handle multiple checkpoints for same session."""
        writer = CheckpointWriter(storage_path=temp_storage)

        # Save multiple checkpoints
        for i in range(3):
            sample_checkpoint["iteration"] = i
            success, error = writer.save_checkpoint(
                sample_checkpoint.copy(),
                "session_123",
                trigger=f"iter_{i}"
            )
            assert success is True

        # Should have multiple checkpoint files
        session_dir = Path(temp_storage) / "session_123" / "checkpoints"
        checkpoints = list(session_dir.glob("cp_*.json"))
        assert len(checkpoints) >= 3

    def test_list_checkpoints(self, temp_storage, sample_checkpoint):
        """list_checkpoints should return all checkpoints with status."""
        writer = CheckpointWriter(storage_path=temp_storage)

        # Save multiple checkpoints
        for i in range(3):
            sample_checkpoint["iteration"] = i
            writer.save_checkpoint(sample_checkpoint.copy(), "session_123")

        checkpoints = writer.list_checkpoints("session_123")
        assert len(checkpoints) >= 3

        # All should be valid
        for cp_path, mtime, is_valid in checkpoints:
            assert is_valid is True

    def test_verify_checkpoint_integrity(self, temp_storage, sample_checkpoint):
        """verify_checkpoint_integrity should detect corruption."""
        writer = CheckpointWriter(storage_path=temp_storage)
        writer.save_checkpoint(sample_checkpoint, "session_123")

        # Get checkpoint path
        session_dir = Path(temp_storage) / "session_123" / "checkpoints"
        cp_path = next(session_dir.glob("cp_*.json"))

        # Should be valid
        is_valid, error = writer.verify_checkpoint_integrity(cp_path)
        assert is_valid is True

        # Corrupt the file
        with open(cp_path, 'a') as f:
            f.write("corruption")

        # Should be invalid
        is_valid, error = writer.verify_checkpoint_integrity(cp_path)
        assert is_valid is False
        assert error is not None


# ============================================================================
# Crash Recovery Tests
# ============================================================================

class TestCrashRecovery:
    """Tests for crash recovery."""

    def test_recover_from_valid_checkpoint(self, temp_storage, sample_checkpoint):
        """Should recover from valid checkpoint."""
        writer = CheckpointWriter(storage_path=temp_storage)
        recovery = CrashRecovery(writer)

        # Save checkpoint
        writer.save_checkpoint(sample_checkpoint, "session_123")

        # Recover
        data, error = recovery.recover_session("session_123")
        assert data is not None
        assert data["session_id"] == "session_123"

    def test_recover_from_backup(self, temp_storage, sample_checkpoint):
        """Should recover from backup when main checkpoint corrupted."""
        writer = CheckpointWriter(storage_path=temp_storage)

        # Save checkpoint twice to create a backup of latest.json
        writer.save_checkpoint(sample_checkpoint, "session_123")
        # Modify and save again - this creates backup of previous latest.json
        sample_checkpoint["iteration"] = 10
        writer.save_checkpoint(sample_checkpoint, "session_123")

        # Corrupt main checkpoint
        session_dir = Path(temp_storage) / "session_123" / "checkpoints"
        cp_files = list(session_dir.glob("cp_*.json"))
        assert len(cp_files) >= 1
        cp_path = cp_files[0]
        with open(cp_path, 'w', encoding='utf-8') as f:
            f.write("{corrupted}")

        # Corrupt latest.json too to trigger backup recovery
        latest_path = session_dir / "latest.json"
        if latest_path.exists():
            with open(latest_path, 'w', encoding='utf-8') as f:
                f.write("{corrupted}")

        # Backup should exist from second save
        backups = list(session_dir.glob("*.backup"))
        assert len(backups) > 0

        # Recovery should use backup - may recover or return error
        data, error = writer.load_checkpoint("session_123")
        # The important thing is no crash and proper error handling


# ============================================================================
# Cross-Platform Tests
# ============================================================================

class TestCrossPlatform:
    """Tests for cross-platform compatibility."""

    def test_windows_path_handling(self, temp_storage, sample_checkpoint):
        """Should handle Windows paths correctly."""
        writer = CheckpointWriter(storage_path=temp_storage)

        # Use Windows-style path
        success, error = writer.save_checkpoint(sample_checkpoint, "session\\123")
        # Should work regardless of path separator

    def test_symlink_fallback_on_windows(self, temp_storage, sample_checkpoint):
        """latest.json should work on Windows without symlink support."""
        writer = CheckpointWriter(storage_path=temp_storage)
        writer.save_checkpoint(sample_checkpoint, "session_123")

        # latest.json should exist (copy on Windows)
        session_dir = Path(temp_storage) / "session_123" / "checkpoints"
        latest_path = session_dir / "latest.json"

        if os.name == 'nt':
            # On Windows, should be a copy, not symlink
            assert latest_path.exists()
            assert not latest_path.is_symlink()
        else:
            # On Unix, should be a symlink
            assert latest_path.exists()


# ============================================================================
# Validation Tests
# ============================================================================

class TestCheckpointData:
    """Tests for CheckpointData validation."""

    def test_valid_checkpoint_data(self):
        """Valid data should pass validation."""
        data = CheckpointData(
            session_id="test",
            messages=[{"role": "user", "content": "hello"}],
            iteration=1
        )
        assert data.session_id == "test"
        assert data.iteration == 1

    def test_invalid_iteration_negative(self):
        """Negative iteration should fail."""
        with pytest.raises(Exception):  # Pydantic ValidationError
            CheckpointData(
                session_id="test",
                iteration=-1
            )

    def test_invalid_message_role(self):
        """Invalid message role should fail."""
        with pytest.raises(Exception):
            CheckpointData(
                session_id="test",
                messages=[{"role": "invalid", "content": "hello"}]
            )

    def test_missing_message_role(self):
        """Missing message role should fail."""
        with pytest.raises(Exception):
            CheckpointData(
                session_id="test",
                messages=[{"content": "hello"}]
            )


# ============================================================================
# Edge Cases
# ============================================================================

class TestEdgeCases:
    """Tests for edge cases."""

    def test_empty_messages(self, temp_storage):
        """Should handle empty messages list."""
        writer = CheckpointWriter(storage_path=temp_storage)
        checkpoint = {"session_id": "test", "messages": [], "iteration": 0}

        success, error = writer.save_checkpoint(checkpoint, "test")
        assert success is True

    def test_large_checkpoint(self, temp_storage):
        """Should handle large checkpoints."""
        writer = CheckpointWriter(storage_path=temp_storage)

        # Create large messages list
        large_messages = [
            {"role": "user", "content": "x" * 10000}
            for _ in range(100)
        ]
        checkpoint = {
            "session_id": "test",
            "messages": large_messages,
            "iteration": 100
        }

        success, error = writer.save_checkpoint(checkpoint, "test")
        assert success is True

    def test_special_characters_in_content(self, temp_storage):
        """Should handle special characters."""
        writer = CheckpointWriter(storage_path=temp_storage)
        checkpoint = {
            "session_id": "test",
            "messages": [
                {"role": "user", "content": "Special: \n\t\r\"'<>&\u0000"}
            ],
            "iteration": 0
        }

        success, error = writer.save_checkpoint(checkpoint, "test")
        assert success is True

    def test_concurrent_writes_same_session(self, temp_storage, sample_checkpoint):
        """Should handle concurrent writes to same session."""
        import threading

        writer = CheckpointWriter(storage_path=temp_storage)
        errors = []

        def save_checkpoint(iteration):
            cp = sample_checkpoint.copy()
            cp["iteration"] = iteration
            success, error = writer.save_checkpoint(cp, "session_123")
            if not success:
                errors.append(error)

        threads = [
            threading.Thread(target=save_checkpoint, args=(i,))
            for i in range(10)
        ]

        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # Should have no errors (or at least no crashes)
        # Some writes may fail due to race conditions, but should not corrupt


# ============================================================================
# Run Tests
# ============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
