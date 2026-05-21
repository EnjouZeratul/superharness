# Continuum 设计规范与实施计划

> 版本: v2.0
> 日期: 2026-05-09
> 基于: 两轮专家评审 (安全/架构/实现/API/性能/项目管理)

---

## 重要说明

**本文档性质**：设计规范 + 实施计划

项目当前状态：
- **无代码基础**：项目只有文档，没有 `src/` 目录和任何 `.py` 文件
- **所有任务都是新开发**：非"改进"或"修复"，而是从零开始实现

---

## 一、项目初始化 (Phase 0) - 必须最先完成

### 1.1 目录结构创建

```
continuum/
├── src/continuum/
│   ├── __init__.py
│   ├── types.py              # 基础类型定义
│   ├── protocols.py          # 协议定义
│   ├── exceptions.py         # 异常定义
│   │
│   ├── core/
│   │   ├── __init__.py
│   │   ├── message.py        # Message 数据模型
│   │   ├── context.py        # ContextManager
│   │   ├── guardrail.py      # Guardrail 验证
│   │   └── parallel.py       # 并行执行
│   │
│   ├── providers/
│   │   ├── __init__.py
│   │   ├── base.py           # LLMProvider 抽象
│   │   ├── openai.py
│   │   └── anthropic.py
│   │
│   ├── storage/
│   │   ├── __init__.py
│   │   ├── base.py           # StorageBackend 抽象
│   │   ├── file_storage.py
│   │   ├── sqlite_storage.py
│   │   └── redis_storage.py
│   │
│   ├── agents/
│   │   ├── __init__.py
│   │   ├── base.py           # Agent 抽象
│   │   ├── handoff.py        # Handoff 机制
│   │   └── memory.py         # ProjectMemory
│   │
│   ├── tools/
│   │   ├── __init__.py
│   │   ├── base.py           # Tool 抽象
│   │   ├── shell.py          # Shell 工具 (安全设计)
│   │   ├── file_ops.py       # 文件操作
│   │   └── search.py         # 搜索工具
│   │
│   ├── sandbox/
│   │   ├── __init__.py
│   │   ├── local.py          # 本地沙箱
│   │   └── docker.py         # Docker 沙箱
│   │
│   ├── security/
│   │   ├── __init__.py
│   │   ├── key_manager.py    # API Key 管理
│   │   └── validator.py      # 输入验证
│   │
│   ├── mcp/
│   │   ├── __init__.py
│   │   └── client.py         # MCP 客户端
│   │
│   └── diagnostics/
│       ├── __init__.py
│       └── health.py         # 健康检查
│
├── tests/
│   ├── conftest.py           # pytest fixtures
│   ├── fake_provider.py      # FakeLLMProvider
│   ├── unit/
│   ├── integration/
│   ├── contracts/            # 契约测试
│   └── performance/          # 性能基准测试
│
├── docs/
├── examples/
├── pyproject.toml
└── README.md
```

### 1.2 依赖配置 (pyproject.toml)

```toml
[project]
name = "continuum"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = [
    "httpx>=0.25.0",
    "pydantic>=2.0.0",
    "tiktoken>=0.5.0",
    "pyyaml>=6.0",
    "anyio>=4.0.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0.0",
    "pytest-asyncio>=0.21.0",
    "pytest-cov>=4.0.0",
    "ruff>=0.1.0",
    "mypy>=1.0.0",
]
keyring = ["keyring>=24.0.0"]
docker = ["aiodocker>=0.21.0"]
redis = ["redis>=5.0.0"]
sqlite = ["aiosqlite>=0.19.0"]
```

### 1.3 核心类型定义 (types.py)

```python
from typing import Literal, Union, Annotated
from pydantic import BaseModel, Field
from datetime import datetime

# 消息角色 - 使用 Literal 类型
MessageRole = Literal["user", "assistant", "system", "tool", "function"]

class ContentPart(BaseModel):
    """多模态内容块"""
    type: Literal["text", "image_url", "image_base64"]
    text: str | None = None
    image_url: str | None = None
    media_type: str | None = None

class ToolCall(BaseModel):
    """工具调用请求"""
    id: str
    type: Literal["function"] = "function"
    function: "FunctionCall"

class FunctionCall(BaseModel):
    """函数调用"""
    name: str
    arguments: str  # JSON 字符串

class Message(BaseModel):
    """统一消息格式"""
    role: MessageRole
    content: str | list[ContentPart]
    name: str | None = None
    tool_call_id: str | None = None
    tool_calls: list[ToolCall] | None = None
    timestamp: datetime = Field(default_factory=datetime.now)
    metadata: dict = Field(default_factory=dict)

    # 工厂方法
    @classmethod
    def user(cls, content: str) -> "Message":
        return cls(role="user", content=content)

    @classmethod
    def assistant(cls, content: str) -> "Message":
        return cls(role="assistant", content=content)

    @classmethod
    def system(cls, content: str) -> "Message":
        return cls(role="system", content=content)
```

### 1.4 FakeLLMProvider (测试基础设施)

```python
# tests/fake_provider.py
from typing import AsyncIterator

class FakeLLMProvider:
    """模拟 LLM Provider，用于测试"""

    def __init__(self, responses: list[str] | None = None):
        self.responses = responses or []
        self.call_count = 0
        self.call_history: list[list[Message]] = []
        self.tool_call_history: list[list[ToolCall]] = []

    async def chat(
        self,
        messages: list[Message],
        tools: list[dict] | None = None,
        **kwargs
    ) -> Message:
        """模拟聊天"""
        self.call_history.append(messages)
        response = self.responses[self.call_count % len(self.responses)]
        self.call_count += 1
        return Message(role="assistant", content=response)

    async def chat_stream(
        self,
        messages: list[Message],
        **kwargs
    ) -> AsyncIterator[dict]:
        """模拟流式输出"""
        response = await self.chat(messages, **kwargs)
        for char in response.content:
            yield {"delta": {"content": char}}

    def set_responses(self, responses: list[str]):
        """设置响应序列"""
        self.responses = responses
        self.call_count = 0

    def assert_called_with(self, expected_messages: list[Message]):
        """断言最后一次调用"""
        assert self.call_history[-1] == expected_messages

    def assert_call_count(self, count: int):
        """断言调用次数"""
        assert self.call_count == count
```

---

## 二、安全设计规范

### 2.1 Shell 命令安全 (S1)

**评分**: 原方案 5/10 → 改进后 8/10

#### 问题分析

| 攻击向量 | 原方案 | 改进方案 |
|----------|--------|----------|
| 参数注入 | 未验证 | 参数白名单 + 类型验证 |
| 命令路径劫持 | 未检查 | 绝对路径白名单 |
| 环境变量注入 | 未控制 | 清理环境变量 |
| 资源耗尽 | 无限制 | 超时 + 资源限制 |

#### 改进实现

```python
# src/continuum/tools/shell.py
import shlex
import asyncio
import subprocess
from pathlib import Path
from typing import Literal

class ShellTool:
    """安全的 Shell 命令执行"""

    # 命令白名单 (绝对路径)
    ALLOWED_COMMANDS = {
        "ls": "/bin/ls",
        "cat": "/bin/cat",
        "grep": "/bin/grep",
        "find": "/usr/bin/find",
        "python": sys.executable,
    }

    # 参数验证规则
    ARGUMENT_RULES = {
        "ls": {"allow_flags": ["-l", "-a", "-la", "-al"], "max_args": 10},
        "cat": {"allow_flags": [], "max_args": 5, "require_file_arg": True},
        "grep": {"allow_flags": ["-r", "-i", "-n"], "max_args": 10},
        "find": {"allow_flags": ["-name", "-type"], "max_args": 10},
    }

    # 危险参数模式
    DANGEROUS_PATTERNS = [
        r'\.\./',           # 路径遍历
        r'[;&|`$]',         # Shell 特殊字符
        r'\$\([^)]+\)',     # 命令替换
        r'`[^`]+`',         # 反引号命令替换
        r'>',               # 重定向
        r'<',               # 输入重定向
    ]

    async def execute(
        self,
        command: str,
        timeout: float = 30.0,
        max_output_size: int = 1024 * 1024  # 1MB
    ) -> str:
        """
        安全执行命令

        Args:
            command: 命令字符串
            timeout: 执行超时
            max_output_size: 最大输出大小

        Returns:
            命令输出

        Raises:
            SecurityError: 命令不在白名单或参数验证失败
            TimeoutError: 执行超时
        """
        # 1. 解析命令
        try:
            args = shlex.split(command)
        except ValueError as e:
            raise SecurityError(f"Invalid command syntax: {e}")

        if not args:
            raise SecurityError("Empty command")

        base_cmd = args[0]

        # 2. 验证命令在白名单
        if base_cmd not in self.ALLOWED_COMMANDS:
            raise SecurityError(
                f"Command '{base_cmd}' not allowed. "
                f"Allowed: {list(self.ALLOWED_COMMANDS.keys())}"
            )

        # 3. 获取绝对路径
        cmd_path = self.ALLOWED_COMMANDS[base_cmd]

        # 4. 验证参数
        rules = self.ARGUMENT_RULES.get(base_cmd, {})
        validated_args = self._validate_arguments(base_cmd, args[1:], rules)

        # 5. 检查危险模式
        for pattern in self.DANGEROUS_PATTERNS:
            if re.search(pattern, command):
                raise SecurityError(f" Dangerous pattern detected: {pattern}")

        # 6. 执行命令 (禁用 shell=True)
        full_args = [cmd_path] + validated_args

        # 清理环境变量
        safe_env = {
            "PATH": "/usr/bin:/bin",
            "HOME": os.environ.get("HOME", "/tmp"),
            "LANG": "C.UTF-8",
        }

        process = await asyncio.create_subprocess_exec(
            *full_args,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            env=safe_env
        )

        try:
            stdout, stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=timeout
            )

            if process.returncode != 0:
                return f"Error (exit {process.returncode}): {stderr.decode()}"

            output = stdout.decode()
            if len(output) > max_output_size:
                output = output[:max_output_size] + "\n... [truncated]"

            return output

        except asyncio.TimeoutError:
            process.kill()
            await process.wait()
            raise TimeoutError(f"Command timed out after {timeout}s")

    def _validate_arguments(
        self,
        cmd: str,
        args: list[str],
        rules: dict
    ) -> list[str]:
        """验证参数"""
        max_args = rules.get("max_args", 10)
        if len(args) > max_args:
            raise SecurityError(f"Too many arguments (max {max_args})")

        allow_flags = rules.get("allow_flags", [])
        validated = []

        for arg in args:
            if arg.startswith("-"):
                if arg not in allow_flags:
                    raise SecurityError(f"Flag '{arg}' not allowed for '{cmd}'")
                validated.append(arg)
            else:
                # 非标志参数，检查路径安全性
                if rules.get("require_file_arg"):
                    # 文件路径验证
                    if ".." in arg:
                        raise SecurityError(f"Path traversal detected: {arg}")
                validated.append(arg)

        return validated
```

### 2.2 API Key 安全存储 (S2)

**评分**: 原方案 6/10 → 改进后 8/10

#### 问题分析

| 问题 | keyring | 环境变量 | 改进方案 |
|------|---------|----------|----------|
| 跨平台兼容 | Linux服务器可能失败 | 全平台支持 | 分层策略 |
| 安全边界 | 同用户进程可读 | 子进程可见 | 加密文件备选 |
| 无加密内存 | 明文加载 | 明文 | bytearray + 立即清零 |
| 无审计日志 | 无 | 无 | 访问日志 |

#### 改进实现

```python
# src/continuum/security/key_manager.py
import os
import json
import keyring
from pathlib import Path
from typing import Optional
from enum import Enum
from datetime import datetime
import logging

logger = logging.getLogger(__name__)

class StorageMethod(Enum):
    KEYRING = "keyring"
    ENV_VAR = "env_var"
    ENCRYPTED_FILE = "encrypted_file"

class SecureKeyManager:
    """分层 API Key 安全管理"""

    SERVICE_NAME = "continuum"

    def __init__(
        self,
        prefer_keyring: bool = True,
        encrypted_file_path: Path | None = None
    ):
        self.prefer_keyring = prefer_keyring
        self.encrypted_file_path = encrypted_file_path
        self._access_log: list[dict] = []
        self._keyring_available = self._check_keyring()

    def _check_keyring(self) -> bool:
        """检查 keyring 是否可用"""
        try:
            keyring.get_password(self.SERVICE_NAME, "__test__")
            return True
        except Exception:
            logger.warning("keyring not available, falling back to env vars")
            return False

    def get_key(self, provider: str) -> str | None:
        """
        获取 API Key

        优先级:
        1. 环境变量 (开发/CI 推荐)
        2. keyring (桌面环境)
        3. 加密文件 (服务器环境)
        """
        # 记录访问
        self._log_access(provider, "read")

        # 1. 环境变量
        env_key = self._get_from_env(provider)
        if env_key:
            return env_key

        # 2. keyring
        if self.prefer_keyring and self._keyring_available:
            key = self._get_from_keyring(provider)
            if key:
                return key

        # 3. 加密文件
        if self.encrypted_file_path:
            return self._get_from_encrypted_file(provider)

        return None

    def _get_from_env(self, provider: str) -> str | None:
        """从环境变量获取"""
        env_map = {
            "openai": "OPENAI_API_KEY",
            "anthropic": "ANTHROPIC_API_KEY",
            "deepseek": "DEEPSEEK_API_KEY",
            "zhipuai": "ZHIPUAI_API_KEY",
        }
        env_name = env_map.get(provider.lower())
        if env_name:
            return os.environ.get(env_name)
        return None

    def _get_from_keyring(self, provider: str) -> str | None:
        """从 keyring 获取"""
        try:
            return keyring.get_password(self.SERVICE_NAME, provider)
        except Exception as e:
            logger.warning(f"keyring access failed: {e}")
            return None

    def _get_from_encrypted_file(self, provider: str) -> str | None:
        """从加密文件获取"""
        # 实现加密文件存储 (使用 master key)
        # 生产环境建议使用云 KMS
        pass

    def store_key(
        self,
        provider: str,
        api_key: str,
        method: StorageMethod = StorageMethod.KEYRING
    ) -> bool:
        """存储 API Key"""
        self._log_access(provider, "write")

        if method == StorageMethod.KEYRING and self._keyring_available:
            try:
                keyring.set_password(self.SERVICE_NAME, provider, api_key)
                return True
            except Exception as e:
                logger.error(f"keyring storage failed: {e}")
                return False

        elif method == StorageMethod.ENV_VAR:
            logger.warning("Cannot store to env var, set it manually")
            return False

        elif method == StorageMethod.ENCRYPTED_FILE:
            return self._store_to_encrypted_file(provider, api_key)

        return False

    def delete_key(self, provider: str) -> bool:
        """删除 API Key"""
        self._log_access(provider, "delete")

        # 从所有存储位置删除
        success = False

        if self._keyring_available:
            try:
                keyring.delete_password(self.SERVICE_NAME, provider)
                success = True
            except keyring.errors.PasswordNotFoundError:
                pass

        return success

    def _log_access(self, provider: str, action: str):
        """记录访问日志"""
        self._access_log.append({
            "timestamp": datetime.now().isoformat(),
            "provider": provider,
            "action": action,
        })

    def get_access_log(self) -> list[dict]:
        """获取访问日志"""
        return self._access_log.copy()
```

### 2.3 沙箱执行环境 (S3)

**评分**: 原方案 4/10 → 改进后 7/10

#### 问题分析

| LocalSandbox 能防止 | LocalSandbox 不能防止 |
|---------------------|----------------------|
| 路径遍历攻击 | 进程逃逸 |
| 命令无限时执行 | 网络访问 |
| - | 资源耗尽 (CPU/内存/fork bomb) |
| - | 系统调用 |

#### 改进实现

```python
# src/continuum/sandbox/local.py
import os
import asyncio
import resource
import shutil
import tempfile
from pathlib import Path
from typing import Optional
from dataclasses import dataclass

@dataclass
class ResourceLimits:
    """资源限制"""
    max_cpu_seconds: int = 30
    max_memory_mb: int = 512
    max_processes: int = 10
    max_file_size_mb: int = 100
    network_enabled: bool = False

class LocalSandbox:
    """
    轻量级本地沙箱

    注意：这是基本的隔离，不是安全容器！
    生产环境请使用 DockerSandbox 或云沙箱。
    """

    def __init__(
        self,
        workspace_root: Path | None = None,
        limits: ResourceLimits | None = None
    ):
        self.workspace_root = workspace_root or Path(tempfile.mkdtemp())
        self.workspace_root.mkdir(parents=True, exist_ok=True)
        self.limits = limits or ResourceLimits()
        self._setup_workspace()

    def _setup_workspace(self):
        """设置工作空间"""
        # 创建基本目录结构
        (self.workspace_root / "workspace").mkdir(exist_ok=True)
        (self.workspace_root / "tmp").mkdir(exist_ok=True)

    def safe_path(self, path: str) -> Path:
        """
        安全路径解析，防止路径遍历

        Raises:
            SecurityError: 路径遍历检测
        """
        # 解析绝对路径
        target = (self.workspace_root / path).resolve()

        # 检查是否在工作空间内
        try:
            target.relative_to(self.workspace_root.resolve())
        except ValueError:
            raise SecurityError(f"Path traversal detected: {path}")

        # 检查符号链接
        if target.is_symlink():
            real_target = target.resolve()
            try:
                real_target.relative_to(self.workspace_root.resolve())
            except ValueError:
                raise SecurityError(f"Symlink escape detected: {path}")

        return target

    async def execute(
        self,
        command: str,
        timeout: float | None = None,
        env: dict | None = None
    ) -> "ExecutionResult":
        """
        在沙箱中执行命令

        注意：此方法不提供完全隔离，仅用于基本防护。
        """
        timeout = timeout or self.limits.max_cpu_seconds

        # 设置资源限制 (Unix only)
        def set_limits():
            if hasattr(resource, 'RLIMIT_CPU'):
                resource.setrlimit(
                    resource.RLIMIT_CPU,
                    (self.limits.max_cpu_seconds, self.limits.max_cpu_seconds + 1)
                )
            if hasattr(resource, 'RLIMIT_AS'):
                resource.setrlimit(
                    resource.RLIMIT_AS,
                    (self.limits.max_memory_mb * 1024 * 1024,
                     self.limits.max_memory_mb * 1024 * 1024)
                )
            if hasattr(resource, 'RLIMIT_NPROC'):
                resource.setrlimit(
                    resource.RLIMIT_NPROC,
                    (self.limits.max_processes, self.limits.max_processes)
                )

        # 构建安全环境变量
        safe_env = {
            "PATH": "/usr/bin:/bin",
            "HOME": str(self.workspace_root),
            "TMPDIR": str(self.workspace_root / "tmp"),
            "LANG": "C.UTF-8",
        }
        if env:
            # 只允许白名单环境变量
            allowed_env_keys = {"PYTHONPATH", "PYTHONIOENCODING"}
            for key in allowed_env_keys:
                if key in env:
                    safe_env[key] = env[key]

        # 执行命令
        args = shlex.split(command)
        process = await asyncio.create_subprocess_exec(
            *args,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=self.workspace_root / "workspace",
            env=safe_env,
            preexec_fn=set_limits if os.name != 'nt' else None
        )

        try:
            stdout, stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=timeout
            )

            return ExecutionResult(
                exit_code=process.returncode,
                stdout=stdout.decode(),
                stderr=stderr.decode(),
                timed_out=False
            )

        except asyncio.TimeoutError:
            process.kill()
            await process.wait()
            return ExecutionResult(
                exit_code=-1,
                stdout="",
                stderr=f"Process timed out after {timeout}s",
                timed_out=True
            )


@dataclass
class ExecutionResult:
    """执行结果"""
    exit_code: int
    stdout: str
    stderr: str
    timed_out: bool = False
```

```python
# src/continuum/sandbox/docker.py
"""Docker 沙箱 - 完整隔离"""

class DockerSandbox:
    """
    Docker 完整隔离沙箱

    生产环境推荐，提供：
    - 进程隔离
    - 网络隔离
    - 文件系统隔离
    - 资源限制
    """

    def __init__(
        self,
        image: str = "python:3.11-slim",
        workspace_root: Path | None = None,
        limits: ResourceLimits | None = None
    ):
        self.image = image
        self.workspace_root = workspace_root
        self.limits = limits or ResourceLimits()
        self.container = None
        self.docker = None

    async def start(self):
        """启动沙箱容器"""
        import aiodocker

        self.docker = aiodocker.Docker()

        # 安全配置
        host_config = {
            "CpuQuota": self.limits.max_cpu_seconds * 1000,
            "Memory": self.limits.max_memory_mb * 1024 * 1024,
            "PidsLimit": self.limits.max_processes,
            "SecurityOpt": ["no-new-privileges"],
            "CapDrop": ["ALL"],
            "ReadonlyRootfs": True,
        }

        if not self.limits.network_enabled:
            host_config["NetworkDisabled"] = True

        self.container = await self.docker.containers.run(
            self.image,
            detach=True,
            working_dir="/workspace",
            host_config=host_config,
            volumes={
                str(self.workspace_root): {
                    "bind": "/workspace",
                    "mode": "rw"
                }
            }
        )

    async def execute(self, command: str, timeout: float = 30.0) -> ExecutionResult:
        """在容器中执行命令"""
        if not self.container:
            raise RuntimeError("Sandbox not started")

        exec_instance = await self.container.exec(
            command,
            stdout=True,
            stderr=True
        )

        stream = await exec_instance.start()

        # 收集输出
        stdout_chunks = []
        stderr_chunks = []

        async for chunk in stream:
            if chunk["type"] == "stdout":
                stdout_chunks.append(chunk["data"])
            elif chunk["type"] == "stderr":
                stderr_chunks.append(chunk["data"])

        inspect = await exec_instance.inspect()

        return ExecutionResult(
            exit_code=inspect["ExitCode"],
            stdout="".join(stdout_chunks),
            stderr="".join(stderr_chunks)
        )

    async def stop(self):
        """停止并清理容器"""
        if self.container:
            await self.container.stop()
            await self.container.delete()
            self.container = None

        if self.docker:
            await self.docker.close()
            self.docker = None
```

### 2.4 MCP 进程注入防护 (S4)

**评分**: 原方案 5/10 → 改进后 8/10

```python
# src/continuum/mcp/client.py
import asyncio
import shutil
import hashlib
from pathlib import Path
from typing import Optional
from pydantic import BaseModel

class MCPServerConfig(BaseModel):
    """MCP 服务器配置"""
    name: str
    command: str
    args: list[str] = []
    version: str | None = None
    allowed_args: list[str] | None = None  # 允许的额外参数
    checksum: str | None = None  # 可执行文件校验和

# 配置文件驱动的服务器列表
DEFAULT_MCP_SERVERS = {
    "filesystem": MCPServerConfig(
        name="filesystem",
        command="uvx",
        args=["mcp-server-filesystem"],
        allowed_args=["."],  # 只允许当前目录
    ),
    "github": MCPServerConfig(
        name="github",
        command="uvx",
        args=["mcp-server-github"],
        allowed_args=[],
    ),
}

class MCPClient:
    """安全的 MCP 客户端"""

    def __init__(
        self,
        servers: dict[str, MCPServerConfig] | None = None,
        config_path: Path | None = None
    ):
        # 从配置文件加载
        if config_path and config_path.exists():
            self.servers = self._load_config(config_path)
        else:
            self.servers = servers or DEFAULT_MCP_SERVERS

        self.connections: dict[str, asyncio.subprocess.Process] = {}

    def _load_config(self, config_path: Path) -> dict[str, MCPServerConfig]:
        """从 YAML 加载配置"""
        import yaml
        with open(config_path) as f:
            data = yaml.safe_load(f)

        servers = {}
        for name, cfg in data.get("mcp_servers", {}).items():
            servers[name] = MCPServerConfig(name=name, **cfg)

        return servers

    async def connect(
        self,
        name: str,
        extra_args: list[str] | None = None
    ):
        """
        连接 MCP 服务器

        Raises:
            SecurityError: 服务器未授权或参数验证失败
        """
        # 1. 检查服务器在授权列表
        if name not in self.servers:
            raise SecurityError(
                f"Unknown MCP server: {name}. "
                f"Allowed: {list(self.servers.keys())}"
            )

        config = self.servers[name]

        # 2. 验证命令路径
        cmd_path = shutil.which(config.command)
        if not cmd_path:
            raise SecurityError(f"Command not found: {config.command}")

        # 3. 可选：验证校验和
        if config.checksum:
            actual_checksum = self._compute_checksum(cmd_path)
            if actual_checksum != config.checksum:
                raise SecurityError(
                    f"Checksum mismatch for {config.command}. "
                    f"Expected: {config.checksum}, Got: {actual_checksum}"
                )

        # 4. 验证额外参数
        final_args = list(config.args)
        if extra_args:
            if config.allowed_args is None:
                raise SecurityError(f"Server '{name}' does not allow extra args")

            for arg in extra_args:
                if arg not in config.allowed_args:
                    raise SecurityError(f"Argument '{arg}' not allowed for '{name}'")
                final_args.append(arg)

        # 5. 启动进程
        process = await asyncio.create_subprocess_exec(
            cmd_path,
            *final_args,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )

        # 6. MCP 握手
        await self._handshake(process)

        self.connections[name] = process

    def _compute_checksum(self, filepath: str) -> str:
        """计算文件 SHA256 校验和"""
        sha256 = hashlib.sha256()
        with open(filepath, "rb") as f:
            for chunk in iter(lambda: f.read(8192), b""):
                sha256.update(chunk)
        return sha256.hexdigest()

    async def _handshake(self, process: asyncio.subprocess.Process):
        """MCP 协议握手"""
        import json

        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "clientInfo": {
                    "name": "continuum",
                    "version": "1.0.0"
                }
            }
        }

        process.stdin.write((json.dumps(init_request) + "\n").encode())
        await process.stdin.drain()

        response = await process.stdout.readline()
        # 验证响应...

    async def list_tools(self, server_name: str) -> list[dict]:
        """获取服务器提供的工具列表"""
        if server_name not in self.connections:
            raise ValueError(f"Not connected to: {server_name}")

        # 发送 tools/list 请求
        # ...
```

### 2.5 其他安全风险 (新增)

```python
# src/continuum/security/validator.py
"""输入验证和输出过滤"""

class InputValidator:
    """输入验证器"""

    @staticmethod
    def sanitize_prompt(prompt: str) -> str:
        """
        清理用户输入，防止 Prompt 注入

        注意：完全防止 Prompt 注入很难，这只是基本防护
        """
        # 移除可疑的模式
        dangerous_patterns = [
            r"ignore (all )?previous instructions",
            r"system prompt",
            r"you are now",
            r"<\|.*?\|>",  # 特殊 token
        ]

        sanitized = prompt
        for pattern in dangerous_patterns:
            sanitized = re.sub(pattern, "", sanitized, flags=re.IGNORECASE)

        return sanitized

    @staticmethod
    def validate_tool_args(tool_name: str, args: dict) -> dict:
        """验证工具调用参数"""
        # 根据工具定义验证参数
        pass


class OutputFilter:
    """输出过滤器"""

    SENSITIVE_PATTERNS = [
        r"sk-[a-zA-Z0-9]{20,}",  # OpenAI API Key
        r"sk-ant-[a-zA-Z0-9-]+",  # Anthropic API Key
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",  # Email
        r"\b\d{16}\b",  # 信用卡号
    ]

    @classmethod
    def redact_sensitive(cls, output: str) -> str:
        """脱敏敏感信息"""
        import re
        redacted = output
        for pattern in cls.SENSITIVE_PATTERNS:
            redacted = re.sub(pattern, "[REDACTED]", redacted)
        return redacted
```

---

## 三、架构设计规范

### 3.1 ContextManager (A1)

**评分**: 原方案 6/10 → 改进后 8/10

```python
# src/continuum/core/context.py
import asyncio
import tiktoken
from abc import ABC, abstractmethod
from typing import Protocol, Iterator

class TokenizerProtocol(Protocol):
    """分词器协议"""
    def encode(self, text: str) -> list[int]: ...
    def decode(self, tokens: list[int]) -> str: ...
    def count_tokens(self, text: str) -> int: ...

class TiktokenTokenizer:
    """tiktoken 分词器实现"""

    def __init__(self, encoding_name: str = "cl100k_base"):
        self.encoding = tiktoken.get_encoding(encoding_name)

    def count_tokens(self, text: str) -> int:
        return len(self.encoding.encode(text))


class CompactionStrategy(ABC):
    """压缩策略抽象"""

    @abstractmethod
    async def compact(
        self,
        messages: list[Message],
        target_tokens: int
    ) -> list[Message]:
        """将消息压缩到目标 token 数以内"""
        pass


class TruncationCompaction(CompactionStrategy):
    """截断压缩策略"""

    def __init__(self, keep_system: bool = True, keep_recent: int = 2):
        self.keep_system = keep_system
        self.keep_recent = keep_recent

    async def compact(
        self,
        messages: list[Message],
        target_tokens: int
    ) -> list[Message]:
        """保留系统消息和最近消息，截断中间历史"""
        result = []

        # 保留系统消息
        system_messages = [m for m in messages if m.role == "system"]
        result.extend(system_messages)

        # 保留最近消息
        recent_messages = [m for m in messages if m.role != "system"][-self.keep_recent:]
        result.extend(recent_messages)

        return result


class SummaryCompaction(CompactionStrategy):
    """摘要压缩策略"""

    def __init__(self, llm_provider: "LLMProvider"):
        self.llm = llm_provider

    async def compact(
        self,
        messages: list[Message],
        target_tokens: int
    ) -> list[Message]:
        """使用 LLM 生成历史摘要"""
        # 提取需要摘要的消息
        to_summarize = [m for m in messages if m.role != "system"]

        # 生成摘要
        summary_prompt = f"Summarize the following conversation:\n\n{to_summarize}"
        summary = await self.llm.chat([
            Message.system("You are a helpful assistant that summarizes conversations."),
            Message.user(summary_prompt)
        ])

        # 构建新消息列表
        result = [m for m in messages if m.role == "system"]
        result.append(Message.assistant(f"[Previous conversation summary: {summary.content}]"))

        return result


class ContextManager:
    """
    统一的上下文管理器

    功能：
    - Token 预算管理
    - 消息压缩
    - 线程安全
    """

    def __init__(
        self,
        max_tokens: int = 128000,
        reserved_tokens: int = 4096,
        tokenizer: TokenizerProtocol | None = None,
        compaction_strategy: CompactionStrategy | None = None
    ):
        self.max_tokens = max_tokens
        self.reserved_tokens = reserved_tokens
        self.tokenizer = tokenizer or TiktokenTokenizer()
        self.compaction_strategy = compaction_strategy or TruncationCompaction()

        self.messages: list[Message] = []
        self._lock = asyncio.Lock()

    def count_tokens(self, messages: list[Message] | None = None) -> int:
        """计算消息 Token 数"""
        messages = messages or self.messages
        total = 0
        for m in messages:
            total += self.tokenizer.count_tokens(m.content)
            if m.tool_calls:
                for tc in m.tool_calls:
                    total += self.tokenizer.count_tokens(tc.function.arguments)
            total += 4  # 消息格式开销
        return total

    def find_max_context(self) -> int:
        """二分搜索最大可用上下文"""
        low, high = 0, self.max_tokens - self.reserved_tokens

        while low < high:
            mid = (low + high + 1) // 2
            truncated = self._truncate_messages(mid)

            if self.count_tokens(truncated) <= mid:
                low = mid
            else:
                high = mid - 1

        return low

    async def add_message(self, message: Message) -> bool:
        """添加消息，自动管理预算"""
        async with self._lock:
            test_messages = self.messages + [message]

            if self.count_tokens(test_messages) > self.max_tokens - self.reserved_tokens:
                # 触发压缩
                self.messages = await self.compaction_strategy.compact(
                    self.messages,
                    self.max_tokens - self.reserved_tokens - self.count_tokens([message])
                )

            self.messages.append(message)
            return True

    def _truncate_messages(self, target_tokens: int) -> list[Message]:
        """截断消息到目标 token 数"""
        result = []
        current_tokens = 0

        for m in reversed(self.messages):
            msg_tokens = self.tokenizer.count_tokens(m.content) + 4
            if current_tokens + msg_tokens <= target_tokens:
                result.insert(0, m)
                current_tokens += msg_tokens
            else:
                break

        return result
```

### 3.2 StorageBackend (A2)

**评分**: 原方案 5/10 → 改进后 8/10

```python
# src/continuum/storage/base.py
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import AsyncIterator, Any
from contextlib import asynccontextmanager

@dataclass
class StorageOptions:
    """存储选项"""
    ttl: int | None = None  # 过期时间（秒）
    namespace: str = ""     # 命名空间
    version: int | None = None  # 版本号（用于乐观锁）

@dataclass
class ListOptions:
    """列表选项"""
    prefix: str = ""
    limit: int = 100
    offset: int = 0

class StorageBackend(ABC):
    """
    增强的存储后端抽象

    支持：
    - TTL
    - 分布式锁
    - 批量操作
    - 事务
    """

    @abstractmethod
    async def read(self, key: str) -> bytes | None:
        """读取数据"""
        pass

    @abstractmethod
    async def write(
        self,
        key: str,
        value: bytes | str,
        options: StorageOptions | None = None
    ) -> None:
        """写入数据"""
        pass

    @abstractmethod
    async def delete(self, key: str) -> bool:
        """删除数据"""
        pass

    @abstractmethod
    async def exists(self, key: str) -> bool:
        """检查键是否存在"""
        pass

    @abstractmethod
    async def list_keys(
        self,
        options: ListOptions | None = None
    ) -> AsyncIterator[str]:
        """异步迭代键列表"""
        pass

    @abstractmethod
    async def compare_and_swap(
        self,
        key: str,
        expected: bytes | None,
        new_value: bytes
    ) -> bool:
        """
        原子 CAS 操作

        用于实现分布式锁和乐观锁
        """
        pass

    @asynccontextmanager
    async def acquire_lock(
        self,
        key: str,
        timeout: float = 10.0
    ):
        """获取分布式锁"""
        lock_key = f"_lock:{key}"

        acquired = await self.compare_and_swap(lock_key, None, b"locked")
        if not acquired:
            raise ResourceLockedError(f"Failed to acquire lock for: {key}")

        try:
            yield
        finally:
            await self.delete(lock_key)

    # 批量操作
    async def read_many(self, keys: list[str]) -> dict[str, bytes | None]:
        """批量读取"""
        results = {}
        for key in keys:
            results[key] = await self.read(key)
        return results

    async def write_many(
        self,
        items: dict[str, bytes | str]
    ) -> None:
        """批量写入"""
        for key, value in items.items():
            await self.write(key, value)
```

### 3.3 Handoff 机制 (A3)

**评分**: 原方案 4/10 → 改进后 7/10

```python
# src/continuum/agents/handoff.py
from enum import Enum
from abc import ABC, abstractmethod
from typing import Callable, Awaitable
from pydantic import BaseModel

class HandoffMode(Enum):
    """Handoff 模式"""
    TRANSFER = "transfer"      # 完全转移控制权
    DELEGATE = "delegate"      # 委托执行，等待结果返回
    BROADCAST = "broadcast"    # 广播给多个 Agent

class HandoffCondition(ABC):
    """Handoff 触发条件"""

    @abstractmethod
    async def should_trigger(self, context: "AgentContext") -> bool:
        pass

class KeywordCondition(HandoffCondition):
    """关键词触发"""

    def __init__(self, keywords: list[str]):
        self.keywords = keywords

    async def should_trigger(self, context: "AgentContext") -> bool:
        return any(
            kw in context.last_message.content
            for kw in self.keywords
        )

class ToolUseCondition(HandoffCondition):
    """工具使用触发"""

    def __init__(self, tool_names: list[str]):
        self.tool_names = tool_names

    async def should_trigger(self, context: "AgentContext") -> bool:
        if not context.last_message.tool_calls:
            return False
        return any(
            tc.function.name in self.tool_names
            for tc in context.last_message.tool_calls
        )

class Handoff(BaseModel):
    """增强的 Handoff 定义"""

    target: str
    mode: HandoffMode = HandoffMode.TRANSFER
    message: str = ""
    conditions: list[HandoffCondition] = []

    # 状态传递选项
    inherit_context: bool = True
    inherit_memory: bool = True
    inherit_tools: bool = False

    # 控制选项
    timeout: float = 300.0
    max_retries: int = 2
    fallback_agent: str | None = None

    class Config:
        arbitrary_types_allowed = True

class HandoffManager:
    """Handoff 管理器"""

    def __init__(self, max_depth: int = 5):
        self.handoffs: dict[str, list[Handoff]] = {}
        self.agents: dict[str, "Agent"] = {}
        self.max_depth = max_depth
        self._call_stack: list[str] = []

    def register_agent(self, agent: "Agent"):
        """注册 Agent"""
        self.agents[agent.name] = agent

    def register_handoff(self, from_agent: str, handoff: Handoff):
        """注册 Handoff"""
        if from_agent not in self.handoffs:
            self.handoffs[from_agent] = []
        self.handoffs[from_agent].append(handoff)

    async def check_and_execute(
        self,
        agent_name: str,
        context: "AgentContext"
    ) -> str | None:
        """检查并执行 Handoff"""
        if agent_name not in self.handoffs:
            return None

        for handoff in self.handoffs[agent_name]:
            triggered = all(
                await cond.should_trigger(context)
                for cond in handoff.conditions
            )

            if triggered:
                return await self.execute_handoff(
                    agent_name,
                    handoff.target,
                    handoff.message or context.last_message.content,
                    context,
                    handoff
                )

        return None

    async def execute_handoff(
        self,
        from_agent: str,
        to_agent: str,
        message: str,
        context: "AgentContext",
        handoff: Handoff | None = None
    ) -> str:
        """执行 Handoff"""
        # 检查深度
        if len(self._call_stack) >= self.max_depth:
            raise HandoffDepthError(
                f"Max handoff depth {self.max_depth} exceeded"
            )

        # 检测循环
        if to_agent in self._call_stack:
            raise HandoffCycleError(
                f"Detected cycle: {' -> '.join(self._call_stack)} -> {to_agent}"
            )

        # 验证目标 Agent
        if to_agent not in self.agents:
            raise ValueError(f"Unknown agent: {to_agent}")

        self._call_stack.append(from_agent)

        try:
            target = self.agents[to_agent]

            # 构建传递上下文
            if handoff:
                transfer_context = self._build_transfer_context(context, handoff)
            else:
                transfer_context = context

            # 执行
            result = await asyncio.wait_for(
                target.run(message, context=transfer_context),
                timeout=handoff.timeout if handoff else 300.0
            )

            return result

        except Exception as e:
            # 尝试回退
            if handoff and handoff.fallback_agent:
                return await self.execute_handoff(
                    to_agent,
                    handoff.fallback_agent,
                    f"Handoff failed: {e}",
                    context,
                    None
                )
            raise

        finally:
            self._call_stack.pop()

    def _build_transfer_context(
        self,
        context: "AgentContext",
        handoff: Handoff
    ) -> "AgentContext":
        """构建传递上下文"""
        new_context = AgentContext()

        if handoff.inherit_context:
            new_context.messages = list(context.messages)

        if handoff.inherit_memory and context.memory:
            new_context.memory = context.memory

        return new_context


class HandoffDepthError(Exception):
    pass

class HandoffCycleError(Exception):
    pass
```

### 3.4 Guardrail 验证 (A4)

**评分**: 原方案 7/10 → 改进后 9/10

```python
# src/continuum/core/guardrail.py
from dataclasses import dataclass
from typing import Callable, Awaitable, Any

@dataclass
class ValidationResult:
    """结构化验证结果"""
    success: bool
    value: Any
    error_type: str | None = None
    error_details: str | None = None
    retry_hint: str | None = None  # 给 LLM 的修改建议

class Guardrail:
    """
    增强的 Guardrail

    支持：
    - 组合验证器
    - 结构化错误提示
    - 异步验证
    """

    def __init__(
        self,
        validator: Callable[[str], Awaitable[ValidationResult]],
        max_retries: int = 3,
        name: str = ""
    ):
        self.validator = validator
        self.max_retries = max_retries
        self.name = name or validator.__name__

    async def validate(self, output: str) -> ValidationResult:
        """验证输出"""
        return await self.validator(output)

    @classmethod
    def compose(
        cls,
        *guardrails: "Guardrail",
        mode: Literal["and", "or"] = "and"
    ) -> "Guardrail":
        """
        组合多个验证器

        mode="and": 所有验证通过才算通过
        mode="or": 任一验证通过即通过
        """
        async def composed_validator(output: str) -> ValidationResult:
            results = []

            for g in guardrails:
                result = await g.validate(output)
                results.append(result)

                if mode == "and" and not result.success:
                    return result

            if mode == "or":
                success_any = any(r.success for r in results)
                if success_any:
                    return ValidationResult(
                        success=True,
                        value=[r.value for r in results if r.success][0]
                    )
                else:
                    return ValidationResult(
                        success=False,
                        value=None,
                        error_type="all_failed",
                        error_details="All validators failed",
                        retry_hint="Try to satisfy at least one validator"
                    )

            # mode == "and"
            return ValidationResult(
                success=True,
                value=results[-1].value
            )

        return cls(
            composed_validator,
            name=f"composed_{'_'.join(g.name for g in guardrails)}"
        )


# 预定义验证器
def length_guardrail(min_length: int = 100, max_length: int | None = None) -> Guardrail:
    """长度验证"""
    async def validator(output: str) -> ValidationResult:
        if len(output) < min_length:
            return ValidationResult(
                success=False,
                value=None,
                error_type="too_short",
                error_details=f"Output length {len(output)} < {min_length}",
                retry_hint=f"Provide a longer response (at least {min_length} characters)"
            )

        if max_length and len(output) > max_length:
            return ValidationResult(
                success=False,
                value=None,
                error_type="too_long",
                error_details=f"Output length {len(output)} > {max_length}",
                retry_hint=f"Provide a shorter response (at most {max_length} characters)"
            )

        return ValidationResult(success=True, value=output)

    return Guardrail(validator, name=f"length_{min_length}_{max_length}")


def json_guardrail() -> Guardrail:
    """JSON 格式验证"""
    import json

    async def validator(output: str) -> ValidationResult:
        try:
            data = json.loads(output)
            return ValidationResult(success=True, value=data)
        except json.JSONDecodeError as e:
            return ValidationResult(
                success=False,
                value=None,
                error_type="invalid_json",
                error_details=str(e),
                retry_hint="Provide valid JSON output"
            )

    return Guardrail(validator, name="json")


def schema_guardrail(schema: dict) -> Guardrail:
    """JSON Schema 验证"""
    import json
    import jsonschema

    async def validator(output: str) -> ValidationResult:
        try:
            data = json.loads(output)
            jsonschema.validate(data, schema)
            return ValidationResult(success=True, value=data)
        except json.JSONDecodeError as e:
            return ValidationResult(
                success=False,
                value=None,
                error_type="invalid_json",
                error_details=str(e),
                retry_hint="Provide valid JSON output"
            )
        except jsonschema.ValidationError as e:
            return ValidationResult(
                success=False,
                value=None,
                error_type="schema_validation",
                error_details=str(e),
                retry_hint=f"Output must match schema: {schema}"
            )

    return Guardrail(validator, name="schema")
```

---

## 四、API 设计规范

### 4.1 quickstart() 入口 (API1)

**评分**: 原方案 7/10 → 改进后 9/10

```python
# src/continuum/__init__.py
from typing import Literal

def quickstart(
    model: str = "gpt-4-turbo",
    provider: Literal["openai", "anthropic"] = "openai",
    api_key: str | None = None,
    tools: list[str] | None = None,
    memory: bool = False,
    system_prompt: str | None = None,
    temperature: float = 0.7,
    max_tokens: int = 4096,
    **kwargs
) -> "Agent":
    """
    快速创建 Agent 实例

    这是最简单的入门方式，适合快速原型开发。

    Args:
        model: 模型名称
        provider: LLM 提供商
        api_key: API Key (不提供则从环境变量读取)
        tools: 内置工具列表 ["search", "file_ops", "shell"]
        memory: 是否启用项目记忆
        system_prompt: 系统提示
        temperature: 温度参数
        max_tokens: 最大输出 token

    Returns:
        配置好的 Agent 实例

    Examples:
        # 最简单用法
        agent = quickstart()
        response = await agent.run("Hello!")

        # 带工具
        agent = quickstart(tools=["search", "file_ops"])

        # 完整配置
        agent = quickstart(
            model="gpt-4-turbo",
            provider="openai",
            api_key=os.environ.get("OPENAI_API_KEY"),
            tools=["search"],
            memory=True,
            system_prompt="You are a helpful coding assistant."
        )
    """
    from .harness import Harness
    from .agents.memory import ProjectMemory

    # 获取 API Key
    if not api_key:
        from .security.key_manager import SecureKeyManager
        key_manager = SecureKeyManager()
        api_key = key_manager.get_key(provider)

        if not api_key:
            raise ValueError(
                f"No API key found for {provider}. "
                f"Set {provider.upper()}_API_KEY environment variable "
                f"or pass api_key parameter."
            )

    # 创建 Harness
    harness = Harness(
        model_provider=provider,
        model_name=model,
        api_key=api_key,
        temperature=temperature,
        max_tokens=max_tokens,
        **kwargs
    )

    # 加载工具
    if tools:
        for tool_name in tools:
            harness.load_builtin_tool(tool_name)

    # 创建 Agent
    agent = harness.create_agent(
        memory=ProjectMemory(".") if memory else None,
        system_prompt=system_prompt
    )

    return agent
```

### 4.2 health_check() 诊断 (API3)

**评分**: 原方案 6/10 → 改进后 8/10

```python
# src/continuum/diagnostics/health.py
from dataclasses import dataclass
from enum import Enum
from typing import Any
from datetime import datetime

class HealthStatus(str, Enum):
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"

@dataclass
class ComponentHealth:
    status: HealthStatus
    message: str | None = None
    latency_ms: float | None = None
    details: dict[str, Any] | None = None
    suggestion: str | None = None

async def health_check(
    components: list[str] | None = None,
    timeout: float = 10.0,
    detailed: bool = False
) -> dict[str, Any]:
    """
    系统健康检查

    Args:
        components: 要检查的组件列表 ["llm", "storage", "mcp"]
        timeout: 单个组件检查超时
        detailed: 是否返回详细信息

    Returns:
        健康状态报告
    """
    results = {
        "status": HealthStatus.HEALTHY,
        "version": __version__,
        "timestamp": datetime.now().isoformat(),
        "checks": {}
    }

    all_components = ["llm", "storage", "mcp", "environment"]
    components = components or all_components

    # 检查 LLM
    if "llm" in components:
        llm_health = await check_llm_health(timeout)
        results["checks"]["llm"] = llm_health
        if llm_health.status == HealthStatus.UNHEALTHY:
            results["status"] = HealthStatus.UNHEALTHY
        elif llm_health.status == HealthStatus.DEGRADED:
            results["status"] = HealthStatus.DEGRADED

    # 检查存储
    if "storage" in components:
        storage_health = await check_storage_health(timeout)
        results["checks"]["storage"] = storage_health

    # 检查 MCP
    if "mcp" in components:
        mcp_health = await check_mcp_health(timeout)
        results["checks"]["mcp"] = mcp_health

    # 检查环境
    if "environment" in components:
        env_health = check_environment_health()
        results["checks"]["environment"] = env_health

    return results

async def check_llm_health(timeout: float = 5.0) -> ComponentHealth:
    """检查 LLM 健康"""
    import time

    try:
        from ..providers import get_default_provider
        provider = get_default_provider()

        start = time.time()
        await provider.chat([Message.user("ping")], timeout=timeout)
        latency = (time.time() - start) * 1000

        return ComponentHealth(
            status=HealthStatus.HEALTHY,
            latency_ms=latency,
            details={
                "provider": provider.name,
                "model": provider.model,
            }
        )
    except Exception as e:
        return ComponentHealth(
            status=HealthStatus.UNHEALTHY,
            message=str(e),
            suggestion="Check API key and network connectivity"
        )

async def check_storage_health(timeout: float = 5.0) -> ComponentHealth:
    """检查存储健康"""
    try:
        from ..storage import get_default_storage
        storage = get_default_storage()

        # 测试读写
        test_key = "_health_check_test"
        await storage.write(test_key, "ok")
        value = await storage.read(test_key)
        await storage.delete(test_key)

        if value != b"ok":
            return ComponentHealth(
                status=HealthStatus.UNHEALTHY,
                message="Storage read/write mismatch"
            )

        return ComponentHealth(
            status=HealthStatus.HEALTHY,
            details={
                "backend": storage.__class__.__name__,
            }
        )
    except Exception as e:
        return ComponentHealth(
            status=HealthStatus.UNHEALTHY,
            message=str(e),
            suggestion="Check storage configuration and permissions"
        )

def check_environment_health() -> ComponentHealth:
    """检查环境"""
    import sys
    import platform

    return ComponentHealth(
        status=HealthStatus.HEALTHY,
        details={
            "python_version": sys.version,
            "platform": platform.platform(),
            "working_directory": os.getcwd(),
        }
    )
```

### 4.3 缺失的关键 API (新增)

```python
# src/continuum/events.py
"""事件系统"""

from typing import Callable, Any
from dataclasses import dataclass
from datetime import datetime

@dataclass
class Event:
    """事件"""
    type: str
    timestamp: datetime
    data: dict[str, Any]

class EventBus:
    """事件总线"""

    def __init__(self):
        self._handlers: dict[str, list[Callable]] = {}

    def on(self, event_type: str, handler: Callable) -> None:
        """注册事件处理器"""
        if event_type not in self._handlers:
            self._handlers[event_type] = []
        self._handlers[event_type].append(handler)

    def off(self, event_type: str, handler: Callable) -> None:
        """取消注册"""
        if event_type in self._handlers:
            self._handlers[event_type].remove(handler)

    async def emit(self, event: Event) -> None:
        """触发事件"""
        handlers = self._handlers.get(event.type, [])
        for handler in handlers:
            await handler(event)

# 内置事件类型
EVENT_TYPES = [
    "on_agent_start",
    "on_agent_end",
    "on_tool_call_start",
    "on_tool_call_end",
    "on_message",
    "on_error",
    "on_token",  # 流式输出
]
```

```python
# src/continuum/session.py
"""会话管理"""

from datetime import datetime
from pydantic import BaseModel

class Session(BaseModel):
    """会话"""
    id: str
    messages: list[Message]
    created_at: datetime
    updated_at: datetime
    metadata: dict = {}

class SessionManager:
    """会话管理器"""

    def __init__(self, storage: StorageBackend):
        self.storage = storage

    async def create_session(self) -> Session:
        """创建会话"""
        import uuid
        session = Session(
            id=str(uuid.uuid4()),
            messages=[],
            created_at=datetime.now(),
            updated_at=datetime.now()
        )
        await self.save_session(session)
        return session

    async def get_session(self, session_id: str) -> Session | None:
        """获取会话"""
        data = await self.storage.read(f"session:{session_id}")
        if data:
            return Session.model_validate_json(data)
        return None

    async def save_session(self, session: Session) -> None:
        """保存会话"""
        session.updated_at = datetime.now()
        await self.storage.write(
            f"session:{session.id}",
            session.model_dump_json()
        )

    async def delete_session(self, session_id: str) -> None:
        """删除会话"""
        await self.storage.delete(f"session:{session_id}")
```

```python
# src/continuum/streaming.py
"""流式输出"""

from typing import AsyncIterator

class StreamingAgent:
    """支持流式输出的 Agent"""

    async def run_stream(self, prompt: str) -> AsyncIterator[str]:
        """流式输出"""
        messages = [Message.user(prompt)]

        async for chunk in self.provider.chat_stream(messages):
            if "delta" in chunk and "content" in chunk["delta"]:
                yield chunk["delta"]["content"]

    async def run_with_events(
        self,
        prompt: str
    ) -> AsyncIterator[Event]:
        """带事件流的输出"""
        # 发送开始事件
        yield Event(type="on_agent_start", timestamp=datetime.now(), data={})

        messages = [Message.user(prompt)]

        async for chunk in self.provider.chat_stream(messages):
            if "delta" in chunk and "content" in chunk["delta"]:
                yield Event(
                    type="on_token",
                    timestamp=datetime.now(),
                    data={"content": chunk["delta"]["content"]}
                )

        # 发送结束事件
        yield Event(type="on_agent_end", timestamp=datetime.now(), data={})
```

---

## 五、性能优化规范

### 5.1 并行工具执行 (P1)

**评分**: 原方案 6/10 → 改进后 8/10

```python
# src/continuum/core/parallel.py
import asyncio
import re
from dataclasses import dataclass
from typing import Callable

@dataclass
class DependencyGraph:
    """工具依赖图"""
    nodes: dict[str, ToolCall]
    edges: set[tuple[str, str]]  # (from_id, to_id)

    def get_execution_layers(self) -> list[list[ToolCall]]:
        """
        拓扑排序，返回并行层级

        每层内的工具可以并行执行
        """
        in_degree = {n: 0 for n in self.nodes}
        for src, dst in self.edges:
            in_degree[dst] += 1

        layers = []
        remaining = set(self.nodes.keys())

        while remaining:
            # 当前层：入度为0的节点
            layer_ids = [n for n in remaining if in_degree[n] == 0]

            if not layer_ids:
                raise CycleDependencyError("检测到循环依赖")

            layer = [self.nodes[n] for n in layer_ids]
            layers.append(layer)

            # 移除当前层
            for node_id in layer_ids:
                remaining.remove(node_id)
                # 更新入度
                for src, dst in self.edges:
                    if src == node_id:
                        in_degree[dst] -= 1

        return layers


def analyze_dependencies(
    tool_calls: list[ToolCall],
    context: dict
) -> DependencyGraph:
    """分析工具间依赖关系"""
    graph = DependencyGraph(
        nodes={tc.id: tc for tc in tool_calls},
        edges=set()
    )

    # 1. 显式变量引用 (ReWOO 风格)
    var_pattern = re.compile(r'#E(\d+)')
    for i, tc in enumerate(tool_calls):
        args_str = tc.function.arguments
        for match in var_pattern.finditer(args_str):
            dep_idx = int(match.group(1)) - 1
            if 0 <= dep_idx < len(tool_calls):
                graph.edges.add((tool_calls[dep_idx].id, tc.id))

    # 2. 资源冲突检测
    file_ops = _extract_file_operations(tool_calls)
    for i, (tc1, files1) in enumerate(file_ops):
        for tc2, files2 in file_ops[i+1:]:
            # 如果两个工具操作相同文件，强制顺序
            if files1 & files2:
                graph.edges.add((tc1.id, tc2.id))

    return graph


async def execute_tools_parallel(
    tool_calls: list[ToolCall],
    executor: Callable[[ToolCall], Awaitable[ToolResult]],
    timeout: float = 30.0,
    max_concurrent: int = 10
) -> list[ToolResult]:
    """
    并行执行工具

    Args:
        tool_calls: 工具调用列表
        executor: 工具执行函数
        timeout: 单个工具超时
        max_concurrent: 最大并发数

    Returns:
        工具结果列表（与输入顺序一致）
    """
    if not tool_calls:
        return []

    # 分析依赖
    graph = analyze_dependencies(tool_calls, {})
    layers = graph.get_execution_layers()

    results: dict[str, ToolResult] = {}
    semaphore = asyncio.Semaphore(max_concurrent)

    async def run_with_limit(call: ToolCall) -> ToolResult:
        async with semaphore:
            return await executor(call)

    # 按层执行
    for layer in layers:
        # 当前层并行执行
        layer_results = await asyncio.gather(
            *[run_with_limit(call) for call in layer],
            return_exceptions=True
        )

        for call, result in zip(layer, layer_results):
            if isinstance(result, Exception):
                results[call.id] = ToolResult(
                    tool_call_id=call.id,
                    error=str(result)
                )
            else:
                results[call.id] = result

    # 按原始顺序返回结果
    return [results[tc.id] for tc in tool_calls]


class CycleDependencyError(Exception):
    pass
```

### 5.2 异步文件操作 (P2)

**评分**: 原方案 7/10 → 改进后 8/10

```python
# src/continuum/tools/file_ops.py
import anyio
from pathlib import Path

async def read_file(path: str) -> str:
    """异步读取文件"""
    return await anyio.Path(path).read_text(encoding='utf-8')

async def write_file(path: str, content: str) -> None:
    """异步写入文件"""
    file_path = anyio.Path(path)
    await file_path.parent.mkdir(parents=True, exist_ok=True)
    await file_path.write_text(content, encoding='utf-8')

async def read_large_file(
    path: str,
    chunk_size: int = 8192
) -> AsyncIterator[str]:
    """流式读取大文件"""
    async with await anyio.Path(path).open('r') as f:
        while chunk := await f.read(chunk_size):
            yield chunk
```

### 5.3 Repository Map 优化 (新增)

```python
# src/continuum/context/repository_map.py
import os
import ast
import json
import networkx as nx
from pathlib import Path
from datetime import datetime

class RepositoryMap:
    """
    基于 PageRank 的代码重要性分析

    优化：
    - 缓存持久化
    - 排除目录
    - 增量更新
    """

    EXCLUDE_DIRS = {
        '.git', '__pycache__', 'node_modules', 'venv', '.venv',
        'dist', 'build', '.egg', 'site-packages', '.mypy_cache'
    }

    def __init__(
        self,
        repo_root: Path,
        max_tokens: int = 4000,
        cache_dir: Path | None = None
    ):
        self.repo_root = repo_root
        self.max_tokens = max_tokens
        self.cache_dir = cache_dir or repo_root / ".continuum"
        self.cache_file = self.cache_dir / "repo_map_cache.json"

        self.graph = nx.DiGraph()
        self._import_cache: dict[str, tuple[float, list[str]]] = {}

    def build_dependency_graph(self, force_rebuild: bool = False):
        """构建依赖图，支持缓存"""
        if not force_rebuild and self._load_cache():
            return

        for py_file in self._iter_python_files():
            imports = self._extract_imports_cached(py_file)
            file_key = str(py_file.relative_to(self.repo_root))
            for imp in imports:
                self.graph.add_edge(file_key, imp, weight=1)

        self._save_cache()

    def _iter_python_files(self) -> Iterator[Path]:
        """高效遍历 Python 文件"""
        for root, dirs, files in os.walk(self.repo_root):
            # 原地排除目录
            dirs[:] = [d for d in dirs if d not in self.EXCLUDE_DIRS]
            for f in files:
                if f.endswith('.py'):
                    yield Path(root) / f

    def _extract_imports_cached(self, file_path: Path) -> list[str]:
        """带缓存的导入提取"""
        mtime = file_path.stat().st_mtime
        cache_key = str(file_path)

        if cache_key in self._import_cache:
            cached_mtime, imports = self._import_cache[cache_key]
            if cached_mtime == mtime:
                return imports

        imports = self._extract_imports(file_path)
        self._import_cache[cache_key] = (mtime, imports)
        return imports

    def _extract_imports(self, file_path: Path) -> list[str]:
        """提取文件导入"""
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            tree = ast.parse(content)

            imports = []
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imports.append(alias.name)
                elif isinstance(node, ast.ImportFrom) and node.module:
                    imports.append(node.module)

            return imports
        except Exception:
            return []

    def get_importance_ranking(self) -> dict[str, float]:
        """PageRank 排序"""
        if not self.graph.nodes:
            return {}
        return nx.pagerank(self.graph)

    def _load_cache(self) -> bool:
        """加载缓存"""
        if not self.cache_file.exists():
            return False

        try:
            with open(self.cache_file) as f:
                data = json.load(f)

            self._import_cache = {
                k: (v[0], v[1]) for k, v in data.get("import_cache", {}).items()
            }

            # 重建图
            for src, dst in data.get("edges", []):
                self.graph.add_edge(src, dst)

            return True
        except Exception:
            return False

    def _save_cache(self):
        """保存缓存"""
        self.cache_dir.mkdir(parents=True, exist_ok=True)

        data = {
            "timestamp": datetime.now().isoformat(),
            "import_cache": {
                k: [v[0], v[1]] for k, v in self._import_cache.items()
            },
            "edges": list(self.graph.edges())
        }

        with open(self.cache_file, 'w') as f:
            json.dump(data, f)
```

### 5.4 性能基准测试框架

```python
# tests/performance/benchmark.py
import pytest
import time
from statistics import mean, stdev
from dataclasses import dataclass

@dataclass
class BenchmarkResult:
    name: str
    count: int
    mean_ms: float
    stdev_ms: float
    min_ms: float
    max_ms: float
    p50_ms: float
    p99_ms: float

class PerformanceBenchmark:
    """性能基准测试"""

    def __init__(self):
        self.results: dict[str, list[float]] = {}

    def measure(self, name: str):
        """测量装饰器"""
        def decorator(func):
            async def wrapper(*args, **kwargs):
                start = time.perf_counter()
                result = await func(*args, **kwargs)
                elapsed = time.perf_counter() - start

                if name not in self.results:
                    self.results[name] = []
                self.results[name].append(elapsed)
                return result
            return wrapper
        return decorator

    def report(self) -> list[BenchmarkResult]:
        """生成报告"""
        reports = []
        for name, times in self.results.items():
            sorted_times = sorted(times)
            reports.append(BenchmarkResult(
                name=name,
                count=len(times),
                mean_ms=mean(times) * 1000,
                stdev_ms=stdev(times) * 1000 if len(times) > 1 else 0,
                min_ms=min(times) * 1000,
                max_ms=max(times) * 1000,
                p50_ms=sorted_times[len(times) // 2] * 1000,
                p99_ms=sorted_times[int(len(times) * 0.99)] * 1000,
            ))
        return reports

# 性能目标
PERFORMANCE_TARGETS = {
    "single_tool_execution": {"mean_ms": 100, "p99_ms": 500},
    "parallel_10_tools": {"mean_ms": 500, "p99_ms": 1000},
    "repo_map_10k_files": {"mean_ms": 5000, "p99_ms": 10000},
    "context_manager_add": {"mean_ms": 10, "p99_ms": 50},
}
```

---

## 六、测试规范

### 6.1 测试覆盖率目标

| 模块 | 目标覆盖率 | 优先级 |
|------|-----------|--------|
| core/context.py | 90% | P0 |
| core/guardrail.py | 85% | P0 |
| providers/base.py | 90% | P0 |
| storage/base.py | 85% | P0 |
| security/*.py | 85% | P0 |
| tools/shell.py | 90% | P0 |
| agents/handoff.py | 80% | P1 |
| mcp/client.py | 70% | P1 |

### 6.2 CI/CD 配置

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ['3.10', '3.11', '3.12']

    steps:
    - uses: actions/checkout@v4

    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v5
      with:
        python-version: ${{ matrix.python-version }}

    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install -e ".[dev]"

    - name: Run linting
      run: |
        ruff check .
        mypy src/

    - name: Run tests
      run: |
        pytest tests/ -v --cov=src/continuum --cov-report=xml

    - name: Upload coverage
      uses: codecov/codecov-action@v4
      with:
        files: ./coverage.xml

  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4

    - name: Run security scan
      run: |
        pip install bandit pip-audit
        bandit -r src/
        pip-audit
```

---

## 七、实施时间表 (修订)

### 总时间：8-10 周

| 阶段 | 时间 | 任务 | 交付物 |
|------|------|------|--------|
| **Phase 0** | Week 1 | 项目初始化 | 目录结构、依赖配置、核心类型、FakeLLMProvider |
| **Phase 1** | Week 2-3 | 核心模块 | LLMProvider、StorageBackend、ContextManager |
| **Phase 2** | Week 4-5 | 安全特性 | Shell工具、API Key管理、沙箱、MCP安全 |
| **Phase 3** | Week 6-7 | Agent 功能 | Memory、Hooks、Handoff、Guardrail |
| **Phase 4** | Week 8-9 | 高级特性 | 并行执行、Repository Map、事件系统 |
| **Phase 5** | Week 10 | 集成发布 | 文档、示例、CI/CD、测试覆盖 |

---

## 八、验收标准 (修订)

### 安全验收

- [ ] 所有 Shell 命令 **100% 禁用 `shell=True`**
- [ ] API Key 仅通过环境变量或 keyring 存储，代码审查确认无明文
- [ ] 文件操作 **100% 通过 safe_path() 验证**
- [ ] MCP 服务器仅允许预定义列表
- [ ] 通过 `pip-audit` 安全扫描，无已知漏洞
- [ ] 通过 `bandit` 静态分析，无高危告警

### 性能验收

- [ ] 单次 LLM 调用 P95 延迟 < 5s (不含网络)
- [ ] 并行 10 个工具调用，总时间 < 单个调用 × 1.5
- [ ] 内存峰值 < 500MB (10 轮对话 + 100 条消息)

### 质量验收

- [ ] 测试覆盖率 > 80%
- [ ] 所有 Provider 通过契约测试
- [ ] CI/CD 流程运行成功
- [ ] API 文档完整
- [ ] 至少 3 个完整示例

---

**文档状态**: v2.0 已完成 (整合两轮专家评审)
**下一步**: 开始 Phase 0 项目初始化
