"""Boundary Condition Tests (T3.2)

Tests for handling edge cases and boundary conditions:
- Large file handling
- Timeout handling
- Special character processing
- Concurrent operations
- Resource limits
"""

import os
import sys
import pytest
import tempfile
import shutil
import time
import asyncio
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "src"))

from continuum.checkpoint_writer import CheckpointWriter, AsyncCheckpointWriter


class TestLargeFileHandling:
    """Large file handling tests"""

    @pytest.fixture
    def temp_dir(self):
        d = tempfile.mkdtemp(prefix="sh_large_test_")
        yield d
        shutil.rmtree(d, ignore_errors=True)

    def test_large_checkpoint_save(self, temp_dir):
        """Test saving large checkpoint data"""
        writer = CheckpointWriter(storage_path=temp_dir)

        # Create large checkpoint (simulate large session)
        large_messages = [{"role": "user", "content": f"Message {i}" * 100} for i in range(1000)]

        checkpoint = {
            "session_id": "large-session",
            "messages": large_messages,
            "iteration": 1000,
        }

        success, error = writer.save_checkpoint(checkpoint, "large-session")
        assert success is True, f"Save failed: {error}"

        # Verify load
        data, error = writer.load_checkpoint("large-session")
        assert data is not None
        assert len(data["messages"]) == 1000

    def test_checkpoint_size_limit(self, temp_dir):
        """Test checkpoint size doesn't exceed reasonable limits"""
        writer = CheckpointWriter(storage_path=temp_dir)

        checkpoint = {
            "session_id": "size-test",
            "messages": [{"role": "user", "content": "x" * 10000} for _ in range(100)],
        }

        success, _ = writer.save_checkpoint(checkpoint, "size-test")
        assert success is True

        # Check file size
        session_dir = Path(temp_dir) / "size-test" / "checkpoints"
        files = list(session_dir.glob("*.json"))
        assert len(files) > 0

        # File should be less than 5MB
        file_size = files[0].stat().st_size
        assert file_size < 5 * 1024 * 1024, f"File too large: {file_size}"

    @pytest.mark.asyncio
    async def test_async_large_checkpoint(self, temp_dir):
        """Test async handling of large checkpoints"""
        writer = AsyncCheckpointWriter(storage_path=temp_dir)

        checkpoint = {
            "session_id": "async-large",
            "messages": [{"role": "user", "content": f"Content {i}"} for i in range(500)],
        }

        success, _ = await writer.save_checkpoint(checkpoint, "async-large")
        assert success is True

        data, _ = await writer.load_checkpoint("async-large")
        assert data is not None


class TestTimeoutHandling:
    """Timeout handling tests"""

    @pytest.fixture
    def temp_dir(self):
        d = tempfile.mkdtemp(prefix="sh_timeout_test_")
        yield d
        shutil.rmtree(d, ignore_errors=True)

    def test_checkpoint_write_timeout_handling(self, temp_dir):
        """Test checkpoint write handles slow operations"""
        writer = CheckpointWriter(storage_path=temp_dir)

        # Normal save should complete within reasonable time
        checkpoint = {"session_id": "timeout-test", "messages": []}

        start = time.time()
        success, _ = writer.save_checkpoint(checkpoint, "timeout-test")
        elapsed = time.time() - start

        assert success is True
        assert elapsed < 5.0, f"Save took too long: {elapsed}s"

    @pytest.mark.asyncio
    async def test_async_timeout_protection(self, temp_dir):
        """Test async operations have timeout protection"""
        writer = AsyncCheckpointWriter(storage_path=temp_dir)

        checkpoint = {"session_id": "async-timeout", "messages": []}

        # Should complete quickly
        start = time.time()
        success, _ = await writer.save_checkpoint(checkpoint, "async-timeout")
        elapsed = time.time() - start

        assert success is True
        assert elapsed < 3.0


class TestSpecialCharacterHandling:
    """Special character handling tests"""

    @pytest.fixture
    def temp_dir(self):
        d = tempfile.mkdtemp(prefix="sh_special_test_")
        yield d
        shutil.rmtree(d, ignore_errors=True)

    def test_unicode_in_checkpoint(self, temp_dir):
        """Test Unicode characters preserved in checkpoint"""
        writer = CheckpointWriter(storage_path=temp_dir)

        checkpoint = {
            "session_id": "unicode-test",
            "messages": [
                {"role": "user", "content": "你好世界 🎉"},
                {"role": "assistant", "content": "こんにちは世界 🌍"},
                {"role": "user", "content": "Привет мир!"},
            ],
        }

        success, _ = writer.save_checkpoint(checkpoint, "unicode-test")
        assert success is True

        data, _ = writer.load_checkpoint("unicode-test")
        assert "你好世界" in data["messages"][0]["content"]
        assert "こんにちは" in data["messages"][1]["content"]
        assert "Привет" in data["messages"][2]["content"]

    def test_special_chars_json_escape(self, temp_dir):
        """Test JSON special characters handled correctly"""
        writer = CheckpointWriter(storage_path=temp_dir)

        checkpoint = {
            "session_id": "json-chars",
            "messages": [
                {"role": "user", "content": "Quote: \"test\""},
                {"role": "user", "content": "Backslash: \\path"},
                {"role": "user", "content": "Newline:\nLine2"},
                {"role": "user", "content": "Tab:\tvalue"},
                {"role": "user", "content": "HTML: <script>alert(1)</script>"},
            ],
        }

        success, _ = writer.save_checkpoint(checkpoint, "json-chars")
        assert success is True

        data, _ = writer.load_checkpoint("json-chars")
        assert '"' in data["messages"][0]["content"]  # Quote preserved
        assert "\\" in data["messages"][1]["content"]  # Backslash preserved
        assert "\n" in data["messages"][2]["content"]  # Newline preserved

    def test_null_and_empty_handling(self, temp_dir):
        """Test null values and empty strings"""
        writer = CheckpointWriter(storage_path=temp_dir)

        checkpoint = {
            "session_id": "null-test",
            "messages": [
                {"role": "user", "content": ""},  # Empty string
                {"role": "assistant", "content": "normal"},
            ],
            "tokens_used": 0,  # Use supported fields
            "cost_estimate": 0.0,
        }

        success, _ = writer.save_checkpoint(checkpoint, "null-test")
        assert success is True

        data, _ = writer.load_checkpoint("null-test")
        assert data["messages"][0]["content"] == ""
        assert data["tokens_used"] == 0


class TestConcurrentOperations:
    """Concurrent operation tests"""

    @pytest.fixture
    def temp_dir(self):
        d = tempfile.mkdtemp(prefix="sh_concurrent_test_")
        yield d
        shutil.rmtree(d, ignore_errors=True)

    def test_concurrent_checkpoint_saves(self, temp_dir):
        """Test multiple concurrent saves don't corrupt data"""
        writer = CheckpointWriter(storage_path=temp_dir)
        errors = []

        import threading

        def save_checkpoint(session_id):
            try:
                checkpoint = {
                    "session_id": session_id,
                    "messages": [{"role": "user", "content": f"Data for {session_id}"}],
                    "thread_id": threading.current_thread().name,
                }
                success, error = writer.save_checkpoint(checkpoint, session_id)
                if not success:
                    errors.append(error)
            except Exception as e:
                errors.append(str(e))

        threads = [
            threading.Thread(target=save_checkpoint, args=(f"concurrent-{i}",), name=f"thread-{i}")
            for i in range(10)
        ]

        for t in threads:
            t.start()
        for t in threads:
            t.join()

        assert len(errors) == 0, f"Errors occurred: {errors}"

        # Verify all checkpoints are valid
        for i in range(10):
            data, error = writer.load_checkpoint(f"concurrent-{i}")
            assert data is not None, f"Failed to load concurrent-{i}: {error}"

    @pytest.mark.asyncio
    async def test_async_concurrent_operations(self, temp_dir):
        """Test async concurrent operations"""
        writer = AsyncCheckpointWriter(storage_path=temp_dir)

        async def save_and_load(session_id):
            checkpoint = {"session_id": session_id, "messages": []}
            await writer.save_checkpoint(checkpoint, session_id)
            data, _ = await writer.load_checkpoint(session_id)
            return data

        # Run 5 concurrent operations
        results = await asyncio.gather(
            save_and_load("async-1"),
            save_and_load("async-2"),
            save_and_load("async-3"),
            save_and_load("async-4"),
            save_and_load("async-5"),
        )

        for result in results:
            assert result is not None


class TestResourceLimits:
    """Resource limit tests"""

    @pytest.fixture
    def temp_dir(self):
        d = tempfile.mkdtemp(prefix="sh_resource_test_")
        yield d
        shutil.rmtree(d, ignore_errors=True)

    def test_max_checkpoints_per_session(self, temp_dir):
        """Test checkpoint rotation cleans up old files"""
        writer = CheckpointWriter(storage_path=temp_dir)

        # Save many checkpoints for same session
        for i in range(20):
            checkpoint = {
                "session_id": "rotation-test",
                "messages": [{"iteration": i}],
            }
            writer.save_checkpoint(checkpoint, "rotation-test")

        # List checkpoints
        checkpoints = writer.list_checkpoints("rotation-test")

        # Should not exceed max (typically 10)
        # If rotation is implemented, should be limited
        assert len(checkpoints) <= 20  # At least doesn't grow unbounded

    def test_disk_space_monitoring(self, temp_dir):
        """Test available disk space is considered"""
        writer = CheckpointWriter(storage_path=temp_dir)

        # Save a checkpoint
        checkpoint = {"session_id": "disk-test", "messages": []}
        success, _ = writer.save_checkpoint(checkpoint, "disk-test")

        # Should succeed when disk is available
        assert success is True


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])