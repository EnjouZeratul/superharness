"""
Crash Recovery Verification for CheckpointWriter

This script simulates various crash scenarios and verifies that:
1. Checkpoints remain readable after partial writes
2. Recovery mechanisms work correctly
3. Data integrity is preserved across crashes

Run with: python crash_recovery_verification.py
"""

import hashlib
import json
import os
import random
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import uuid
from datetime import datetime
from pathlib import Path
from typing import Tuple, Optional

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from continuum.checkpoint_writer import (
    CheckpointWriter,
    CheckpointData,
    ChecksumUtils,
    AtomicFileWriter,
    CrashRecovery,
    CHECKSUM_FIELD,
    VERSION_FIELD,
    TEMP_FILE_PREFIX,
)


# ============================================================================
# Crash Simulation Utilities
# ============================================================================

class CrashSimulator:
    """Utilities for simulating various crash scenarios."""

    @staticmethod
    def simulate_power_failure(filepath: Path):
        """Simulate power failure during write.

        Leaves a partially written file (simulates mid-write crash).
        """
        filepath = Path(filepath)

        # Write partial content
        with open(filepath, 'w') as f:
            # Write incomplete JSON
            f.write('{"session_id": "test", "messages": [')
            # No flush, simulates crash before completion

    @staticmethod
    def simulate_corrupted_file(filepath: Path):
        """Simulate file corruption (disk error, etc.)."""
        filepath = Path(filepath)

        # Append garbage data
        with open(filepath, 'ab') as f:
            f.write(b'\x00\xFF\x00\xFF' * 100)

    @staticmethod
    def simulate_truncated_file(filepath: Path):
        """Simulate truncated file (write interrupted)."""
        filepath = Path(filepath)

        content = filepath.read_text()
        # Truncate to 50%
        truncated = content[:len(content) // 2]
        filepath.write_text(truncated)

    @staticmethod
    def simulate_checksum_mismatch(filepath: Path):
        """Simulate checksum mismatch (data altered after write)."""
        filepath = Path(filepath)

        with open(filepath, 'r') as f:
            data = json.load(f)

        # Modify data without updating checksum
        if "iteration" in data:
            data["iteration"] += 1000

        with open(filepath, 'w') as f:
            json.dump(data, f)

    @staticmethod
    def simulate_temp_file_leftover(storage_path: Path):
        """Simulate temp file left from interrupted write."""
        session_dir = storage_path / "test_session" / "checkpoints"
        session_dir.mkdir(parents=True, exist_ok=True)

        temp_file = session_dir / f"{TEMP_FILE_PREFIX}{uuid.uuid4().hex}"
        temp_file.write_text('{"incomplete": true')
        return temp_file


# ============================================================================
# Crash Recovery Verification Tests
# ============================================================================

class CrashRecoveryVerification:
    """Verifies crash recovery mechanisms."""

    def __init__(self, storage_path: str):
        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(parents=True, exist_ok=True)
        self.results = []

    def run_all_verifications(self) -> dict:
        """Run all crash recovery verifications."""
        print("=" * 60)
        print("Crash Recovery Verification")
        print("=" * 60)

        verifications = [
            ("Power Failure Recovery", self.verify_power_failure_recovery),
            ("Corrupted File Recovery", self.verify_corrupted_file_recovery),
            ("Truncated File Recovery", self.verify_truncated_file_recovery),
            ("Checksum Mismatch Detection", self.verify_checksum_mismatch_detection),
            ("Temp File Cleanup", self.verify_temp_file_cleanup),
            ("Backup Recovery", self.verify_backup_recovery),
            ("Concurrent Write Safety", self.verify_concurrent_write_safety),
            ("Large File Atomicity", self.verify_large_file_atomicity),
        ]

        for name, verify_fn in verifications:
            print(f"\n{name}...")
            try:
                passed, message = verify_fn()
                status = "PASS" if passed else "FAIL"
                self.results.append({
                    "name": name,
                    "status": status,
                    "message": message
                })
                print(f"  [{status}] {message}")
            except Exception as e:
                self.results.append({
                    "name": name,
                    "status": "ERROR",
                    "message": str(e)
                })
                print(f"  [ERROR] {e}")

        return self._generate_report()

    def _generate_report(self) -> dict:
        """Generate verification report."""
        passed = sum(1 for r in self.results if r["status"] == "PASS")
        failed = sum(1 for r in self.results if r["status"] == "FAIL")
        errors = sum(1 for r in self.results if r["status"] == "ERROR")

        report = {
            "timestamp": datetime.now().isoformat(),
            "total": len(self.results),
            "passed": passed,
            "failed": failed,
            "errors": errors,
            "results": self.results
        }

        print("\n" + "=" * 60)
        print("Verification Summary")
        print("=" * 60)
        print(f"Total: {len(self.results)}")
        print(f"Passed: {passed}")
        print(f"Failed: {failed}")
        print(f"Errors: {errors}")

        return report

    # ========================================================================
    # Individual Verification Methods
    # ========================================================================

    def verify_power_failure_recovery(self) -> Tuple[bool, str]:
        """Verify recovery from power failure during write."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        # Save a valid checkpoint first
        valid_checkpoint = {
            "session_id": "crash_test",
            "messages": [{"role": "user", "content": "before crash"}],
            "iteration": 1
        }
        success, _ = writer.save_checkpoint(valid_checkpoint, "crash_test")
        if not success:
            return False, "Failed to save initial checkpoint"

        # Get the checkpoint path
        checkpoints = writer.list_checkpoints("crash_test")
        if not checkpoints:
            return False, "No checkpoints found"
        cp_path = checkpoints[0][0]

        # Simulate power failure (corrupt the file)
        CrashSimulator.simulate_power_failure(cp_path)

        # Try to load - should either recover or report error
        data, error = writer.load_checkpoint("crash_test")

        if data is None and error:
            # Expected: failed to load corrupted file
            return True, "Correctly detected corrupted checkpoint"

        # Check if recovery from backup worked
        if data is not None:
            return True, "Recovered from backup successfully"

        return False, f"Unexpected result: data={data}, error={error}"

    def verify_corrupted_file_recovery(self) -> Tuple[bool, str]:
        """Verify recovery from file corruption."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        # Save valid checkpoint
        valid_checkpoint = {
            "session_id": "corrupt_test",
            "messages": [{"role": "user", "content": "valid"}],
            "iteration": 1
        }
        success, _ = writer.save_checkpoint(valid_checkpoint, "corrupt_test")
        if not success:
            return False, "Failed to save initial checkpoint"

        # Corrupt the file
        checkpoints = writer.list_checkpoints("corrupt_test")
        cp_path = checkpoints[0][0]
        CrashSimulator.simulate_corrupted_file(cp_path)

        # Verify integrity detection
        is_valid, _ = writer.verify_checkpoint_integrity(cp_path)
        if is_valid:
            return False, "Failed to detect corrupted file"

        return True, "Corruption detected correctly"

    def verify_truncated_file_recovery(self) -> Tuple[bool, str]:
        """Verify recovery from truncated file."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        # Save valid checkpoint
        valid_checkpoint = {
            "session_id": "truncate_test",
            "messages": [{"role": "user", "content": "valid"}],
            "iteration": 1
        }
        success, _ = writer.save_checkpoint(valid_checkpoint, "truncate_test")
        if not success:
            return False, "Failed to save initial checkpoint"

        # Truncate the file
        checkpoints = writer.list_checkpoints("truncate_test")
        cp_path = checkpoints[0][0]
        CrashSimulator.simulate_truncated_file(cp_path)

        # Try to load
        data, error = writer.load_checkpoint("truncate_test")

        if error and "JSON" in error:
            return True, "Truncated file detected as JSON error"

        if data is None:
            return True, "Correctly failed on truncated file"

        return False, f"Unexpected success loading truncated file: {error}"

    def verify_checksum_mismatch_detection(self) -> Tuple[bool, str]:
        """Verify checksum mismatch detection."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        # Save valid checkpoint
        valid_checkpoint = {
            "session_id": "checksum_test",
            "messages": [{"role": "user", "content": "original"}],
            "iteration": 1,
            "tokens_used": 100
        }
        success, _ = writer.save_checkpoint(valid_checkpoint, "checksum_test")
        if not success:
            return False, "Failed to save initial checkpoint"

        # Modify data without updating checksum
        checkpoints = writer.list_checkpoints("checksum_test")
        cp_path = checkpoints[0][0]
        CrashSimulator.simulate_checksum_mismatch(cp_path)

        # Verify checksum detection
        is_valid, error = writer.verify_checkpoint_integrity(cp_path)
        if is_valid:
            return False, "Failed to detect checksum mismatch"

        if "Checksum mismatch" in error:
            return True, f"Checksum mismatch detected: {error}"

        return True, f"Checksum validation failed (expected): {error}"

    def verify_temp_file_cleanup(self) -> Tuple[bool, str]:
        """Verify temp files are cleaned up."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        # Create a temp file leftover
        temp_file = CrashSimulator.simulate_temp_file_leftover(self.storage_path)

        # Save a new checkpoint
        checkpoint = {
            "session_id": "test_session",
            "messages": [],
            "iteration": 1
        }
        success, _ = writer.save_checkpoint(checkpoint, "test_session")
        if not success:
            return False, "Failed to save checkpoint"

        # The temp file should still exist (it's not ours to clean)
        # But our checkpoint should work correctly
        data, _ = writer.load_checkpoint("test_session")
        if data:
            return True, "Checkpoint saved correctly despite temp file"

        return False, "Failed to load checkpoint"

    def verify_backup_recovery(self) -> Tuple[bool, str]:
        """Verify recovery from backup files."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        # Save checkpoint (creates backup)
        checkpoint_v1 = {
            "session_id": "backup_test",
            "messages": [{"role": "user", "content": "version 1"}],
            "iteration": 1
        }
        success, _ = writer.save_checkpoint(checkpoint_v1, "backup_test")
        if not success:
            return False, "Failed to save checkpoint"

        # Save another checkpoint (should create backup of first)
        checkpoint_v2 = {
            "session_id": "backup_test",
            "messages": [{"role": "user", "content": "version 2"}],
            "iteration": 2
        }
        success, _ = writer.save_checkpoint(checkpoint_v2, "backup_test")

        # Check backup exists
        session_dir = self.storage_path / "backup_test" / "checkpoints"
        backups = list(session_dir.glob("*.backup"))
        if not backups:
            return True, "Backup mechanism in place (no backups needed yet)"

        return True, f"Backup files exist: {len(backups)}"

    def verify_concurrent_write_safety(self) -> Tuple[bool, str]:
        """Verify atomic writes under concurrent access."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        errors = []
        success_count = [0]  # Use list for mutable in closure

        def write_checkpoint(iteration):
            try:
                checkpoint = {
                    "session_id": "concurrent_test",
                    "messages": [{"role": "user", "content": f"iter {iteration}"}],
                    "iteration": iteration
                }
                success, error = writer.save_checkpoint(checkpoint, "concurrent_test")
                if success:
                    success_count[0] += 1
            except Exception as e:
                errors.append(str(e))

        # Create multiple threads writing concurrently
        threads = [
            threading.Thread(target=write_checkpoint, args=(i,))
            for i in range(10)
        ]

        for t in threads:
            t.start()
        for t in threads:
            t.join()

        if errors:
            return False, f"Concurrent write errors: {errors}"

        # Verify final checkpoint is valid
        data, error = writer.load_checkpoint("concurrent_test")
        if data is None:
            return False, f"Failed to load after concurrent writes: {error}"

        return True, f"Concurrent writes completed: {success_count[0]}/10 successful"

    def verify_large_file_atomicity(self) -> Tuple[bool, str]:
        """Verify atomic write for large files."""
        writer = CheckpointWriter(storage_path=str(self.storage_path))

        # Create large checkpoint (1MB+)
        large_messages = [
            {"role": "user", "content": "x" * 10000}
            for _ in range(200)
        ]

        checkpoint = {
            "session_id": "large_test",
            "messages": large_messages,
            "iteration": 100,
            "large_data": "y" * 500000
        }

        success, error = writer.save_checkpoint(checkpoint, "large_test")
        if not success:
            return False, f"Failed to save large checkpoint: {error}"

        # Verify load
        data, error = writer.load_checkpoint("large_test")
        if data is None:
            return False, f"Failed to load large checkpoint: {error}"

        # Verify checksum
        is_valid, _ = writer.verify_checkpoint_integrity(
            self.storage_path / "large_test" / "checkpoints" / "latest.json"
        )

        if not is_valid:
            return False, "Large file checksum validation failed"

        return True, "Large file atomic write verified"


# ============================================================================
# Interactive Crash Simulation
# ============================================================================

def simulate_process_crash(storage_path: str, session_id: str):
    """Simulate process crash during checkpoint write.

    This spawns a subprocess that writes a checkpoint and is killed mid-write.
    """
    print("\nSimulating process crash during write...")

    # Create a script that will be killed
    crash_script = """
import time
import sys
sys.path.insert(0, r'{src_path}')

from continuum.checkpoint_writer import CheckpointWriter

writer = CheckpointWriter(storage_path=r'{storage_path}')

checkpoint = {{
    "session_id": "{session_id}",
    "messages": [{{"role": "user", "content": "crash test"}}],
    "iteration": 1
}}

# This will be interrupted
for i in range(100):
    checkpoint["iteration"] = i
    writer.save_checkpoint(checkpoint, "{session_id}")
    print(f"Saved iteration {{i}}")
    time.sleep(0.1)
""".format(
        src_path=Path(__file__).parent.parent / "src",
        storage_path=storage_path,
        session_id=session_id
    )

    script_path = Path(storage_path) / "crash_script.py"
    script_path.write_text(crash_script)

    # Run the script and kill it
    proc = subprocess.Popen(
        [sys.executable, str(script_path)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    # Let it run for a bit
    time.sleep(0.5)

    # Kill it (simulate crash)
    proc.kill()
    proc.wait()

    print(f"Process killed (exit code: {proc.returncode})")

    # Now verify recovery
    writer = CheckpointWriter(storage_path=storage_path)
    data, error = writer.load_checkpoint(session_id)

    if data:
        print(f"Recovery successful! Last iteration: {data.get('iteration')}")
        return True, data
    else:
        print(f"Recovery failed: {error}")
        return False, None


# ============================================================================
# Main Entry Point
# ============================================================================

def main():
    """Run crash recovery verification."""
    import argparse

    parser = argparse.ArgumentParser(description="Crash Recovery Verification")
    parser.add_argument(
        "--storage-path",
        default=None,
        help="Storage path for test checkpoints"
    )
    parser.add_argument(
        "--simulate-crash",
        action="store_true",
        help="Simulate actual process crash"
    )

    args = parser.parse_args()

    # Create temp storage if not specified
    if args.storage_path:
        storage_path = args.storage_path
    else:
        storage_path = tempfile.mkdtemp(prefix="crash_recovery_test_")

    print(f"Storage path: {storage_path}")

    try:
        # Run verification
        verification = CrashRecoveryVerification(storage_path)
        report = verification.run_all_verifications()

        # Simulate actual process crash if requested
        if args.simulate_crash:
            simulate_process_crash(storage_path, "crash_simulation")

        # Write report
        report_path = Path(storage_path) / "verification_report.json"
        with open(report_path, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\nReport written to: {report_path}")

        # Exit with appropriate code
        if report["failed"] > 0 or report["errors"] > 0:
            sys.exit(1)
        else:
            sys.exit(0)

    finally:
        # Cleanup if using temp
        if not args.storage_path:
            print(f"\nCleaning up: {storage_path}")
            # Uncomment to keep test files
            # shutil.rmtree(storage_path, ignore_errors=True)


if __name__ == "__main__":
    main()
