# SuperHarness 生产级可靠性设计

> 版本: v1.0
> 日期: 2026-05-09
> 定位: 简洁可靠的Agent运行时可靠性保障
> 基于: TECHNOLOGY_PRODUCT_POLICY.md + 十四轮专家评审共识

---

## 一、设计原则与定位

### 1.1 核心定位

```
可靠性目标 = 高可用 × 容错性 × 可观测 × 安全性

具体目标:
- 可用性: 99.5% SLA (月停机<4小时)
- 容错性: 单点故障自动恢复, RTO<30分钟
- 可观测: 全链路追踪, 指标实时可见
- 安全性: API密钥隔离, 敏感数据脱敏
```

### 1.2 与MVP核心功能的关系

```
MVP核心功能: 会话无缝延续

可靠性保障:
├── 会话持久化: Checkpoint可靠保存
├── 崩溃恢复: 自动恢复到最新状态
├── API容错: LLM调用失败自动重试
└── 状态一致性: Checkpoint完整性校验
```

### 1.3 开源精神下的可靠性原则

```
原则1: 透明性 > 隐藏复杂性
- 所有错误可追溯
- 所有状态可视化
- 所有配置可审计

原则2: 用户可控 > 自动魔法
- 用户可手动触发Checkpoint
- 用户可配置重试策略
- 用户可设置安全限制

原则3: 渐进式复杂度
- 默认配置即可靠
- 高级配置可选
- 专家级完全自定义
```

---

## 二、容错机制设计

### 2.1 LLM API调用失败处理

#### 2.1.1 三层容错架构

```
┌─────────────────────────────────────────────────┐
│ Layer 3: 降级层                                  │
│ - 模型Fallback (GPT-4o → GPT-4o-mini)          │
│ - Provider Fallback (OpenAI → Anthropic)        │
│ - 本地模型备用                                   │
└─────────────────────────────────────────────────┘
           ▲ Layer 2失败后触发
┌─────────────────────────────────────────────────┐
│ Layer 2: 熔断层                                  │
│ - 连续失败5次触发熔断                            │
│ - 熔断时间: 30秒 → 60秒 → 120秒 (指数退避)       │
│- 半开状态探测                                    │
└─────────────────────────────────────────────────┘
           ▲ Layer 1失败后触发
┌─────────────────────────────────────────────────┐
│ Layer 1: 自动重试层                              │
│ - 网络错误: 自动重试3次, 指数退避                │
│ - Rate Limit: 等待后重试(429 Retry-After)      │
│ - Timeout: 延长超时后重试                        │
└─────────────────────────────────────────────────┘
```

#### 2.1.2 重试策略设计

```python
from dataclasses import dataclass
from typing import Optional, Callable
import asyncio

@dataclass
class RetryConfig:
    """重试配置"""
    max_retries: int = 3
    base_delay: float = 1.0          # 基础延迟(秒)
    max_delay: float = 60.0          # 最大延迟(秒)
    exponential_base: float = 2.0    # 指数基数

    # 可重试的错误类型
    retryable_errors: tuple = (
        "ConnectionError",
        "TimeoutError",
        "RateLimitError",
        "ServiceUnavailableError",
    )

class RetryExecutor:
    """重试执行器"""

    async def execute_with_retry(
        self,
        func: Callable,
        config: RetryConfig = None,
        on_retry: Optional[Callable] = None
    ) -> Any:
        """带重试的执行"""
        config = config or RetryConfig()
        last_error = None

        for attempt in range(config.max_retries + 1):
            try:
                return await func()
            except Exception as e:
                last_error = e

                # 判断是否可重试
                if not self._is_retryable(e, config):
                    raise

                # 最后一次尝试不等待
                if attempt == config.max_retries:
                    break

                # 计算延迟
                delay = min(
                    config.base_delay * (config.exponential_base ** attempt),
                    config.max_delay
                )

                # 回调通知
                if on_retry:
                    on_retry(attempt + 1, e, delay)

                # 等待后重试
                await asyncio.sleep(delay)

        # 所有重试失败
        raise LLMRetryExhaustedError(
            f"All {config.max_retries} retries failed"
        ) from last_error
```

#### 2.1.3 熔断器设计

```python
from enum import Enum
from datetime import datetime, timedelta
import threading

class CircuitState(Enum):
    CLOSED = "closed"        # 正常状态
    OPEN = "open"           # 熔断状态
    HALF_OPEN = "half_open" # 半开状态

class CircuitBreaker:
    """熔断器"""

    def __init__(
        self,
        failure_threshold: int = 5,
        recovery_timeout: int = 30,
        half_open_max_calls: int = 3
    ):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.half_open_max_calls = half_open_max_calls

        self.state = CircuitState.CLOSED
        self.failure_count = 0
        self.last_failure_time: Optional[datetime] = None
        self.half_open_calls = 0
        self._lock = threading.Lock()

    async def call(self, func: Callable) -> Any:
        """通过熔断器调用"""
        with self._lock:
            # 检查是否应该从OPEN转为HALF_OPEN
            if self.state == CircuitState.OPEN:
                if self._should_attempt_recovery():
                    self.state = CircuitState.HALF_OPEN
                    self.half_open_calls = 0
                else:
                    raise CircuitBreakerOpenError(
                        "Circuit breaker is open"
                    )

            # HALF_OPEN状态下限制调用次数
            if self.state == CircuitState.HALF_OPEN:
                if self.half_open_calls >= self.half_open_max_calls:
                    raise CircuitBreakerOpenError(
                        "Circuit breaker in half-open, max calls reached"
                    )
                self.half_open_calls += 1

        # 执行调用
        try:
            result = await func()
            self._on_success()
            return result
        except Exception as e:
            self._on_failure()
            raise

    def _should_attempt_recovery(self) -> bool:
        """是否应该尝试恢复"""
        if not self.last_failure_time:
            return False
        elapsed = datetime.now() - self.last_failure_time
        return elapsed > timedelta(seconds=self.recovery_timeout)

    def _on_success(self):
        """调用成功"""
        with self._lock:
            if self.state == CircuitState.HALF_OPEN:
                self.state = CircuitState.CLOSED
            self.failure_count = 0

    def _on_failure(self):
        """调用失败"""
        with self._lock:
            self.failure_count += 1
            self.last_failure_time = datetime.now()

            if self.state == CircuitState.HALF_OPEN:
                self.state = CircuitState.OPEN
            elif self.failure_count >= self.failure_threshold:
                self.state = CircuitState.OPEN
```

#### 2.1.4 模型Fallback链

```python
@dataclass
class ModelFallbackConfig:
    """模型降级配置"""
    primary_model: str = "gpt-4o"
    fallback_models: list = None  # ["gpt-4o-mini", "gpt-3.5-turbo"]

    def __post_init__(self):
        if self.fallback_models is None:
            self.fallback_models = ["gpt-4o-mini"]

class ModelFallbackProvider:
    """模型降级提供者"""

    def __init__(
        self,
        provider: LLMProvider,
        config: ModelFallbackConfig
    ):
        self.provider = provider
        self.config = config
        self.current_model_index = 0

    async def chat_with_fallback(
        self,
        messages: list[Message],
        **kwargs
    ) -> LLMResponse:
        """带降级的聊天"""
        models_to_try = [
            self.config.primary_model,
            *self.config.fallback_models
        ]

        last_error = None
        for model in models_to_try:
            try:
                return await self.provider.chat(
                    messages=messages,
                    model=model,
                    **kwargs
                )
            except Exception as e:
                last_error = e
                # 记录降级事件
                self._log_fallback(model, e)
                continue

        # 所有模型都失败
        raise AllModelsFailedError(
            f"All models failed: {models_to_try}"
        ) from last_error

    def _log_fallback(self, failed_model: str, error: Exception):
        """记录降级事件"""
        # 写入日志
        logger.warning(
            f"Model fallback: {failed_model} failed, trying next",
            extra={
                "failed_model": failed_model,
                "error_type": type(error).__name__,
                "error_message": str(error)
            }
        )
```

### 2.2 工具执行失败处理

#### 2.2.1 工具执行生命周期

```
工具执行流程:
1. 参数验证 → 失败返回错误信息给LLM
2. 权限检查 → 失败返回权限错误
3. 执行工具(超时控制) → 超时中断并返回超时信息
4. 结果验证 → 异常结果返回给LLM处理
5. 资源清理 → 确保无资源泄漏

失败处理:
├── 可恢复错误: 返回错误信息给LLM, 让LLM调整参数重试
├── 不可恢复错误: 终止执行, 保存Checkpoint
└── 资源耗尽: 隔离工具, 防止影响其他工具
```

#### 2.2.2 工具隔离机制

```python
import asyncio
from typing import Dict, Set
import resource

class ToolIsolationManager:
    """工具隔离管理器"""

    def __init__(self):
        self.failed_tools: Dict[str, int] = {}  # 工具名 -> 失败次数
        self.isolated_tools: Set[str] = set()
        self.failure_threshold = 3
        self.isolation_duration = 300  # 5分钟

    async def execute_with_isolation(
        self,
        tool_name: str,
        func: Callable,
        timeout: float = 30.0
    ) -> Any:
        """带隔离的执行"""
        # 检查工具是否被隔离
        if tool_name in self.isolated_tools:
            raise ToolIsolatedError(
                f"Tool {tool_name} is isolated due to repeated failures"
            )

        try:
            # 设置资源限制
            self._set_resource_limits()

            # 带超时执行
            result = await asyncio.wait_for(
                func(),
                timeout=timeout
            )

            # 成功后重置失败计数
            self.failed_tools.pop(tool_name, None)
            return result

        except asyncio.TimeoutError:
            self._record_failure(tool_name)
            raise ToolTimeoutError(
                f"Tool {tool_name} timed out after {timeout}s"
            )
        except Exception as e:
            self._record_failure(tool_name)
            raise
        finally:
            self._restore_resource_limits()

    def _record_failure(self, tool_name: str):
        """记录失败"""
        self.failed_tools[tool_name] = \
            self.failed_tools.get(tool_name, 0) + 1

        if self.failed_tools[tool_name] >= self.failure_threshold:
            self.isolated_tools.add(tool_name)
            # 5分钟后自动解除隔离
            asyncio.create_task(
                self._auto_unisolate(tool_name)
            )

    async def _auto_unisolate(self, tool_name: str):
        """自动解除隔离"""
        await asyncio.sleep(self.isolation_duration)
        self.isolated_tools.discard(tool_name)
        self.failed_tools.pop(tool_name, None)

    def _set_resource_limits(self):
        """设置资源限制"""
        # 限制内存使用(可选)
        # soft, hard = resource.getrlimit(resource.RLIMIT_AS)
        # resource.setrlimit(resource.RLIMIT_AS, (512 * 1024 * 1024, hard))
        pass

    def _restore_resource_limits(self):
        """恢复资源限制"""
        pass
```

#### 2.2.3 工具执行超时控制

```python
@dataclass
class ToolTimeoutConfig:
    """工具超时配置"""
    default_timeout: float = 30.0
    max_timeout: float = 300.0
    timeout_per_tool: Dict[str, float] = None

    def get_timeout(self, tool_name: str) -> float:
        """获取工具超时时间"""
        if self.timeout_per_tool and tool_name in self.timeout_per_tool:
            return min(
                self.timeout_per_tool[tool_name],
                self.max_timeout
            )
        return self.default_timeout

class TimeoutAwareToolExecutor:
    """超时感知的工具执行器"""

    def __init__(self, config: ToolTimeoutConfig):
        self.config = config
        self.isolation_manager = ToolIsolationManager()

    async def execute_tool(
        self,
        tool: Tool,
        arguments: dict
    ) -> ToolResult:
        """执行工具"""
        timeout = self.config.get_timeout(tool.name)

        # 验证参数
        try:
            validated_args = tool.validate_arguments(arguments)
        except ValidationError as e:
            return ToolResult(
                tool_call_id="",
                content=f"参数验证失败: {e}",
                is_error=True
            )

        # 执行工具(带隔离和超时)
        try:
            result = await self.isolation_manager.execute_with_isolation(
                tool_name=tool.name,
                func=lambda: tool.execute(validated_args),
                timeout=timeout
            )

            return ToolResult(
                tool_call_id="",
                content=json.dumps(result) if not isinstance(result, str) else result,
                is_error=False
            )

        except ToolTimeoutError as e:
            return ToolResult(
                tool_call_id="",
                content=f"工具执行超时({timeout}秒), 可能是任务过于复杂或参数不当。建议: 1)简化参数 2)分步执行 3)使用更快的工具",
                is_error=True
            )
        except ToolIsolatedError as e:
            return ToolResult(
                tool_call_id="",
                content=f"工具已被临时隔离(连续失败{self.isolation_manager.failure_threshold}次), 将在{self.isolation_manager.isolation_duration}秒后自动恢复。请尝试其他工具或等待。",
                is_error=True
            )
        except Exception as e:
            return ToolResult(
                tool_call_id="",
                content=f"工具执行失败: {str(e)}",
                is_error=True
            )
```

### 2.3 Checkpoint损坏处理

#### 2.3.1 Checkpoint完整性校验

```python
import hashlib
import json
from typing import Optional

@dataclass
class CheckpointMetadata:
    """Checkpoint元数据"""
    checksum: str              # SHA256校验和
    version: str               # 格式版本
    created_at: datetime
    size_bytes: int

class CheckpointValidator:
    """Checkpoint校验器"""

    VERSION = "1.0"

    @classmethod
    def validate_checkpoint(
        cls,
        checkpoint_path: str
    ) -> Tuple[bool, Optional[str]]:
        """校验Checkpoint完整性"""
        try:
            # 读取文件
            with open(checkpoint_path, 'r', encoding='utf-8') as f:
                data = json.load(f)

            # 1. 结构校验
            required_fields = [
                'session_id', 'created_at', 'messages',
                'iteration', 'tokens_used', 'cost_estimate'
            ]
            for field in required_fields:
                if field not in data:
                    return False, f"Missing required field: {field}"

            # 2. 数据类型校验
            if not isinstance(data['messages'], list):
                return False, "messages must be a list"
            if not isinstance(data['iteration'], int):
                return False, "iteration must be an integer"

            # 3. 逻辑一致性校验
            if data['iteration'] < 0:
                return False, "iteration cannot be negative"
            if data['tokens_used'] < 0:
                return False, "tokens_used cannot be negative"

            # 4. 消息格式校验
            for i, msg in enumerate(data['messages']):
                if 'role' not in msg or 'content' not in msg:
                    return False, f"Message {i} missing role or content"
                if msg['role'] not in ['system', 'user', 'assistant', 'tool']:
                    return False, f"Message {i} has invalid role: {msg['role']}"

            # 5. Checksum校验(如果有)
            if '_checksum' in data:
                expected_checksum = data.pop('_checksum')
                actual_checksum = cls._compute_checksum(data)
                if expected_checksum != actual_checksum:
                    return False, "Checksum mismatch - file may be corrupted"

            return True, None

        except json.JSONDecodeError as e:
            return False, f"JSON decode error: {e}"
        except Exception as e:
            return False, f"Validation error: {e}"

    @classmethod
    def _compute_checksum(cls, data: dict) -> str:
        """计算校验和"""
        canonical_json = json.dumps(data, sort_keys=True)
        return hashlib.sha256(canonical_json.encode()).hexdigest()

    @classmethod
    def add_checksum(cls, checkpoint: dict) -> dict:
        """添加校验和"""
        checkpoint_copy = checkpoint.copy()
        checksum = cls._compute_checksum(checkpoint_copy)
        checkpoint_copy['_checksum'] = checksum
        checkpoint_copy['_version'] = cls.VERSION
        return checkpoint_copy
```

#### 2.3.2 Checkpoint修复机制

```python
class CheckpointRepairer:
    """Checkpoint修复器"""

    @classmethod
    def repair_checkpoint(
        cls,
        checkpoint_path: str,
        backup_path: Optional[str] = None
    ) -> Tuple[bool, Optional[dict]]:
        """尝试修复损坏的Checkpoint"""
        # 1. 尝试读取
        try:
            with open(checkpoint_path, 'r', encoding='utf-8') as f:
                data = json.load(f)
        except json.JSONDecodeError:
            # JSON完全损坏, 尝试从备份恢复
            return cls._recover_from_backup(checkpoint_path, backup_path)

        # 2. 尝试修复结构
        repaired_data = data.copy()
        repairs_made = []

        # 修复缺失字段
        if 'iteration' not in repaired_data:
            repaired_data['iteration'] = len(repaired_data.get('messages', []))
            repairs_made.append("Added missing iteration field")

        if 'tokens_used' not in repaired_data:
            repaired_data['tokens_used'] = 0
            repairs_made.append("Added missing tokens_used field")

        if 'cost_estimate' not in repaired_data:
            repaired_data['cost_estimate'] = 0.0
            repairs_made.append("Added missing cost_estimate field")

        # 修复消息格式
        if 'messages' in repaired_data:
            valid_messages = []
            for msg in repaired_data['messages']:
                if cls._repair_message(msg):
                    valid_messages.append(msg)
            if len(valid_messages) != len(repaired_data['messages']):
                repairs_made.append(
                    f"Removed {len(repaired_data['messages']) - len(valid_messages)} invalid messages"
                )
                repaired_data['messages'] = valid_messages

        # 3. 验证修复后的数据
        is_valid, error = CheckpointValidator.validate_checkpoint_data(repaired_data)
        if not is_valid:
            logger.error(f"Repair failed: {error}")
            return False, None

        # 4. 记录修复日志
        if repairs_made:
            logger.info(f"Checkpoint repaired: {'; '.join(repairs_made)}")

        return True, repaired_data

    @classmethod
    def _repair_message(cls, msg: dict) -> bool:
        """尝试修复单条消息"""
        if not isinstance(msg, dict):
            return False
        if 'role' not in msg:
            return False
        if 'content' not in msg:
            msg['content'] = ''
        return True

    @classmethod
    def _recover_from_backup(
        cls,
        checkpoint_path: str,
        backup_path: Optional[str]
    ) -> Tuple[bool, Optional[dict]]:
        """从备份恢复"""
        if backup_path and os.path.exists(backup_path):
            try:
                with open(backup_path, 'r', encoding='utf-8') as f:
                    data = json.load(f)
                logger.info(f"Recovered checkpoint from backup: {backup_path}")
                return True, data
            except Exception as e:
                logger.error(f"Backup recovery failed: {e}")
        return False, None
```

#### 2.3.3 Checkpoint回滚机制

```python
class CheckpointRollbackManager:
    """Checkpoint回滚管理器"""

    def __init__(
        self,
        max_checkpoints: int = 50,
        storage_path: str = "~/.superharness/sessions"
    ):
        self.max_checkpoints = max_checkpoints
        self.storage_path = Path(storage_path).expanduser()

    def list_checkpoints(
        self,
        session_id: str
    ) -> List[Tuple[str, datetime, int]]:
        """列出所有可用Checkpoint"""
        session_dir = self.storage_path / session_id / "checkpoints"
        if not session_dir.exists():
            return []

        checkpoints = []
        for cp_file in sorted(session_dir.glob("cp_*.json")):
            try:
                stat = cp_file.stat()
                checkpoints.append((
                    str(cp_file),
                    datetime.fromtimestamp(stat.st_mtime),
                    stat.st_size
                ))
            except Exception:
                continue

        return sorted(checkpoints, key=lambda x: x[1], reverse=True)

    def rollback_to_checkpoint(
        self,
        session_id: str,
        checkpoint_id: Optional[str] = None
    ) -> Tuple[bool, Optional[dict]]:
        """回滚到指定Checkpoint"""
        checkpoints = self.list_checkpoints(session_id)
        if not checkpoints:
            return False, None

        # 如果没有指定, 回滚到最新的有效Checkpoint
        if checkpoint_id is None:
            for cp_path, cp_time, _ in checkpoints:
                is_valid, _ = CheckpointValidator.validate_checkpoint(cp_path)
                if is_valid:
                    checkpoint_id = cp_path
                    break

        if checkpoint_id is None:
            return False, None

        # 加载Checkpoint
        try:
            with open(checkpoint_id, 'r', encoding='utf-8') as f:
                data = json.load(f)
            logger.info(f"Rolled back to checkpoint: {checkpoint_id}")
            return True, data
        except Exception as e:
            logger.error(f"Failed to load checkpoint: {e}")
            return False, None

    def prune_old_checkpoints(self, session_id: str):
        """清理旧Checkpoint"""
        checkpoints = self.list_checkpoints(session_id)
        if len(checkpoints) > self.max_checkpoints:
            # 删除最旧的Checkpoint
            to_delete = checkpoints[self.max_checkpoints:]
            for cp_path, _, _ in to_delete:
                try:
                    os.remove(cp_path)
                    logger.debug(f"Pruned old checkpoint: {cp_path}")
                except Exception as e:
                    logger.warning(f"Failed to prune checkpoint: {e}")
```

---

## 三、可观测性系统设计

### 3.1 执行追踪系统

#### 3.1.1 追踪数据结构

```python
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional
from datetime import datetime
from enum import Enum
import uuid

class SpanKind(Enum):
    """Span类型"""
    LLM_CALL = "llm_call"
    TOOL_EXECUTION = "tool_execution"
    CHECKPOINT = "checkpoint"
    AGENT_ITERATION = "agent_iteration"
    MESSAGE_PROCESSING = "message_processing"

@dataclass
class Span:
    """单个追踪单元"""
    span_id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])
    trace_id: str = ""
    parent_span_id: Optional[str] = None
    kind: SpanKind = SpanKind.AGENT_ITERATION
    name: str = ""
    start_time: datetime = field(default_factory=datetime.now)
    end_time: Optional[datetime] = None
    duration_ms: Optional[float] = None

    # 输入输出
    input_data: Dict[str, Any] = field(default_factory=dict)
    output_data: Dict[str, Any] = field(default_factory=dict)

    # 元数据
    attributes: Dict[str, Any] = field(default_factory=dict)
    events: List[Dict[str, Any]] = field(default_factory=list)

    # 状态
    status: str = "OK"  # OK, ERROR, CANCELLED
    error_message: Optional[str] = None

@dataclass
class Trace:
    """完整追踪链"""
    trace_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    session_id: str = ""
    spans: List[Span] = field(default_factory=list)
    root_span_id: Optional[str] = None

    # 汇总信息
    total_duration_ms: float = 0.0
    total_tokens: int = 0
    total_cost: float = 0.0
    llm_calls: int = 0
    tool_calls: int = 0
```

#### 3.1.2 追踪收集器

```python
import threading
from contextlib import contextmanager
from typing import Optional

class TraceCollector:
    """追踪收集器"""

    _instance = None
    _lock = threading.Lock()

    def __new__(cls):
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    cls._instance = super().__new__(cls)
                    cls._instance._traces = {}
                    cls._instance._current_span = {}
        return cls._instance

    def start_trace(self, session_id: str) -> Trace:
        """开始新追踪"""
        trace = Trace(session_id=session_id)
        self._traces[trace.trace_id] = trace
        return trace

    @contextmanager
    def span(
        self,
        kind: SpanKind,
        name: str,
        trace_id: Optional[str] = None,
        parent_span_id: Optional[str] = None,
        **attributes
    ):
        """创建Span上下文管理器"""
        span = Span(
            kind=kind,
            name=name,
            parent_span_id=parent_span_id,
            attributes=attributes
        )

        # 设置trace_id
        if trace_id:
            span.trace_id = trace_id
            if trace_id in self._traces:
                self._traces[trace_id].spans.append(span)
                if self._traces[trace_id].root_span_id is None:
                    self._traces[trace_id].root_span_id = span.span_id

        # 记录当前span
        thread_id = threading.get_ident()
        self._current_span[thread_id] = span

        try:
            yield span
        finally:
            # 结束span
            span.end_time = datetime.now()
            span.duration_ms = (
                span.end_time - span.start_time
            ).total_seconds() * 1000

            # 清理当前span
            self._current_span.pop(thread_id, None)

    def add_event(
        self,
        name: str,
        attributes: Optional[Dict] = None
    ):
        """添加事件到当前span"""
        thread_id = threading.get_ident()
        span = self._current_span.get(thread_id)
        if span:
            span.events.append({
                "name": name,
                "timestamp": datetime.now().isoformat(),
                "attributes": attributes or {}
            })

    def get_trace(self, trace_id: str) -> Optional[Trace]:
        """获取追踪"""
        return self._traces.get(trace_id)

    def export_trace(self, trace_id: str) -> Dict:
        """导出追踪数据"""
        trace = self.get_trace(trace_id)
        if not trace:
            return {}

        return {
            "trace_id": trace.trace_id,
            "session_id": trace.session_id,
            "spans": [
                {
                    "span_id": s.span_id,
                    "kind": s.kind.value,
                    "name": s.name,
                    "start_time": s.start_time.isoformat(),
                    "end_time": s.end_time.isoformat() if s.end_time else None,
                    "duration_ms": s.duration_ms,
                    "input": s.input_data,
                    "output": s.output_data,
                    "attributes": s.attributes,
                    "events": s.events,
                    "status": s.status,
                    "error": s.error_message
                }
                for s in trace.spans
            ],
            "summary": {
                "total_duration_ms": trace.total_duration_ms,
                "total_tokens": trace.total_tokens,
                "total_cost": trace.total_cost,
                "llm_calls": trace.llm_calls,
                "tool_calls": trace.tool_calls
            }
        }
```

#### 3.1.3 控制台追踪输出

```python
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.live import Live

class ConsoleTracer:
    """控制台追踪输出"""

    def __init__(self):
        self.console = Console()

    def trace_llm_call(
        self,
        model: str,
        prompt_tokens: int,
        completion_tokens: int,
        duration_ms: float,
        cost: float
    ):
        """追踪LLM调用"""
        self.console.print(
            f"[dim]LLM调用[/dim] {model} | "
            f"输入: {prompt_tokens} | 输出: {completion_tokens} | "
            f"耗时: {duration_ms:.0f}ms | 成本: ${cost:.4f}"
        )

    def trace_tool_call(
        self,
        tool_name: str,
        arguments: dict,
        result_preview: str,
        duration_ms: float,
        is_error: bool = False
    ):
        """追踪工具调用"""
        status = "[red]失败[/red]" if is_error else "[green]成功[/green]"
        args_str = json.dumps(arguments, ensure_ascii=False)[:50]
        result_str = result_preview[:100] if result_preview else "无输出"

        self.console.print(
            f"[dim]工具调用[/dim] {tool_name}({args_str}...) | "
            f"结果: {result_str}... | "
            f"耗时: {duration_ms:.0f}ms | {status}"
        )

    def trace_checkpoint(
        self,
        checkpoint_id: str,
        iteration: int,
        tokens_used: int,
        trigger: str
    ):
        """追踪Checkpoint"""
        self.console.print(
            f"[dim]Checkpoint[/dim] #{iteration} | "
            f"ID: {checkpoint_id[:8]} | "
            f"累计Token: {tokens_used} | "
            f"触发: {trigger}"
        )

    def show_summary(self, trace: Trace):
        """显示追踪摘要"""
        table = Table(title="执行摘要")
        table.add_column("指标", style="cyan")
        table.add_column("值", style="green")

        table.add_row("总耗时", f"{trace.total_duration_ms:.0f}ms")
        table.add_row("总Token", str(trace.total_tokens))
        table.add_row("总成本", f"${trace.total_cost:.4f}")
        table.add_row("LLM调用次数", str(trace.llm_calls))
        table.add_row("工具调用次数", str(trace.tool_calls))

        self.console.print(table)

    def show_timeline(self, trace: Trace):
        """显示时间线"""
        timeline = []
        for span in sorted(trace.spans, key=lambda s: s.start_time):
            indent = "  " * (1 if span.parent_span_id else 0)
            timeline.append(
                f"{indent}[{span.kind.value}] {span.name} "
                f"({span.duration_ms:.0f}ms)"
            )

        self.console.print(Panel(
            "\n".join(timeline),
            title="执行时间线",
            expand=False
        ))
```

### 3.2 性能指标系统

#### 3.2.1 指标定义

```python
from dataclasses import dataclass
from typing import Dict, List
from collections import defaultdict
import statistics

@dataclass
class MetricPoint:
    """指标数据点"""
    timestamp: datetime
    value: float
    labels: Dict[str, str]

class MetricsStore:
    """指标存储"""

    def __init__(self):
        self._metrics: Dict[str, List[MetricPoint]] = defaultdict(list)
        self._counters: Dict[str, float] = defaultdict(float)
        self._histograms: Dict[str, List[float]] = defaultdict(list)

    def record_latency(
        self,
        name: str,
        value: float,
        labels: Optional[Dict] = None
    ):
        """记录延迟"""
        self._metrics[name].append(MetricPoint(
            timestamp=datetime.now(),
            value=value,
            labels=labels or {}
        ))
        self._histograms[name].append(value)

    def increment_counter(
        self,
        name: str,
        value: float = 1.0,
        labels: Optional[Dict] = None
    ):
        """增加计数器"""
        self._counters[name] += value
        self._metrics[name].append(MetricPoint(
            timestamp=datetime.now(),
            value=self._counters[name],
            labels=labels or {}
        ))

    def get_percentile(
        self,
        name: str,
        percentile: float = 95.0
    ) -> Optional[float]:
        """获取百分位数"""
        values = self._histograms.get(name, [])
        if not values:
            return None
        return statistics.quantiles(values, n=100)[int(percentile) - 1]

    def get_summary(self) -> Dict:
        """获取指标摘要"""
        summary = {}

        # 计数器
        for name, value in self._counters.items():
            summary[name] = value

        # 直方图统计
        for name, values in self._histograms.items():
            if values:
                summary[f"{name}_p50"] = statistics.median(values)
                summary[f"{name}_p95"] = self.get_percentile(name, 95)
                summary[f"{name}_p99"] = self.get_percentile(name, 99)
                summary[f"{name}_avg"] = statistics.mean(values)
                summary[f"{name}_max"] = max(values)
                summary[f"{name}_min"] = min(values)

        return summary
```

#### 3.2.2 关键指标定义

```python
class AgentMetrics:
    """Agent关键指标"""

    # 延迟指标
    LLM_CALL_LATENCY = "llm_call_latency_ms"
    TOOL_EXECUTION_LATENCY = "tool_execution_latency_ms"
    CHECKPOINT_SAVE_LATENCY = "checkpoint_save_latency_ms"
    END_TO_END_LATENCY = "end_to_end_latency_ms"

    # Token指标
    TOKENS_PROMPT = "tokens_prompt"
    TOKENS_COMPLETION = "tokens_completion"
    TOKENS_TOTAL = "tokens_total"

    # 成本指标
    COST_PER_CALL = "cost_per_call"
    COST_TOTAL = "cost_total"

    # 错误指标
    LLM_ERRORS = "llm_errors"
    TOOL_ERRORS = "tool_errors"
    TIMEOUT_ERRORS = "timeout_errors"

    # 成功率指标
    LLM_SUCCESS_RATE = "llm_success_rate"
    TOOL_SUCCESS_RATE = "tool_success_rate"

    # 并发指标
    CONCURRENT_REQUESTS = "concurrent_requests"
    ACTIVE_SESSIONS = "active_sessions"

class MetricsCollector:
    """指标收集器"""

    def __init__(self):
        self.store = MetricsStore()

    def record_llm_call(
        self,
        model: str,
        latency_ms: float,
        tokens_prompt: int,
        tokens_completion: int,
        cost: float,
        success: bool
    ):
        """记录LLM调用"""
        labels = {"model": model}

        self.store.record_latency(
            AgentMetrics.LLM_CALL_LATENCY,
            latency_ms,
            labels
        )

        self.store.increment_counter(
            AgentMetrics.TOKENS_PROMPT,
            tokens_prompt,
            labels
        )

        self.store.increment_counter(
            AgentMetrics.TOKENS_COMPLETION,
            tokens_completion,
            labels
        )

        self.store.increment_counter(
            AgentMetrics.COST_TOTAL,
            cost,
            labels
        )

        if not success:
            self.store.increment_counter(
                AgentMetrics.LLM_ERRORS,
                1,
                labels
            )

    def record_tool_call(
        self,
        tool_name: str,
        latency_ms: float,
        success: bool
    ):
        """记录工具调用"""
        labels = {"tool": tool_name}

        self.store.record_latency(
            AgentMetrics.TOOL_EXECUTION_LATENCY,
            latency_ms,
            labels
        )

        if not success:
            self.store.increment_counter(
                AgentMetrics.TOOL_ERRORS,
                1,
                labels
            )

    def get_dashboard_data(self) -> Dict:
        """获取Dashboard数据"""
        summary = self.store.get_summary()

        return {
            "timestamp": datetime.now().isoformat(),
            "metrics": {
                "llm": {
                    "total_calls": summary.get("llm_errors", 0) + summary.get("llm_success", 0),
                    "avg_latency_p50": summary.get(f"{AgentMetrics.LLM_CALL_LATENCY}_p50"),
                    "avg_latency_p95": summary.get(f"{AgentMetrics.LLM_CALL_LATENCY}_p95"),
                    "error_count": summary.get(AgentMetrics.LLM_ERRORS, 0),
                },
                "tokens": {
                    "total_prompt": summary.get(AgentMetrics.TOKENS_PROMPT, 0),
                    "total_completion": summary.get(AgentMetrics.TOKENS_COMPLETION, 0),
                    "total": summary.get(AgentMetrics.TOKENS_TOTAL, 0),
                },
                "cost": {
                    "total": summary.get(AgentMetrics.COST_TOTAL, 0),
                },
                "tools": {
                    "total_calls": summary.get("tool_errors", 0) + summary.get("tool_success", 0),
                    "avg_latency_p50": summary.get(f"{AgentMetrics.TOOL_EXECUTION_LATENCY}_p50"),
                    "error_count": summary.get(AgentMetrics.TOOL_ERRORS, 0),
                }
            }
        }
```

### 3.3 成本追踪系统

#### 3.3.1 实时成本追踪

```python
@dataclass
class CostTracker:
    """成本追踪器"""

    session_id: str
    model: str

    # 累计数据
    total_tokens_prompt: int = 0
    total_tokens_completion: int = 0
    total_cost: float = 0.0

    # 模型定价(每1k tokens)
    PRICING = {
        "gpt-4o": {"prompt": 0.0025, "completion": 0.01},
        "gpt-4o-mini": {"prompt": 0.00015, "completion": 0.0006},
        "gpt-4-turbo": {"prompt": 0.01, "completion": 0.03},
        "gpt-3.5-turbo": {"prompt": 0.0005, "completion": 0.0015},
        "claude-3-opus": {"prompt": 0.015, "completion": 0.075},
        "claude-3-sonnet": {"prompt": 0.003, "completion": 0.015},
    }

    def record_usage(
        self,
        tokens_prompt: int,
        tokens_completion: int
    ):
        """记录使用量"""
        self.total_tokens_prompt += tokens_prompt
        self.total_tokens_completion += tokens_completion

        # 计算成本
        pricing = self.PRICING.get(self.model, {"prompt": 0.001, "completion": 0.002})
        cost = (
            (tokens_prompt / 1000) * pricing["prompt"] +
            (tokens_completion / 1000) * pricing["completion"]
        )
        self.total_cost += cost

        return cost

    def estimate_next_call(
        self,
        estimated_prompt: int,
        estimated_completion: int
    ) -> Tuple[float, float]:
        """估算下次调用成本"""
        pricing = self.PRICING.get(self.model, {"prompt": 0.001, "completion": 0.002})

        min_cost = (estimated_prompt / 1000) * pricing["prompt"]
        max_cost = (
            (estimated_prompt / 1000) * pricing["prompt"] +
            (estimated_completion / 1000) * pricing["completion"]
        )

        return min_cost, max_cost

    def get_report(self) -> Dict:
        """获取成本报告"""
        return {
            "session_id": self.session_id,
            "model": self.model,
            "tokens": {
                "prompt": self.total_tokens_prompt,
                "completion": self.total_tokens_completion,
                "total": self.total_tokens_prompt + self.total_tokens_completion
            },
            "cost": {
                "total_usd": round(self.total_cost, 4),
                "per_1k_tokens": round(
                    self.total_cost / ((self.total_tokens_prompt + self.total_tokens_completion) / 1000),
                    4
                ) if self.total_tokens_prompt + self.total_tokens_completion > 0 else 0
            }
        }
```

#### 3.3.2 成本预测

```python
class CostPredictor:
    """成本预测器"""

    def __init__(self, history_file: str = "~/.superharness/cost_history.json"):
        self.history_file = Path(history_file).expanduser()
        self.history = self._load_history()

    def predict_task_cost(
        self,
        task_type: str,
        model: str
    ) -> Dict:
        """预测任务成本"""
        # 基于历史数据预测
        similar_tasks = [
            h for h in self.history
            if h.get("task_type") == task_type and h.get("model") == model
        ]

        if len(similar_tasks) < 3:
            # 数据不足, 返回默认估算
            return {
                "min_cost": 0.01,
                "max_cost": 0.5,
                "avg_cost": 0.1,
                "confidence": "low",
                "message": "历史数据不足, 使用保守估算"
            }

        costs = [t["cost"] for t in similar_tasks]

        return {
            "min_cost": min(costs),
            "max_cost": max(costs),
            "avg_cost": statistics.mean(costs),
            "median_cost": statistics.median(costs),
            "confidence": "high" if len(costs) >= 10 else "medium",
            "sample_size": len(costs),
            "message": f"基于最近{len(costs)}次类似任务"
        }

    def estimate_remaining_budget(
        self,
        monthly_budget: float,
        days_remaining: int
    ) -> Dict:
        """估算剩余预算"""
        # 本月已用
        month_start = datetime.now().replace(day=1, hour=0, minute=0, second=0)
        used_this_month = sum(
            h["cost"] for h in self.history
            if datetime.fromisoformat(h["timestamp"]) >= month_start
        )

        remaining = monthly_budget - used_this_month
        daily_budget = remaining / days_remaining if days_remaining > 0 else 0

        return {
            "monthly_budget": monthly_budget,
            "used_this_month": round(used_this_month, 2),
            "remaining": round(remaining, 2),
            "days_remaining": days_remaining,
            "daily_budget": round(daily_budget, 2),
            "budget_health": "good" if remaining > monthly_budget * 0.2 else "warning"
        }

    def _load_history(self) -> List[Dict]:
        """加载历史数据"""
        if not self.history_file.exists():
            return []
        try:
            with open(self.history_file, 'r') as f:
                return json.load(f)
        except Exception:
            return []

    def save_to_history(self, record: Dict):
        """保存到历史"""
        self.history.append({
            **record,
            "timestamp": datetime.now().isoformat()
        })

        # 只保留最近1000条
        if len(self.history) > 1000:
            self.history = self.history[-1000:]

        with open(self.history_file, 'w') as f:
            json.dump(self.history, f, indent=2)
```

---

## 四、安全性机制设计

### 4.1 API密钥管理

#### 4.1.1 密钥存储策略

```python
import os
import keyring
from typing import Optional
from pathlib import Path

class APIKeyManager:
    """API密钥管理器"""

    SERVICE_NAME = "superharness"

    @classmethod
    def get_key(cls, provider: str) -> Optional[str]:
        """获取API密钥"""
        # 1. 优先从环境变量读取
        env_key = os.environ.get(f"{provider.upper()}_API_KEY")
        if env_key:
            return env_key

        # 2. 从配置文件读取
        config_key = cls._read_from_config(provider)
        if config_key:
            return config_key

        # 3. 从系统密钥环读取
        try:
            return keyring.get_password(cls.SERVICE_NAME, provider)
        except Exception:
            return None

    @classmethod
    def set_key(cls, provider: str, key: str, store_in_keyring: bool = False):
        """设置API密钥"""
        if store_in_keyring:
            try:
                keyring.set_password(cls.SERVICE_NAME, provider, key)
            except Exception as e:
                logger.warning(f"Failed to store in keyring: {e}")

        # 同时保存到配置文件(加密)
        cls._save_to_config(provider, key)

    @classmethod
    def _read_from_config(cls, provider: str) -> Optional[str]:
        """从配置文件读取"""
        config_path = Path("~/.superharness/keys.json").expanduser()
        if not config_path.exists():
            return None

        try:
            with open(config_path, 'r') as f:
                keys = json.load(f)
            return keys.get(provider)
        except Exception:
            return None

    @classmethod
    def _save_to_config(cls, provider: str, key: str):
        """保存到配置文件"""
        config_path = Path("~/.superharness/keys.json").expanduser()
        config_path.parent.mkdir(parents=True, exist_ok=True)

        # 限制文件权限
        try:
            keys = {}
            if config_path.exists():
                with open(config_path, 'r') as f:
                    keys = json.load(f)

            keys[provider] = key

            with open(config_path, 'w') as f:
                json.dump(keys, f)

            # Unix系统设置权限为600
            if os.name != 'nt':
                os.chmod(config_path, 0o600)
        except Exception as e:
            logger.error(f"Failed to save key to config: {e}")

    @classmethod
    def validate_key(cls, provider: str, key: str) -> bool:
        """验证密钥格式"""
        if provider == "openai":
            return key.startswith("sk-") and len(key) >= 40
        elif provider == "anthropic":
            return key.startswith("sk-ant-") and len(key) >= 40
        # 其他provider...
        return len(key) >= 20
```

#### 4.1.2 密钥轮换

```python
class APIKeyRotator:
    """API密钥轮换器"""

    def __init__(self):
        self.keys: Dict[str, List[str]] = {}
        self.current_index: Dict[str, int] = {}

    def add_key_pool(self, provider: str, keys: List[str]):
        """添加密钥池"""
        self.keys[provider] = keys
        self.current_index[provider] = 0

    def get_next_key(self, provider: str) -> str:
        """获取下一个密钥(轮询)"""
        if provider not in self.keys or not self.keys[provider]:
            raise ValueError(f"No keys available for {provider}")

        keys = self.keys[provider]
        idx = self.current_index[provider]
        key = keys[idx]

        # 移动到下一个
        self.current_index[provider] = (idx + 1) % len(keys)

        return key

    def mark_key_failed(self, provider: str, key: str):
        """标记密钥失败"""
        if provider in self.keys:
            try:
                self.keys[provider].remove(key)
                if not self.keys[provider]:
                    logger.error(f"All keys failed for {provider}")
            except ValueError:
                pass
```

### 4.2 工具执行沙箱

#### 4.2.1 沙箱隔离策略

```python
import subprocess
import tempfile
import shutil
from typing import Set, Optional

class ToolSandbox:
    """工具执行沙箱"""

    def __init__(
        self,
        allowed_paths: Optional[Set[str]] = None,
        denied_paths: Optional[Set[str]] = None,
        read_only: bool = False,
        network_allowed: bool = True
    ):
        self.allowed_paths = allowed_paths or set()
        self.denied_paths = denied_paths or set()
        self.read_only = read_only
        self.network_allowed = network_allowed

    def validate_path_access(self, path: str, write: bool = False) -> Tuple[bool, str]:
        """验证路径访问权限"""
        abs_path = os.path.abspath(path)

        # 检查拒绝路径
        for denied in self.denied_paths:
            if abs_path.startswith(denied):
                return False, f"Path {path} is denied"

        # 检查允许路径
        if self.allowed_paths:
            allowed = False
            for allowed_path in self.allowed_paths:
                if abs_path.startswith(allowed_path):
                    allowed = True
                    break
            if not allowed:
                return False, f"Path {path} is not in allowed paths"

        # 检查只读
        if write and self.read_only:
            return False, "Sandbox is read-only"

        return True, ""

    def create_temp_workspace(self) -> str:
        """创建临时工作空间"""
        temp_dir = tempfile.mkdtemp(prefix="superharness_sandbox_")
        self.allowed_paths.add(temp_dir)
        return temp_dir

    def cleanup_workspace(self, path: str):
        """清理工作空间"""
        if path in self.allowed_paths:
            self.allowed_paths.remove(path)
        if os.path.exists(path):
            shutil.rmtree(path)

class SandboxedToolExecutor:
    """沙箱工具执行器"""

    def __init__(self, sandbox: ToolSandbox):
        self.sandbox = sandbox

    async def execute_file_tool(
        self,
        tool_name: str,
        file_path: str,
        operation: str,
        **kwargs
    ) -> ToolResult:
        """执行文件相关工具"""
        # 验证路径
        write_ops = ["write", "delete", "move", "copy"]
        needs_write = operation in write_ops

        is_valid, error = self.sandbox.validate_path_access(
            file_path,
            write=needs_write
        )

        if not is_valid:
            return ToolResult(
                tool_call_id="",
                content=f"权限错误: {error}",
                is_error=True
            )

        # 执行工具
        # ...
```

#### 4.2.2 资源限制

```python
import resource
import signal
from typing import Optional

class ResourceLimiter:
    """资源限制器"""

    def __init__(
        self,
        max_memory_mb: int = 512,
        max_cpu_seconds: int = 60,
        max_file_size_mb: int = 100
    ):
        self.max_memory_mb = max_memory_mb
        self.max_cpu_seconds = max_cpu_seconds
        self.max_file_size_mb = max_file_size_mb

    def apply_limits(self):
        """应用资源限制"""
        # 内存限制
        if self.max_memory_mb:
            soft, hard = resource.getrlimit(resource.RLIMIT_AS)
            resource.setrlimit(
                resource.RLIMIT_AS,
                (self.max_memory_mb * 1024 * 1024, hard)
            )

        # CPU时间限制
        if self.max_cpu_seconds:
            resource.setrlimit(
                resource.RLIMIT_CPU,
                (self.max_cpu_seconds, self.max_cpu_seconds + 1)
            )

        # 文件大小限制
        if self.max_file_size_mb:
            resource.setrlimit(
                resource.RLIMIT_FSIZE,
                (self.max_file_size_mb * 1024 * 1024, self.max_file_size_mb * 1024 * 1024)
            )

    @staticmethod
    def check_memory_usage() -> Dict:
        """检查内存使用"""
        import psutil
        process = psutil.Process()

        return {
            "memory_mb": process.memory_info().rss / 1024 / 1024,
            "memory_percent": process.memory_percent(),
            "cpu_percent": process.cpu_percent()
        }
```

### 4.3 敏感数据脱敏

#### 4.3.1 脱敏规则

```python
import re
from typing import List, Tuple

class DataSanitizer:
    """数据脱敏器"""

    # 敏感数据模式
    SENSITIVE_PATTERNS = [
        # API密钥
        (r'sk-[a-zA-Z0-9]{20,}', 'sk-***REDACTED***'),
        (r'sk-ant-[a-zA-Z0-9]{20,}', 'sk-ant-***REDACTED***'),

        # 密码
        (r'password["\s]*[:=]["\s]*[^\s"\']+', 'password=***REDACTED***'),
        (r'passwd["\s]*[:=]["\s]*[^\s"\']+', 'passwd=***REDACTED***'),

        # Token
        (r'token["\s]*[:=]["\s]*[a-zA-Z0-9_-]{20,}', 'token=***REDACTED***'),

        # 邮箱
        (r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}', '***@***.***'),

        # 手机号
        (r'1[3-9]\d{9}', '1**********'),

        # 身份证
        (r'\d{17}[\dXx]', '******************'),

        # 信用卡
        (r'\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}', '****-****-****-****'),
    ]

    @classmethod
    def sanitize(cls, text: str) -> str:
        """脱敏文本"""
        sanitized = text
        for pattern, replacement in cls.SENSITIVE_PATTERNS:
            sanitized = re.sub(pattern, replacement, sanitized, flags=re.IGNORECASE)
        return sanitized

    @classmethod
    def sanitize_dict(cls, data: dict) -> dict:
        """脱敏字典"""
        result = {}
        for key, value in data.items():
            if isinstance(value, str):
                result[key] = cls.sanitize(value)
            elif isinstance(value, dict):
                result[key] = cls.sanitize_dict(value)
            elif isinstance(value, list):
                result[key] = [
                    cls.sanitize(item) if isinstance(item, str) else item
                    for item in value
                ]
            else:
                result[key] = value
        return result

    @classmethod
    def sanitize_message(cls, message: dict) -> dict:
        """脱敏消息"""
        sanitized = message.copy()
        if 'content' in sanitized and isinstance(sanitized['content'], str):
            sanitized['content'] = cls.sanitize(sanitized['content'])
        return sanitized

    @classmethod
    def sanitize_messages(cls, messages: List[dict]) -> List[dict]:
        """脱敏消息列表"""
        return [cls.sanitize_message(msg) for msg in messages]
```

#### 4.3.2 日志脱敏

```python
import logging

class SanitizedLogger:
    """脱敏日志器"""

    def __init__(self, name: str):
        self.logger = logging.getLogger(name)
        self.sanitizer = DataSanitizer()

    def info(self, msg: str, *args, **kwargs):
        """INFO日志(脱敏)"""
        sanitized_msg = self.sanitizer.sanitize(msg)
        self.logger.info(sanitized_msg, *args, **kwargs)

    def warning(self, msg: str, *args, **kwargs):
        """WARNING日志(脱敏)"""
        sanitized_msg = self.sanitizer.sanitize(msg)
        self.logger.warning(sanitized_msg, *args, **kwargs)

    def error(self, msg: str, *args, **kwargs):
        """ERROR日志(脱敏)"""
        sanitized_msg = self.sanitizer.sanitize(msg)
        self.logger.error(sanitized_msg, *args, **kwargs)

    def debug(self, msg: str, *args, **kwargs):
        """DEBUG日志(脱敏)"""
        sanitized_msg = self.sanitizer.sanitize(msg)
        self.logger.debug(sanitized_msg, *args, **kwargs)

    def log_llm_request(
        self,
        messages: List[dict],
        model: str
    ):
        """记录LLM请求(脱敏)"""
        sanitized_messages = self.sanitizer.sanitize_messages(messages)
        self.debug(
            f"LLM Request to {model}: {json.dumps(sanitized_messages, ensure_ascii=False)}"
        )

    def log_tool_call(
        self,
        tool_name: str,
        arguments: dict,
        result: Any
    ):
        """记录工具调用(脱敏)"""
        sanitized_args = self.sanitizer.sanitize_dict(arguments)
        sanitized_result = self.sanitizer.sanitize(str(result)) if result else None

        self.debug(
            f"Tool Call: {tool_name}({json.dumps(sanitized_args, ensure_ascii=False)}) -> {sanitized_result}"
        )
```

---

## 五、高可用机制设计

### 5.1 会话持久化

#### 5.1.1 自动Checkpoint保存

```python
class SessionPersistence:
    """会话持久化管理器"""

    def __init__(
        self,
        storage_path: str = "~/.superharness/sessions",
        auto_save_interval: int = 5,  # 每5轮保存一次
        max_checkpoints: int = 50
    ):
        self.storage_path = Path(storage_path).expanduser()
        self.auto_save_interval = auto_save_interval
        self.max_checkpoints = max_checkpoints

        self.storage_path.mkdir(parents=True, exist_ok=True)

    def auto_checkpoint(
        self,
        context: ExecutionContext,
        trigger: str = "periodic"
    ) -> Checkpoint:
        """自动创建Checkpoint"""
        # 检查是否需要保存
        if trigger == "periodic":
            if context.iteration % self.auto_save_interval != 0:
                return None

        # 创建Checkpoint
        checkpoint = context.to_checkpoint()
        checkpoint.trigger = trigger

        # 保存到文件
        self._save_checkpoint(checkpoint)

        # 清理旧Checkpoint
        self._prune_old_checkpoints(context.session_id)

        return checkpoint

    def _save_checkpoint(self, checkpoint: Checkpoint):
        """保存Checkpoint"""
        session_dir = self.storage_path / checkpoint.session_id / "checkpoints"
        session_dir.mkdir(parents=True, exist_ok=True)

        # 文件名包含时间戳和触发原因
        timestamp = checkpoint.created_at.strftime("%Y%m%d_%H%M%S")
        filename = f"cp_{timestamp}_{checkpoint.checkpoint_id[:8]}.json"
        filepath = session_dir / filename

        # 序列化并保存
        data = {
            "checkpoint_id": checkpoint.checkpoint_id,
            "session_id": checkpoint.session_id,
            "created_at": checkpoint.created_at.isoformat(),
            "trigger": checkpoint.trigger,
            "agent_state": checkpoint.agent_state.value,
            "iteration": checkpoint.iteration,
            "messages": checkpoint.messages,
            "tool_calls_pending": checkpoint.tool_calls_pending,
            "tool_results": checkpoint.tool_results,
            "tokens_used": checkpoint.tokens_used,
            "cost_estimate": checkpoint.cost_estimate,
            "resume_hint": checkpoint.resume_hint
        }

        # 添加校验和
        data = CheckpointValidator.add_checksum(data)

        # 写入文件
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)

        # 更新latest符号链接
        latest_path = session_dir / "latest.json"
        if latest_path.exists():
            latest_path.unlink()
        latest_path.symlink_to(filepath.name)

        logger.info(f"Checkpoint saved: {filename}")

    def _prune_old_checkpoints(self, session_id: str):
        """清理旧Checkpoint"""
        session_dir = self.storage_path / session_id / "checkpoints"
        if not session_dir.exists():
            return

        # 获取所有Checkpoint文件(排除latest链接)
        checkpoints = sorted(
            [f for f in session_dir.glob("cp_*.json")],
            key=lambda f: f.stat().st_mtime
        )

        # 删除超出限制的旧文件
        while len(checkpoints) > self.max_checkpoints:
            old_file = checkpoints.pop(0)
            old_file.unlink()
            logger.debug(f"Pruned old checkpoint: {old_file.name}")
```

#### 5.1.2 崩溃恢复

```python
class CrashRecovery:
    """崩溃恢复管理器"""

    def __init__(self, persistence: SessionPersistence):
        self.persistence = persistence

    def detect_crash(self) -> Optional[Dict]:
        """检测上次是否异常退出"""
        # 查找所有会话
        sessions_dir = self.persistence.storage_path
        if not sessions_dir.exists():
            return None

        # 查找最近修改的会话
        recent_sessions = sorted(
            sessions_dir.iterdir(),
            key=lambda d: d.stat().st_mtime,
            reverse=True
        )

        if not recent_sessions:
            return None

        # 检查会话是否异常终止
        for session_dir in recent_sessions[:5]:  # 只检查最近5个
            session_meta = session_dir / "session_meta.json"
            if not session_meta.exists():
                continue

            try:
                with open(session_meta, 'r') as f:
                    meta = json.load(f)

                # 检查是否正常结束
                if meta.get("is_active", False) and meta.get("termination_reason") is None:
                    # 会话仍在活动状态, 但未正常结束
                    return {
                        "session_id": meta["session_id"],
                        "last_activity": meta.get("last_updated"),
                        "last_iteration": meta.get("last_iteration", 0)
                    }
            except Exception:
                continue

        return None

    def recover_session(
        self,
        session_id: str
    ) -> Tuple[bool, Optional[ExecutionContext]]:
        """恢复会话"""
        # 加载最新Checkpoint
        checkpoint = self.persistence.load_latest_checkpoint(session_id)
        if not checkpoint:
            return False, None

        # 验证Checkpoint
        is_valid, error = CheckpointValidator.validate_checkpoint(checkpoint)
        if not is_valid:
            # 尝试修复
            is_valid, checkpoint = CheckpointRepairer.repair_checkpoint(checkpoint)
            if not is_valid:
                return False, None

        # 恢复ExecutionContext
        context = ExecutionContext.from_checkpoint(checkpoint)

        # 标记恢复信息
        context.resume_count += 1
        context.last_resumed_at = datetime.now()

        return True, context

    def show_recovery_prompt(self, crash_info: Dict) -> bool:
        """显示恢复提示"""
        console = Console()

        console.print("\n[yellow]检测到上次异常退出[/yellow]")
        console.print(f"会话ID: {crash_info['session_id']}")
        console.print(f"最后活动: {crash_info['last_activity']}")
        console.print(f"最后轮次: {crash_info['last_iteration']}")

        console.print("\n是否恢复上次会话? [Y/n]: ", end="")

        response = input().strip().lower()
        return response in ['', 'y', 'yes']
```

### 5.2 状态一致性

#### 5.2.1 并发控制

```python
import threading
from contextlib import contextmanager

class StateLock:
    """状态锁 - 使用 threading.Condition 实现等待/通知机制

    注意：RLock 没有 wait() 方法，只有 Condition 才有 wait()/notify_all()
    """

    def __init__(self):
        self._lock = threading.Lock()
        self._condition = threading.Condition(self._lock)
        self._readers = 0
        self._writers = 0

    @contextmanager
    def read_lock(self):
        """读锁"""
        with self._condition:
            while self._writers > 0:
                self._condition.wait()
            self._readers += 1

        try:
            yield
        finally:
            with self._condition:
                self._readers -= 1
                self._condition.notify_all()

    @contextmanager
    def write_lock(self):
        """写锁"""
        with self._condition:
            while self._readers > 0 or self._writers > 0:
                self._condition.wait()
            self._writers += 1

        try:
            yield
        finally:
            with self._condition:
                self._writers -= 1
                self._condition.notify_all()

class ConcurrentSessionManager:
    """并发安全会话管理器"""

    def __init__(self):
        self._sessions: Dict[str, ExecutionContext] = {}
        self._locks: Dict[str, StateLock] = {}
        self._global_lock = threading.Lock()

    def get_or_create_session(
        self,
        session_id: str,
        config: AgentConfig
    ) -> ExecutionContext:
        """获取或创建会话(线程安全)"""
        with self._global_lock:
            if session_id not in self._sessions:
                self._sessions[session_id] = ExecutionContext(
                    session_id=session_id,
                    agent_id=config.agent_id
                )
                self._locks[session_id] = StateLock()

            return self._sessions[session_id]

    def update_session(
        self,
        session_id: str,
        update_fn: Callable[[ExecutionContext], None]
    ):
        """更新会话状态(线程安全)"""
        if session_id not in self._locks:
            return

        with self._locks[session_id].write_lock():
            context = self._sessions[session_id]
            update_fn(context)

    def read_session(
        self,
        session_id: str,
        read_fn: Callable[[ExecutionContext], T]
    ) -> T:
        """读取会话状态(线程安全)"""
        if session_id not in self._locks:
            return None

        with self._locks[session_id].read_lock():
            context = self._sessions[session_id]
            return read_fn(context)
```

#### 5.2.2 事务性操作

```python
from typing import Callable, TypeVar

T = TypeVar('T')

class TransactionalExecutor:
    """事务性执行器"""

    def __init__(self, persistence: SessionPersistence):
        self.persistence = persistence

    @contextmanager
    def atomic_checkpoint(
        self,
        context: ExecutionContext,
        operation_name: str
    ):
        """原子性Checkpoint操作"""
        # 保存前置状态
        pre_checkpoint = context.to_checkpoint()
        pre_checkpoint.trigger = f"before_{operation_name}"

        try:
            # 执行操作
            yield context

            # 操作成功, 保存后置状态
            post_checkpoint = context.to_checkpoint()
            post_checkpoint.trigger = f"after_{operation_name}"
            self.persistence._save_checkpoint(post_checkpoint)

        except Exception as e:
            # 操作失败, 回滚到前置状态
            logger.error(f"Operation {operation_name} failed, rolling back: {e}")

            # 恢复到前置状态
            restored_context = ExecutionContext.from_checkpoint(pre_checkpoint)
            context.__dict__.update(restored_context.__dict__)

            # 保存回滚Checkpoint
            rollback_checkpoint = context.to_checkpoint()
            rollback_checkpoint.trigger = f"rollback_{operation_name}"
            self.persistence._save_checkpoint(rollback_checkpoint)

            raise

    def execute_with_retry_and_rollback(
        self,
        context: ExecutionContext,
        operation: Callable[[ExecutionContext], T],
        operation_name: str,
        max_retries: int = 3
    ) -> T:
        """带重试和回滚的执行"""
        last_error = None

        for attempt in range(max_retries):
            try:
                with self.atomic_checkpoint(context, f"{operation_name}_attempt{attempt}"):
                    result = operation(context)
                    return result
            except Exception as e:
                last_error = e
                logger.warning(
                    f"Attempt {attempt + 1}/{max_retries} failed for {operation_name}: {e}"
                )
                if attempt < max_retries - 1:
                    asyncio.sleep(2 ** attempt)  # 指数退避

        # 所有重试失败
        raise TransactionFailedError(
            f"All {max_retries} attempts failed for {operation_name}"
        ) from last_error
```

### 5.3 资源管理

#### 5.3.1 内存管理

```python
import gc
import weakref
from typing import Optional

class MemoryManager:
    """内存管理器"""

    MEMORY_LIMIT_MB = 512
    WARNING_THRESHOLD = 0.8
    CRITICAL_THRESHOLD = 0.95

    def __init__(self):
        self._tracked_objects: weakref.WeakSet = weakref.WeakSet()

    def track_object(self, obj: Any):
        """跟踪对象"""
        self._tracked_objects.add(obj)

    def check_memory(self) -> Dict:
        """检查内存使用"""
        import psutil
        process = psutil.Process()

        memory_mb = process.memory_info().rss / 1024 / 1024
        memory_percent = memory_mb / self.MEMORY_LIMIT_MB

        return {
            "used_mb": memory_mb,
            "limit_mb": self.MEMORY_LIMIT_MB,
            "percent": memory_percent,
            "status": self._get_status(memory_percent)
        }

    def _get_status(self, percent: float) -> str:
        """获取状态"""
        if percent < self.WARNING_THRESHOLD:
            return "ok"
        elif percent < self.CRITICAL_THRESHOLD:
            return "warning"
        else:
            return "critical"

    def optimize_if_needed(self) -> bool:
        """必要时优化内存"""
        status = self.check_memory()

        if status["status"] == "warning":
            logger.warning(
                f"Memory usage high: {status['used_mb']:.1f}MB / {status['limit_mb']:.1f}MB"
            )
            # 清理引用
            gc.collect()
            return True

        elif status["status"] == "critical":
            logger.error(
                f"Memory usage critical: {status['used_mb']:.1f}MB / {status['limit_mb']:.1f}MB"
            )
            # 强制垃圾回收
            gc.collect()
            # 清理缓存
            self._clear_caches()
            return True

        return False

    def _clear_caches(self):
        """清理缓存"""
        # 清理Checkpoint缓存
        # 清理消息历史缓存
        # 清理工具结果缓存
        pass

class ContextSizeLimiter:
    """上下文大小限制器"""

    MAX_MESSAGES = 100
    MAX_MESSAGE_LENGTH = 50000  # 字符

    def limit_context(
        self,
        messages: List[dict],
        max_tokens: int = 100000
    ) -> List[dict]:
        """限制上下文大小"""
        # 限制消息数量
        if len(messages) > self.MAX_MESSAGES:
            # 保留系统消息和最近的N条消息
            system_messages = [m for m in messages if m.get("role") == "system"]
            other_messages = [m for m in messages if m.get("role") != "system"]

            keep_count = self.MAX_MESSAGES - len(system_messages)
            messages = system_messages + other_messages[-keep_count:]

        # 限制单条消息长度
        limited_messages = []
        for msg in messages:
            if len(msg.get("content", "")) > self.MAX_MESSAGE_LENGTH:
                msg = msg.copy()
                msg["content"] = msg["content"][:self.MAX_MESSAGE_LENGTH] + "\n...[truncated]"
            limited_messages.append(msg)

        return limited_messages
```

#### 5.3.2 文件句柄管理

```python
import os
from contextlib import contextmanager
from typing import Generator

class FileHandleManager:
    """文件句柄管理器"""

    MAX_OPEN_FILES = 100
    WARNING_THRESHOLD = 80

    def __init__(self):
        self._open_files: Dict[int, str] = {}
        self._lock = threading.Lock()

    @contextmanager
    def open_file(
        self,
        filepath: str,
        mode: str = 'r',
        **kwargs
    ) -> Generator:
        """安全打开文件"""
        fd = None
        try:
            # 检查句柄数量
            self._check_handle_count()

            # 打开文件
            f = open(filepath, mode, **kwargs)
            fd = f.fileno()

            with self._lock:
                self._open_files[fd] = filepath

            yield f

        finally:
            if fd is not None:
                with self._lock:
                    self._open_files.pop(fd, None)
                try:
                    f.close()
                except Exception:
                    pass

    def _check_handle_count(self):
        """检查句柄数量"""
        with self._lock:
            count = len(self._open_files)

            if count > self.WARNING_THRESHOLD:
                logger.warning(
                    f"Open file handles high: {count}/{self.MAX_OPEN_FILES}"
                )

            if count >= self.MAX_OPEN_FILES:
                raise ResourceExhaustedError(
                    f"Too many open files: {count}/{self.MAX_OPEN_FILES}"
                )

    def get_open_files(self) -> Dict[int, str]:
        """获取打开的文件"""
        with self._lock:
            return self._open_files.copy()

    def close_all(self):
        """关闭所有文件"""
        with self._lock:
            for fd, filepath in self._open_files.items():
                try:
                    os.close(fd)
                except Exception:
                    pass
            self._open_files.clear()
```

---

## 六、可靠性测试策略

### 6.1 故障注入测试

```python
import random
from typing import Callable

class FaultInjector:
    """故障注入器"""

    def __init__(
        self,
        inject_probability: float = 0.1,
        fault_types: List[str] = None
    ):
        self.inject_probability = inject_probability
        self.fault_types = fault_types or [
            "network_error",
            "timeout_error",
            "rate_limit_error",
            "invalid_response_error",
            "checkpoint_corruption"
        ]

    def should_inject(self) -> bool:
        """是否注入故障"""
        return random.random() < self.inject_probability

    def inject_llm_fault(self) -> Exception:
        """注入LLM故障"""
        fault_type = random.choice(self.fault_types[:4])

        if fault_type == "network_error":
            return ConnectionError("Injected network error")
        elif fault_type == "timeout_error":
            return TimeoutError("Injected timeout error")
        elif fault_type == "rate_limit_error":
            return RateLimitError("Injected rate limit error")
        else:
            return ValueError("Injected invalid response")

    async def inject_with_probability(
        self,
        func: Callable,
        inject_fault: bool = True
    ):
        """按概率注入故障"""
        if inject_fault and self.should_inject():
            raise self.inject_llm_fault()
        return await func()
```

### 6.2 混沌测试

```python
class ChaosTester:
    """混沌测试器"""

    def __init__(self):
        self.fault_injector = FaultInjector()

    async def run_chaos_test(
        self,
        agent_func: Callable,
        iterations: int = 100
    ) -> Dict:
        """运行混沌测试"""
        results = {
            "total_iterations": iterations,
            "success_count": 0,
            "failure_count": 0,
            "recovery_count": 0
        }

        for i in range(iterations):
            try:
                # 运行Agent(带故障注入)
                await self.fault_injector.inject_with_probability(
                    agent_func,
                    inject_fault=(i % 5 == 0)  # 每5次注入1次故障
                )
                results["success_count"] += 1
            except Exception as e:
                results["failure_count"] += 1

                # 测试恢复能力
                try:
                    await self._test_recovery()
                    results["recovery_count"] += 1
                except Exception:
                    pass

        # 计算可靠性指标
        results["success_rate"] = results["success_count"] / iterations
        results["recovery_rate"] = (
            results["recovery_count"] / results["failure_count"]
            if results["failure_count"] > 0 else 0
        )

        return results

    async def _test_recovery(self):
        """测试恢复能力"""
        # 测试Checkpoint恢复
        # 测试重试机制
        # 测试降级机制
        pass
```

### 6.3 压力测试

```python
class StressTester:
    """压力测试器"""

    async def stress_test_concurrent_sessions(
        self,
        num_sessions: int = 10,
        duration_seconds: int = 60
    ) -> Dict:
        """并发会话压力测试"""
        results = {
            "sessions_created": 0,
            "sessions_completed": 0,
            "sessions_failed": 0,
            "total_checkpoints": 0,
            "peak_memory_mb": 0
        }

        tasks = []
        start_time = time.time()

        for i in range(num_sessions):
            task = asyncio.create_task(
                self._run_session(f"stress_test_{i}", duration_seconds)
            )
            tasks.append(task)

        # 等待所有任务完成
        done, pending = await asyncio.wait(
            tasks,
            timeout=duration_seconds + 10
        )

        # 统计结果
        for task in done:
            try:
                result = task.result()
                results["sessions_completed"] += 1
                results["total_checkpoints"] += result.get("checkpoints", 0)
            except Exception:
                results["sessions_failed"] += 1

        results["sessions_created"] = num_sessions

        return results

    async def _run_session(
        self,
        session_id: str,
        duration: int
    ) -> Dict:
        """运行单个会话"""
        checkpoint_count = 0
        start_time = time.time()

        while time.time() - start_time < duration:
            # 模拟Agent执行
            await asyncio.sleep(1)
            checkpoint_count += 1

        return {"checkpoints": checkpoint_count}
```

---

## 七、监控告警系统

### 7.1 健康检查端点

```python
from fastapi import FastAPI, Response
from typing import Dict

app = FastAPI()

class HealthChecker:
    """健康检查器"""

    def __init__(self):
        self.metrics_collector = MetricsCollector()
        self.memory_manager = MemoryManager()

    def check_health(self) -> Dict:
        """健康检查"""
        checks = {
            "status": "healthy",
            "timestamp": datetime.now().isoformat(),
            "checks": {}
        }

        # 内存检查
        memory_status = self.memory_manager.check_memory()
        checks["checks"]["memory"] = {
            "status": memory_status["status"],
            "used_mb": memory_status["used_mb"],
            "limit_mb": memory_status["limit_mb"]
        }

        # Checkpoint检查
        # pending_checkpoints = self._get_pending_checkpoints()
        # checks["checks"]["checkpoints"] = {
        #     "status": "ok" if pending_checkpoints < 100 else "warning",
        #     "pending": pending_checkpoints
        # }

        # 确定整体状态
        if any(c["status"] in ["warning", "critical"] for c in checks["checks"].values()):
            checks["status"] = "degraded"

        return checks

# FastAPI端点
health_checker = HealthChecker()

@app.get("/health")
async def health_check():
    """健康检查端点"""
    health = health_checker.check_health()

    if health["status"] == "healthy":
        return health
    elif health["status"] == "degraded":
        return Response(
            content=json.dumps(health),
            status_code=200,
            media_type="application/json"
        )
    else:
        return Response(
            content=json.dumps(health),
            status_code=503,
            media_type="application/json"
        )

@app.get("/ready")
async def readiness_check():
    """就绪检查端点"""
    # 检查是否可以接受请求
    return {"status": "ready"}

@app.get("/metrics")
async def metrics_endpoint():
    """Prometheus格式指标端点"""
    metrics = health_checker.metrics_collector.get_dashboard_data()
    return metrics
```

### 7.2 告警规则

```python
class AlertRule:
    """告警规则"""

    def __init__(
        self,
        name: str,
        condition: Callable[[Dict], bool],
        severity: str,  # info, warning, critical
        message: str,
        cooldown_seconds: int = 300
    ):
        self.name = name
        self.condition = condition
        self.severity = severity
        self.message = message
        self.cooldown_seconds = cooldown_seconds
        self.last_triggered: Optional[datetime] = None

    def should_alert(self, metrics: Dict) -> bool:
        """是否应该告警"""
        if not self.condition(metrics):
            return False

        # 检查冷却时间
        if self.last_triggered:
            elapsed = (datetime.now() - self.last_triggered).total_seconds()
            if elapsed < self.cooldown_seconds:
                return False

        self.last_triggered = datetime.now()
        return True

class AlertManager:
    """告警管理器"""

    def __init__(self):
        self.rules: List[AlertRule] = []
        self._setup_default_rules()

    def _setup_default_rules(self):
        """设置默认告警规则"""
        # 内存告警
        self.rules.append(AlertRule(
            name="high_memory",
            condition=lambda m: m.get("memory_percent", 0) > 80,
            severity="warning",
            message="Memory usage above 80%"
        ))

        # 错误率告警
        self.rules.append(AlertRule(
            name="high_error_rate",
            condition=lambda m: m.get("error_rate", 0) > 0.1,
            severity="critical",
            message="Error rate above 10%"
        ))

        # Checkpoint失败告警
        self.rules.append(AlertRule(
            name="checkpoint_failure",
            condition=lambda m: m.get("checkpoint_failure_count", 0) > 3,
            severity="warning",
            message="Multiple checkpoint failures"
        ))

    def check_alerts(self, metrics: Dict) -> List[Dict]:
        """检查告警"""
        alerts = []

        for rule in self.rules:
            if rule.should_alert(metrics):
                alerts.append({
                    "name": rule.name,
                    "severity": rule.severity,
                    "message": rule.message,
                    "timestamp": datetime.now().isoformat()
                })

        return alerts

    def send_alert(self, alert: Dict):
        """发送告警"""
        # 发送到日志
        if alert["severity"] == "critical":
            logger.critical(f"ALERT: {alert['name']} - {alert['message']}")
        elif alert["severity"] == "warning":
            logger.warning(f"ALERT: {alert['name']} - {alert['message']}")
        else:
            logger.info(f"ALERT: {alert['name']} - {alert['message']}")

        # 可扩展: 发送到邮件/Slack/PagerDuty等
```

---

## 八、总结与实施路线图

### 8.1 可靠性能力矩阵

| 能力域 | MVP阶段 | Phase 2 | Phase 3 | 企业版 |
|--------|---------|---------|---------|--------|
| **容错** | 基础重试(3次) | 熔断器 + Fallback | 完整容错体系 | 多活容灾 |
| **可观测** | 控制台追踪 | 结构化日志 | Dashboard | 完整APM |
| **成本** | 实时显示 | 预测 + 预算 | 报表 + 分析 | 多项目预算 |
| **安全** | 环境变量 | 密钥轮换 | 沙箱隔离 | 完整安全体系 |
| **高可用** | 自动Checkpoint | 崩溃恢复 | 并发安全 | 99.9% SLA |

### 8.2 实施优先级

```
P0 (MVP必须):
├── 基础重试机制 (指数退避)
├── Checkpoint完整性校验
├── API密钥环境变量读取
├── 控制台追踪输出
└── 实时成本显示

P1 (短期):
├── 熔断器
├── 工具执行超时
├── 崩溃恢复
└── 敏感数据脱敏

P2 (中期):
├── 模型Fallback
├── Dashboard
├── 成本预测
└── 工具沙箱

P3 (企业版):
├── 完整APM
├── 多项目预算
├── 完整安全体系
└── 99.9% SLA保障
```

### 8.3 成功指标

| 指标 | MVP目标 | 6个月目标 | 企业版目标 |
|------|---------|----------|-----------|
| 可用性 | 95% | 99% | 99.9% |
| RTO(恢复时间) | <30分钟 | <10分钟 | <5分钟 |
| Checkpoint成功率 | 95% | 99% | 99.9% |
| 错误自动恢复率 | 70% | 90% | 95% |
| 内存泄漏 | 0 | 0 | 0 |

---

**文档状态**: v1.0 新建
**下一步**: 实现P0可靠性机制, 编写单元测试, 集成测试验证
