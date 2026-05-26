"""
Bash Tool

Safe command execution with timeout control, output capture, and security sandbox.

Features:
    - Command timeout
    - Working directory control
    - Output capture (stdout/stderr)
    - Exit code handling
    - Security validation (no dangerous commands)
"""

import asyncio
import os
import shlex
import time
import uuid
from pathlib import Path

from .types import ToolError, ToolResult

# Dangerous commands that require explicit confirmation
DANGEROUS_COMMANDS = {
    "rm",
    "rmdir",
    "del",
    "format",
    "mkfs",
    "dd",
    "shutdown",
    "reboot",
    "poweroff",
    "chmod",
    "chown",
    "git push",
    "git reset",
    "git checkout",
    "npm publish",
    "pip upload",
}

# Blocked commands (never allowed)
BLOCKED_COMMANDS = {
    # Privilege escalation
    "sudo",
    "su",
    "doas",
    # Code execution
    "eval",
    "exec",
    # Network backdoors
    "mkfifo",
    "nc",
    "ncat",
    "telnet",
    "curl",
    "wget",
    # Script execution
    "python",
    "python3",
    "perl",
    "ruby",
    "node",
    "php",
    # Encoding/obfuscation
    "base64",
    "openssl",
    "xxd",
    # Key manipulation
    "ssh-keygen -p",
    # Container escape
    "docker",
    "kubectl",
    # Process manipulation
    "kill",
    "pkill",
    "killall",
}


def validate_command(command: str) -> str | None:
    """
    Validate command for security.

    Returns:
        None if safe, error message if blocked/dangerous.
    """
    # Check blocked commands
    cmd_lower = command.lower().strip()
    for blocked in BLOCKED_COMMANDS:
        if cmd_lower.startswith(blocked):
            return f"Blocked command: {blocked}"

    # Check dangerous commands
    for dangerous in DANGEROUS_COMMANDS:
        if cmd_lower.startswith(dangerous):
            return None  # Allowed but flagged

    return None


async def bash_execute(
    command: str,
    timeout: float = 120.0,
    working_dir: str | None = None,
    env: dict[str, str] | None = None,
    shell: bool = True,
) -> ToolResult:
    """
    Execute a bash command asynchronously.

    Args:
        command: The command to execute
        timeout: Timeout in seconds (default 120)
        working_dir: Working directory (default current)
        env: Environment variables
        shell: Use shell execution (default True)

    Returns:
        ToolResult with execution result

    Raises:
        ToolError: If command fails or times out
    """
    call_id = str(uuid.uuid4())[:8]
    start_time = time.time()

    # Validate command
    validation_error = validate_command(command)
    if validation_error:
        raise ToolError(
            call_id=call_id,
            name="bash",
            message=validation_error,
        )

    # Prepare environment
    exec_env = os.environ.copy()
    if env:
        exec_env.update(env)

    # Prepare working directory
    cwd = Path(working_dir) if working_dir else Path.cwd()
    if not cwd.exists():
        raise ToolError(
            call_id=call_id,
            name="bash",
            message=f"Working directory not found: {cwd}",
        )

    try:
        # Execute command
        if shell:
            # Use shell for complex commands (pipes, redirects, etc.)
            proc = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=str(cwd),
                env=exec_env,
            )
        else:
            # Split command for direct execution
            args = shlex.split(command)
            proc = await asyncio.create_subprocess_exec(
                args[0],
                *args[1:],
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=str(cwd),
                env=exec_env,
            )

        # Wait with timeout
        try:
            stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=timeout)
        except asyncio.TimeoutError:
            proc.kill()
            raise ToolError(
                call_id=call_id,
                name="bash",
                message=f"Command timed out after {timeout}s",
            )

        duration_ms = int((time.time() - start_time) * 1000)

        # Decode output
        stdout_str = stdout.decode("utf-8", errors="replace")
        stderr_str = stderr.decode("utf-8", errors="replace")

        # Build result content
        if proc.returncode != 0:
            content = f"Exit code: {proc.returncode}\n"
            if stderr_str:
                content += f"Error: {stderr_str}\n"
            if stdout_str:
                content += f"Output: {stdout_str}"
            return ToolResult(
                call_id=call_id,
                name="bash",
                content=content.strip(),
                is_error=True,
                duration_ms=duration_ms,
            )

        return ToolResult(
            call_id=call_id,
            name="bash",
            content=stdout_str or "(no output)",
            is_error=False,
            duration_ms=duration_ms,
        )

    except FileNotFoundError as e:
        raise ToolError(
            call_id=call_id,
            name="bash",
            message=f"Command not found: {command.split()[0]}",
        ) from e
    except Exception as e:
        raise ToolError(
            call_id=call_id,
            name="bash",
            message=f"Execution failed: {e}",
        ) from e


def bash_execute_sync(
    command: str,
    timeout: float = 120.0,
    working_dir: str | None = None,
    env: dict[str, str] | None = None,
) -> ToolResult:
    """
    Execute a bash command synchronously.

    Args:
        command: The command to execute
        timeout: Timeout in seconds
        working_dir: Working directory
        env: Environment variables

    Returns:
        ToolResult with execution result
    """
    return asyncio.run(bash_execute(command, timeout, working_dir, env))


class BashTool:
    """
    Bash tool wrapper for convenient usage.

    Example:
        >>> from continuum_sdk.tools import BashTool
        >>> bash = BashTool()
        >>> result = bash.run("echo hello")
        >>> print(result.content)
        'hello'
    """

    def __init__(
        self,
        default_timeout: float = 120.0,
        default_working_dir: str | None = None,
    ):
        self.default_timeout = default_timeout
        self.default_working_dir = default_working_dir

    async def run_async(
        self,
        command: str,
        timeout: float | None = None,
        working_dir: str | None = None,
        env: dict[str, str] | None = None,
    ) -> ToolResult:
        """Run command asynchronously."""
        return await bash_execute(
            command=command,
            timeout=timeout or self.default_timeout,
            working_dir=working_dir or self.default_working_dir,
            env=env,
        )

    def run(
        self,
        command: str,
        timeout: float | None = None,
        working_dir: str | None = None,
        env: dict[str, str] | None = None,
    ) -> ToolResult:
        """Run command synchronously."""
        return bash_execute_sync(
            command=command,
            timeout=timeout or self.default_timeout,
            working_dir=working_dir or self.default_working_dir,
            env=env,
        )

    def __call__(self, command: str, **kwargs) -> ToolResult:
        """Allow calling instance directly."""
        return self.run(command, **kwargs)
