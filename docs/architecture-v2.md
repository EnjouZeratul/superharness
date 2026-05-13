# Egg-Harness 架构设计 v2.0

> 版本: v2.0  
> 日期: 2026-05-08  
> 变更: 移除 LangChain/LangGraph 依赖，全自研架构

---

## 一、架构决策

### 1.1 为什么移除 LangChain？

| 问题 | 说明 |
|------|------|
| **抽象过重** | LangChain 的 Runnable/Chain 抽象层增加了理解和调试难度 |
| **版本频繁变更** | API 不稳定，升级成本高 |
| **性能开销** | 多层抽象带来额外的性能损耗 |
| **成熟框架不用** | Aider、Claude Code、OpenCode 都直接调用 API |
| **灵活性受限** | 深度定制需要绕过框架限制 |

### 1.2 自研策略

```
借鉴而非依赖：
├── 借鉴 LangChain 的 Tool Schema 生成方式
├── 借鉴 LangGraph 的状态图设计思想
├── 借鉴 Aider 的 Repository Map 算法
├── 借鉴 Claude Code 的记忆系统
└── 直接调用各厂商 API（httpx/aiohttp）
```

---

## 二、整体架构

### 2.1 分层架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        egg-harness v2                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Interface Layer                          │ │
│  │    CLI  |  Python SDK  |  REST API  |  MCP Server          │ │
│  └───────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Core Engine                              │ │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │ │
│  │  │ Context │ │  Tool   │ │Workflow │ │ Memory  │         │ │
│  │  │ Manager │ │ System  │ │ Engine  │ │ System  │         │ │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘         │ │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │ │
│  │  │  Agent  │ │  Event  │ │ Feedback│ │  Hook   │         │ │
│  │  │ Runtime │ │  Bus    │ │  Loop   │ │ System  │         │ │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘         │ │
│  └───────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Integration Layer                        │ │
│  │    Git  |  MCP  |  LSP  |  CI/CD  |  Web                   │ │
│  └───────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Storage Layer                            │ │
│  │    Memory  |  SQLite  |  Redis  |  PostgreSQL              │ │
│  └───────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    LLM Provider Layer                       │ │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐  │ │
│  │  │ OpenAI │ │Anthropic│ │DashScope│ │ZhipuAI │ │DeepSeek│  │ │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘  │ │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐             │ │
│  │  │ Ollama │ │ vLLM   │ │OpenRouter│ │ 自定义 │             │ │
│  │  └────────┘ └────────┘ └────────┘ └────────┘             │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 核心模块职责

| 模块 | 职责 | 关键能力 |
|------|------|----------|
| **Agent Runtime** | Agent 执行循环 | Tool Calling Loop、流式输出、错误恢复 |
| **Context Manager** | 上下文管理 | 消息历史、Auto Compact、Repository Map |
| **Tool System** | 工具管理 | 注册、Schema 生成、拦截器、MCP |
| **Workflow Engine** | 工作流编排 | 状态图、条件分支、并行执行 |
| **Memory System** | 记忆管理 | CLAUDE.md 风格、Auto Memory |
| **Event Bus** | 事件系统 | 发布/订阅、异步处理 |
| **Feedback Loop** | 反馈闭环 | 工具结果→上下文、用户反馈 |
| **Hook System** | 钩子系统 | 工具执行前后钩子 |

---

## 三、核心模块设计

### 3.1 LLM Provider（直接 API 调用）

#### 3.1.1 统一接口

```python
from abc import ABC, abstractmethod
from typing import AsyncIterator, Optional
from pydantic import BaseModel

class Message(BaseModel):
    """统一消息格式"""
    role: str  # system | user | assistant | tool
    content: str
    tool_calls: Optional[list["ToolCall"]] = None
    tool_call_id: Optional[str] = None
    name: Optional[str] = None

class ToolCall(BaseModel):
    """工具调用"""
    id: str
    type: str = "function"
    function: FunctionCall

class FunctionCall(BaseModel):
    """函数调用"""
    name: str
    arguments: str  # JSON 字符串

class LLMResponse(BaseModel):
    """LLM 响应"""
    content: Optional[str] = None
    tool_calls: Optional[list[ToolCall]] = None
    finish_reason: str  # stop | tool_calls | length
    usage: Optional["Usage"] = None

class LLMProvider(ABC):
    """LLM 提供商基类 - 直接调用 API"""
    
    @abstractmethod
    async def chat(
        self,
        messages: list[Message],
        model: str,
        tools: Optional[list[dict]] = None,
        temperature: float = 0.7,
        max_tokens: Optional[int] = None,
        stream: bool = False,
    ) -> LLMResponse | AsyncIterator[LLMResponse]:
        """发送聊天请求"""
        pass
    
    @abstractmethod
    async def validate_api_key(self) -> bool:
        """验证 API Key"""
        pass
```

#### 3.1.2 OpenAI 适配器（使用 httpx 直接调用）

```python
import httpx
import json

class OpenAIProvider(LLMProvider):
    """OpenAI 适配器 - 直接调用 API"""
    
    BASE_URL = "https://api.openai.com/v1"
    
    def __init__(self, api_key: str, base_url: Optional[str] = None):
        self.api_key = api_key
        self.base_url = base_url or self.BASE_URL
        self.client = httpx.AsyncClient(
            base_url=self.base_url,
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            timeout=120.0,
        )
    
    async def chat(
        self,
        messages: list[Message],
        model: str,
        tools: Optional[list[dict]] = None,
        temperature: float = 0.7,
        max_tokens: Optional[int] = None,
        stream: bool = False,
    ) -> LLMResponse | AsyncIterator[LLMResponse]:
        """调用 OpenAI Chat API"""
        
        payload = {
            "model": model,
            "messages": [self._format_message(m) for m in messages],
            "temperature": temperature,
        }
        
        if max_tokens:
            payload["max_tokens"] = max_tokens
        
        if tools:
            payload["tools"] = tools
            payload["tool_choice"] = "auto"
        
        if stream:
            return self._stream_chat(payload)
        
        response = await self.client.post("/chat/completions", json=payload)
        response.raise_for_status()
        data = response.json()
        
        return self._parse_response(data)
    
    def _format_message(self, message: Message) -> dict:
        """格式化消息为 OpenAI 格式"""
        msg = {"role": message.role, "content": message.content}
        
        if message.tool_calls:
            msg["tool_calls"] = [
                {
                    "id": tc.id,
                    "type": "function",
                    "function": {
                        "name": tc.function.name,
                        "arguments": tc.function.arguments,
                    }
                }
                for tc in message.tool_calls
            ]
        
        if message.tool_call_id:
            msg["tool_call_id"] = message.tool_call_id
        
        return msg
    
    def _parse_response(self, data: dict) -> LLMResponse:
        """解析 OpenAI 响应"""
        choice = data["choices"][0]
        message = choice["message"]
        
        tool_calls = None
        if message.get("tool_calls"):
            tool_calls = [
                ToolCall(
                    id=tc["id"],
                    function=FunctionCall(
                        name=tc["function"]["name"],
                        arguments=tc["function"]["arguments"],
                    )
                )
                for tc in message["tool_calls"]
            ]
        
        return LLMResponse(
            content=message.get("content"),
            tool_calls=tool_calls,
            finish_reason=choice["finish_reason"],
            usage=data.get("usage"),
        )
    
    async def _stream_chat(self, payload: dict) -> AsyncIterator[LLMResponse]:
        """流式聊天"""
        payload["stream"] = True
        
        async with self.client.stream("POST", "/chat/completions", json=payload) as response:
            response.raise_for_status()
            async for line in response.aiter_lines():
                if line.startswith("data: "):
                    data = line[6:]
                    if data == "[DONE]":
                        break
                    chunk = json.loads(data)
                    yield self._parse_chunk(chunk)
```

#### 3.1.3 Anthropic 适配器

```python
class AnthropicProvider(LLMProvider):
    """Anthropic 适配器 - 直接调用 Claude API"""
    
    BASE_URL = "https://api.anthropic.com/v1"
    
    def __init__(self, api_key: str):
        self.api_key = api_key
        self.client = httpx.AsyncClient(
            base_url=self.BASE_URL,
            headers={
                "x-api-key": api_key,
                "anthropic-version": "2023-06-01",
                "Content-Type": "application/json",
            },
            timeout=120.0,
        )
    
    async def chat(
        self,
        messages: list[Message],
        model: str,
        tools: Optional[list[dict]] = None,
        temperature: float = 0.7,
        max_tokens: Optional[int] = None,
        stream: bool = False,
    ) -> LLMResponse:
        """调用 Claude API"""
        
        # Claude 的 system 需要单独传递
        system_msg = None
        chat_messages = []
        
        for msg in messages:
            if msg.role == "system":
                system_msg = msg.content
            else:
                chat_messages.append(self._format_message(msg))
        
        payload = {
            "model": model,
            "messages": chat_messages,
            "max_tokens": max_tokens or 4096,
            "temperature": temperature,
        }
        
        if system_msg:
            payload["system"] = system_msg
        
        if tools:
            payload["tools"] = [self._format_tool(t) for t in tools]
        
        response = await self.client.post("/messages", json=payload)
        response.raise_for_status()
        data = response.json()
        
        return self._parse_response(data)
    
    def _format_tool(self, tool: dict) -> dict:
        """格式化工具为 Claude 格式"""
        return {
            "name": tool["function"]["name"],
            "description": tool["function"]["description"],
            "input_schema": tool["function"]["parameters"],
        }
```

### 3.2 Agent Runtime（Tool Calling Loop）

```python
class AgentRuntime:
    """Agent 执行运行时 - 核心循环"""
    
    def __init__(
        self,
        llm: LLMProvider,
        model: str,
        tools: ToolRegistry,
        context: ContextManager,
        hooks: HookSystem,
    ):
        self.llm = llm
        self.model = model
        self.tools = tools
        self.context = context
        self.hooks = hooks
    
    async def run(
        self,
        user_input: str,
        max_iterations: int = 10,
    ) -> str:
        """执行 Agent 循环"""
        
        # 1. 添加用户消息
        await self.context.add_message(Message(
            role="user",
            content=user_input,
        ))
        
        # 2. Tool Calling Loop
        for i in range(max_iterations):
            # 获取上下文
            messages = await self.context.get_messages()
            tool_schemas = self.tools.get_schemas()
            
            # 调用 LLM
            response = await self.llm.chat(
                messages=messages,
                model=self.model,
                tools=tool_schemas if tool_schemas else None,
            )
            
            # 3. 处理响应
            if response.finish_reason == "stop":
                # 纯文本响应，结束循环
                await self.context.add_message(Message(
                    role="assistant",
                    content=response.content,
                ))
                return response.content
            
            elif response.finish_reason == "tool_calls":
                # 需要调用工具
                await self.context.add_message(Message(
                    role="assistant",
                    content=response.content,
                    tool_calls=response.tool_calls,
                ))
                
                # 执行所有工具调用
                for tool_call in response.tool_calls:
                    result = await self._execute_tool(tool_call)
                    
                    # 工具结果注入上下文
                    await self.context.add_message(Message(
                        role="tool",
                        content=result,
                        tool_call_id=tool_call.id,
                        name=tool_call.function.name,
                    ))
            
            else:
                # 异常情况
                break
        
        return "达到最大迭代次数"
    
    async def _execute_tool(self, tool_call: ToolCall) -> str:
        """执行单个工具调用"""
        tool_name = tool_call.function.name
        
        # 执行前置钩子
        await self.hooks.run_before(tool_name, tool_call.function.arguments)
        
        try:
            # 解析参数
            args = json.loads(tool_call.function.arguments)
            
            # 执行工具
            result = await self.tools.invoke(tool_name, **args)
            
            # 执行后置钩子
            await self.hooks.run_after(tool_name, result)
            
            return str(result)
        
        except Exception as e:
            # 错误处理
            error_msg = f"工具 {tool_name} 执行失败: {str(e)}"
            await self.hooks.run_on_error(tool_name, e)
            return error_msg
```

### 3.3 Tool System

```python
import inspect
from typing import Callable, Any, get_type_hints

class ToolDefinition(BaseModel):
    """工具定义"""
    name: str
    description: str
    parameters: dict[str, Any]  # JSON Schema
    func: Callable
    permissions: list[str] = []

class ToolRegistry:
    """工具注册表"""
    
    def __init__(self):
        self._tools: dict[str, ToolDefinition] = {}
    
    def tool(
        self,
        name: Optional[str] = None,
        description: Optional[str] = None,
        permissions: Optional[list[str]] = None,
    ) -> Callable:
        """工具装饰器"""
        def decorator(func: Callable) -> Callable:
            tool_name = name or func.__name__
            tool_desc = description or func.__doc__ or ""
            
            # 从函数签名生成 JSON Schema
            parameters = self._generate_schema(func)
            
            self._tools[tool_name] = ToolDefinition(
                name=tool_name,
                description=tool_desc,
                parameters=parameters,
                func=func,
                permissions=permissions or [],
            )
            
            @wraps(func)
            async def wrapper(*args, **kwargs):
                return await self.invoke(tool_name, *args, **kwargs)
            
            return wrapper
        return decorator
    
    def _generate_schema(self, func: Callable) -> dict:
        """从函数签名生成 JSON Schema"""
        sig = inspect.signature(func)
        hints = get_type_hints(func)
        
        properties = {}
        required = []
        
        for name, param in sig.parameters.items():
            if name == "self":
                continue
            
            hint = hints.get(name, str)
            prop = self._type_to_schema(hint)
            
            # 从 docstring 获取描述
            prop["description"] = self._get_param_desc(func, name)
            
            properties[name] = prop
            
            if param.default == inspect.Parameter.empty:
                required.append(name)
        
        return {
            "type": "object",
            "properties": properties,
            "required": required,
        }
    
    def _type_to_schema(self, type_hint) -> dict:
        """Python 类型转 JSON Schema"""
        type_map = {
            str: {"type": "string"},
            int: {"type": "integer"},
            float: {"type": "number"},
            bool: {"type": "boolean"},
            list: {"type": "array"},
            dict: {"type": "object"},
        }
        return type_map.get(type_hint, {"type": "string"})
    
    def get_schemas(self) -> list[dict]:
        """获取所有工具的 OpenAI 格式 schema"""
        return [
            {
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                }
            }
            for tool in self._tools.values()
        ]
    
    async def invoke(self, name: str, **kwargs) -> Any:
        """调用工具"""
        tool = self._tools.get(name)
        if not tool:
            raise ToolNotFoundError(f"Tool '{name}' not found")
        
        # 支持同步和异步函数
        if inspect.iscoroutinefunction(tool.func):
            return await tool.func(**kwargs)
        else:
            return tool.func(**kwargs)
```

### 3.4 Context Manager

```python
class ContextManager:
    """上下文管理器"""
    
    def __init__(
        self,
        max_tokens: int = 128000,
        auto_compact_threshold: float = 0.95,
        storage: Optional[StorageBackend] = None,
    ):
        self.max_tokens = max_tokens
        self.auto_compact_threshold = auto_compact_threshold
        self.storage = storage or MemoryStorage()
        
        self._messages: list[Message] = []
        self._system_prompt: Optional[str] = None
        self._token_counter = TokenCounter()
    
    async def add_message(self, message: Message) -> None:
        """添加消息"""
        self._messages.append(message)
        
        # 检查是否需要自动压缩
        if self._should_compact():
            await self._auto_compact()
    
    async def get_messages(self) -> list[Message]:
        """获取所有消息"""
        messages = []
        
        # 添加系统提示词
        if self._system_prompt:
            messages.append(Message(
                role="system",
                content=self._system_prompt,
            ))
        
        messages.extend(self._messages)
        return messages
    
    async def set_system_prompt(self, prompt: str) -> None:
        """设置系统提示词"""
        self._system_prompt = prompt
    
    def _should_compact(self) -> bool:
        """检查是否需要压缩"""
        total_tokens = self._token_counter.count_messages(self._messages)
        return total_tokens > self.max_tokens * self.auto_compact_threshold
    
    async def _auto_compact(self) -> None:
        """自动压缩上下文"""
        # 保留系统消息和最近的消息
        system_msgs = [m for m in self._messages if m.role == "system"]
        other_msgs = [m for m in self._messages if m.role != "system"]
        
        # 生成摘要
        summary = await self._generate_summary(other_msgs[:-10])
        
        # 重新组织消息
        self._messages = [
            Message(role="system", content=f"历史摘要:\n{summary}"),
            *other_msgs[-10:],
        ]
    
    async def _generate_summary(self, messages: list[Message]) -> str:
        """生成消息摘要"""
        # 使用 LLM 生成摘要
        # 这里需要注入一个轻量级 LLM
        pass
```

### 3.5 Workflow Engine（状态图）

```python
from typing import TypedDict, Callable, Any
from enum import Enum

class NodeStatus(Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"

class WorkflowNode:
    """工作流节点"""
    
    def __init__(self, name: str, func: Callable):
        self.name = name
        self.func = func
    
    async def execute(self, state: dict) -> dict:
        """执行节点"""
        if inspect.iscoroutinefunction(self.func):
            return await self.func(state)
        else:
            return self.func(state)

class WorkflowEdge:
    """工作流边"""
    
    def __init__(
        self,
        source: str,
        target: str,
        condition: Optional[Callable] = None,
    ):
        self.source = source
        self.target = target
        self.condition = condition  # 条件函数，返回 bool

class StateGraph:
    """状态图"""
    
    def __init__(self, state_schema: type = dict):
        self.state_schema = state_schema
        self.nodes: dict[str, WorkflowNode] = {}
        self.edges: list[WorkflowEdge] = []
        self.entry_point: Optional[str] = None
    
    def add_node(self, name: str, func: Callable) -> "StateGraph":
        """添加节点"""
        self.nodes[name] = WorkflowNode(name, func)
        return self
    
    def add_edge(self, source: str, target: str) -> "StateGraph":
        """添加边"""
        self.edges.append(WorkflowEdge(source, target))
        return self
    
    def add_conditional_edge(
        self,
        source: str,
        condition: Callable,
        targets: dict[str, str],
    ) -> "StateGraph":
        """添加条件边"""
        for condition_value, target in targets.items():
            self.edges.append(WorkflowEdge(
                source=source,
                target=target,
                condition=lambda state, cv=condition_value: condition(state) == cv,
            ))
        return self
    
    def set_entry_point(self, name: str) -> "StateGraph":
        """设置入口点"""
        self.entry_point = name
        return self
    
    def compile(self) -> "CompiledGraph":
        """编译工作流"""
        return CompiledGraph(self)

class CompiledGraph:
    """编译后的工作流"""
    
    def __init__(self, graph: StateGraph):
        self.graph = graph
    
    async def invoke(self, initial_state: dict) -> dict:
        """执行工作流"""
        state = initial_state.copy()
        current_node = self.graph.entry_point
        
        while current_node and current_node != "__end__":
            node = self.graph.nodes.get(current_node)
            if not node:
                raise WorkflowError(f"Node '{current_node}' not found")
            
            # 执行节点
            state = await node.execute(state)
            
            # 查找下一个节点
            current_node = self._get_next_node(current_node, state)
        
        return state
    
    def _get_next_node(self, current: str, state: dict) -> Optional[str]:
        """获取下一个节点"""
        for edge in self.graph.edges:
            if edge.source == current:
                if edge.condition is None:
                    return edge.target
                if edge.condition(state):
                    return edge.target
        
        return "__end__"
```

### 3.6 Memory System

```python
class ProjectMemory:
    """项目记忆 - 借鉴 CLAUDE.md"""
    
    def __init__(self, project_root: str):
        self.project_root = project_root
        self.memory_file = os.path.join(project_root, "EGG.md")
    
    async def load(self) -> str:
        """加载项目记忆"""
        if os.path.exists(self.memory_file):
            with open(self.memory_file, "r", encoding="utf-8") as f:
                return f.read()
        return ""
    
    async def save(self, content: str) -> None:
        """保存项目记忆"""
        with open(self.memory_file, "w", encoding="utf-8") as f:
            f.write(content)
    
    async def append(self, section: str, content: str) -> None:
        """追加内容到指定章节"""
        existing = await self.load()
        
        # 查找章节
        pattern = f"## {section}"
        if pattern in existing:
            # 追加到现有章节
            existing = existing.replace(pattern, f"{pattern}\n{content}")
        else:
            # 创建新章节
            existing += f"\n\n## {section}\n{content}"
        
        await self.save(existing)

class AutoMemory:
    """自动记忆 - 借鉴 Claude Code Auto Memory"""
    
    def __init__(self, storage: StorageBackend):
        self.storage = storage
    
    async def learn(self, key: str, value: str, category: str = "general") -> None:
        """学习新知识"""
        memory = {
            "key": key,
            "value": value,
            "category": category,
            "timestamp": time.time(),
            "access_count": 0,
        }
        
        await self.storage.set(f"memory:{category}:{key}", memory)
    
    async def recall(self, query: str, category: Optional[str] = None) -> list[dict]:
        """回忆相关知识"""
        # 简单实现：关键词匹配
        # 进阶实现：向量相似度搜索
        prefix = f"memory:{category}:" if category else "memory:"
        keys = await self.storage.list(prefix)
        
        results = []
        for key in keys:
            memory = await self.storage.get(key)
            if query.lower() in memory["value"].lower():
                memory["access_count"] += 1
                results.append(memory)
        
        return sorted(results, key=lambda x: x["access_count"], reverse=True)
```

### 3.7 Hook System

```python
from typing import Callable, Any
from enum import Enum

class HookType(Enum):
    BEFORE_TOOL = "before_tool"
    AFTER_TOOL = "after_tool"
    ON_ERROR = "on_error"
    BEFORE_LLM = "before_llm"
    AFTER_LLM = "after_llm"

class HookSystem:
    """钩子系统"""
    
    def __init__(self):
        self._hooks: dict[HookType, list[Callable]] = {}
    
    def register(self, hook_type: HookType, handler: Callable) -> None:
        """注册钩子"""
        if hook_type not in self._hooks:
            self._hooks[hook_type] = []
        self._hooks[hook_type].append(handler)
    
    async def run_before(self, tool_name: str, args: str) -> None:
        """执行工具前置钩子"""
        for hook in self._hooks.get(HookType.BEFORE_TOOL, []):
            await hook(tool_name, args)
    
    async def run_after(self, tool_name: str, result: Any) -> None:
        """执行工具后置钩子"""
        for hook in self._hooks.get(HookType.AFTER_TOOL, []):
            await hook(tool_name, result)
    
    async def run_on_error(self, tool_name: str, error: Exception) -> None:
        """执行错误钩子"""
        for hook in self._hooks.get(HookType.ON_ERROR, []):
            await hook(tool_name, error)

# 内置钩子
class LoggingHook:
    """日志钩子"""
    
    def __init__(self, logger):
        self.logger = logger
    
    async def before_tool(self, tool_name: str, args: str):
        self.logger.info(f"Calling tool: {tool_name}", args=args)
    
    async def after_tool(self, tool_name: str, result: Any):
        self.logger.info(f"Tool completed: {tool_name}")
    
    async def on_error(self, tool_name: str, error: Exception):
        self.logger.error(f"Tool failed: {tool_name}", error=str(error))

class MetricsHook:
    """指标钩子"""
    
    def __init__(self):
        self.tool_calls = {}
        self.tool_errors = {}
    
    async def after_tool(self, tool_name: str, result: Any):
        self.tool_calls[tool_name] = self.tool_calls.get(tool_name, 0) + 1
    
    async def on_error(self, tool_name: str, error: Exception):
        self.tool_errors[tool_name] = self.tool_errors.get(tool_name, 0) + 1
```

---

## 四、项目结构

```
egg-harness/
├── src/
│   └── egg_harness/
│       ├── __init__.py
│       ├── core/
│       │   ├── __init__.py
│       │   ├── harness.py          # Harness 主类
│       │   ├── agent.py            # Agent 类
│       │   └── config.py           # 配置管理
│       ├── llm/
│       │   ├── __init__.py
│       │   ├── base.py             # LLMProvider 基类
│       │   ├── registry.py         # Provider 注册表
│       │   ├── messages.py         # 消息类型定义
│       │   └── providers/
│       │       ├── __init__.py
│       │       ├── openai.py       # OpenAI 适配器
│       │       ├── anthropic.py    # Anthropic 适配器
│       │       ├── dashscope.py    # 阿里云适配器
│       │       ├── zhipuai.py      # 智谱适配器
│       │       ├── deepseek.py     # DeepSeek 适配器
│       │       └── ollama.py       # Ollama 适配器
│       ├── runtime/
│       │   ├── __init__.py
│       │   ├── loop.py             # Agent 执行循环
│       │   └── streaming.py        # 流式输出处理
│       ├── context/
│       │   ├── __init__.py
│       │   ├── manager.py          # ContextManager
│       │   ├── compact.py          # Auto Compact
│       │   ├── repo_map.py         # Repository Map
│       │   └── token_counter.py    # Token 计数器
│       ├── tools/
│       │   ├── __init__.py
│       │   ├── registry.py         # ToolRegistry
│       │   ├── schema.py           # Schema 生成
│       │   ├── interceptor.py      # 拦截器
│       │   └── builtin/
│       │       ├── __init__.py
│       │       ├── file_ops.py     # 文件操作工具
│       │       ├── shell.py        # Shell 命令工具
│       │       ├── search.py       # 搜索工具
│       │       └── git.py          # Git 工具
│       ├── workflow/
│       │   ├── __init__.py
│       │   ├── graph.py            # StateGraph
│       │   ├── node.py             # 节点定义
│       │   └── edge.py             # 边定义
│       ├── memory/
│       │   ├── __init__.py
│       │   ├── project.py          # ProjectMemory
│       │   ├── auto.py             # AutoMemory
│       │   └── vector.py           # 向量记忆
│       ├── hooks/
│       │   ├── __init__.py
│       │   ├── system.py           # HookSystem
│       │   └── builtin/
│       │       ├── logging.py      # 日志钩子
│       │       └── metrics.py      # 指标钩子
│       ├── events/
│       │   ├── __init__.py
│       │   └── bus.py              # EventBus
│       ├── feedback/
│       │   ├── __init__.py
│       │   └── loop.py             # FeedbackLoop
│       ├── storage/
│       │   ├── __init__.py
│       │   ├── base.py             # StorageBackend
│       │   ├── memory.py           # 内存存储
│       │   └── sqlite.py           # SQLite 存储
│       └── integrations/
│           ├── __init__.py
│           ├── mcp.py              # MCP 支持
│           └── git.py              # Git 集成
├── tests/
├── examples/
├── docs/
├── pyproject.toml
└── README.md
```

---

## 五、开发计划

### Phase 1: 核心基础 (3-4 周)

- [ ] LLM Provider 框架 + OpenAI/Anthropic 适配器
- [ ] Agent Runtime (Tool Calling Loop)
- [ ] 基础 Tool Registry + Schema 生成
- [ ] ContextManager (消息历史)
- [ ] 基础工具 (文件读写、Shell 执行)
- [ ] 配置系统 (YAML)

### Phase 2: 上下文增强 (3-4 周)

- [ ] Auto Compact (自动上下文压缩)
- [ ] Repository Map (代码库地图)
- [ ] Memory System (项目记忆 + 自动记忆)
- [ ] 更多 LLM Provider (DashScope, ZhipuAI, DeepSeek)
- [ ] Token 计数器

### Phase 3: 工作流和钩子 (3-4 周)

- [ ] StateGraph 工作流引擎
- [ ] Hook System
- [ ] Event Bus
- [ ] Feedback Loop (工具结果→上下文)
- [ ] 更多内置工具

### Phase 4: 集成和生产 (2-3 周)

- [ ] MCP 支持
- [ ] Git 集成
- [ ] SQLite 存储后端
- [ ] CLI 接口
- [ ] 文档和示例

---

## 七、通用设计模式（15 个）

### 7.1 P0 核心模式

#### 模式 1: ACI（Agent-Computer Interface）设计

**来源**: SWE-Agent、AutoCodeRover

**核心思想**: 为 Agent 专门设计工具接口，而非暴露系统 API

```python
# ❌ 错误做法：暴露系统 API
tools = [
    "os.path.join",
    "subprocess.run",
    "open(path, 'r').read()"
]

# ✅ 正确做法：ACI 设计
tools = [
    "search_class(class_name)",      # 搜索类定义
    "search_method(method_name)",    # 搜索方法
    "open_file(path, line_range)",   # 打开文件（带 token 预算）
    "edit_file(path, old, new)",     # 搜索-替换格式编辑
    "run_command(cmd, timeout)",     # 执行命令（带超时）
]
```

**ACI 设计原则**:
1. **动作空间精简**: 不给 Agent 过多工具选择
2. **反馈即时性**: 每个动作都有明确结果
3. **错误可恢复**: 失败时提供修复建议
4. **上下文感知**: 输出考虑 token 预算

```python
class ACITool(ABC):
    """ACI 工具基类"""
    
    @abstractmethod
    async def execute(self, **kwargs) -> ToolResult:
        """执行工具"""
        pass
    
    def format_result(self, result: Any, token_budget: int = 2000) -> str:
        """格式化结果，考虑 token 预算"""
        formatted = str(result)
        if len(formatted) > token_budget:
            formatted = formatted[:token_budget] + "\n... [truncated]"
        return formatted

class ToolResult(BaseModel):
    """结构化工具结果"""
    success: bool
    output: str
    error_type: Optional[str] = None
    suggestion: Optional[str] = None  # 错误时的修复建议
```

---

#### 模式 2: 渐进式上下文构建

**来源**: AutoCodeRover、SWE-Agent、Cline、Aider

**核心思想**: 不一次性理解整个代码库，增量方式逐步构建上下文

```
渐进式构建流程：
1. Initial Scan  → Repository Map（PageRank 排序的关键符号）
2. Locate        → AST 搜索定位问题区域
3. Deep Dive     → 浏览相关文件和上下文
4. Refine        → 缩小范围，精确到需要修改的代码
```

```python
class ProgressiveContextBuilder:
    """渐进式上下文构建器"""
    
    def __init__(
        self,
        repo_map: RepositoryMap,      # Aider 风格
        ast_index: ASTIndex,          # AutoCodeRover 风格
        token_budget: int = 8000,
    ):
        self.repo_map = repo_map
        self.ast_index = ast_index
        self.token_budget = token_budget
    
    async def build_context(
        self,
        query: str,
        phase: str = "auto",
    ) -> Context:
        """构建上下文"""
        
        # Phase 1: 获取 Repository Map
        map_context = self.repo_map.get_relevant(
            query=query,
            token_budget=self.token_budget * 0.2,  # 20% 给 map
        )
        
        # Phase 2: AST 搜索定位
        ast_results = await self.ast_index.search(
            query=query,
            search_type="semantic",
        )
        
        # Phase 3: 浏览相关文件
        file_contents = await self._browse_files(
            files=ast_results.files,
            token_budget=self.token_budget * 0.6,  # 60% 给文件内容
        )
        
        # Phase 4: 组装上下文
        return Context(
            repo_map=map_context,
            search_results=ast_results,
            file_contents=file_contents,
            token_usage=self._count_tokens(map_context, file_contents),
        )
```

---

#### 模式 3: 自我修正循环

**来源**: Cline、SWE-Agent、OpenHands

**核心思想**: Agent 执行后检查结果，自动尝试修正

```python
class SelfCorrectionLoop:
    """自我修正循环"""
    
    def __init__(self, max_corrections: int = 3):
        self.max_corrections = max_corrections
    
    async def execute_with_correction(
        self,
        agent: Agent,
        task: str,
    ) -> str:
        """执行任务，支持自动修正"""
        
        for attempt in range(self.max_corrections + 1):
            # 执行任务
            result = await agent.execute(task)
            
            # 检查结果
            check_result = await self._check_result(result)
            
            if check_result.success:
                return result
            
            if attempt < self.max_corrections:
                # 构建修正提示
                correction_prompt = f"""
之前的执行失败了：
- 错误类型: {check_result.error_type}
- 错误信息: {check_result.error_message}
- 修复建议: {check_result.suggestion}

请根据以上信息修正你的操作。
"""
                task = correction_prompt
            else:
                return f"达到最大修正次数 ({self.max_corrections}): {result}"
    
    async def _check_result(self, result: str) -> CheckResult:
        """检查执行结果"""
        # 检查 linter 错误
        # 检查测试失败
        # 检查运行时错误
        pass
```

---

### 7.2 P1 重要模式

#### 模式 4: YAML 配置驱动

**来源**: SWE-Agent、ChatDev、OpenHands、Continue

```yaml
# egg.yaml - Agent 配置文件
name: code-reviewer
version: 1.0.0

model:
  provider: openai
  name: gpt-4
  temperature: 0.7

tools:
  - name: search_class
    description: 搜索类定义
  - name: search_method
    description: 搜索方法定义
  - name: edit_file
    description: 编辑文件
  - name: run_tests
    description: 运行测试

system_prompt: |
  你是一个代码审查专家。
  请仔细审查代码并提供改进建议。

max_iterations: 30
max_corrections: 3

environment:
  sandbox: docker
  image: python:3.11
```

---

#### 模式 5: 沙箱执行环境

**来源**: OpenHands、SWE-Agent

```python
class Sandbox(ABC):
    """沙箱基类"""
    
    @abstractmethod
    async def execute(self, command: str) -> SandboxResult:
        """在沙箱中执行命令"""
        pass
    
    @abstractmethod
    async def write_file(self, path: str, content: str) -> None:
        """写入文件"""
        pass
    
    @abstractmethod
    async def read_file(self, path: str) -> str:
        """读取文件"""
        pass

class DockerSandbox(Sandbox):
    """Docker 沙箱"""
    
    def __init__(self, image: str = "python:3.11"):
        self.image = image
    
    async def execute(self, command: str) -> SandboxResult:
        # 在 Docker 容器中执行
        pass

class LocalSandbox(Sandbox):
    """本地沙箱（受限）"""
    
    def __init__(self, allowed_paths: list[str]):
        self.allowed_paths = allowed_paths
```

---

#### 模式 6: Human-in-the-Loop

**来源**: Cline、OpenHands

```python
class ApprovalLevel(Enum):
    """审批级别"""
    ALWAYS = "always"           # 总是询问
    READ_ONLY = "read_only"     # 只读操作自动批准
    WRITE_REVIEW = "write_review"  # 写操作需要审查
    FULL_AUTO = "full_auto"     # 全自动

class HumanInTheLoop:
    """人机协作控制器"""
    
    def __init__(self, approval_level: ApprovalLevel):
        self.approval_level = approval_level
    
    async def check_approval(
        self,
        tool_name: str,
        tool_args: dict,
        risk_level: str,
    ) -> bool:
        """检查是否需要人工审批"""
        
        if self.approval_level == ApprovalLevel.FULL_AUTO:
            return True
        
        if self.approval_level == ApprovalLevel.READ_ONLY:
            return tool_name not in ["edit_file", "write_file", "run_command"]
        
        # 需要人工审批
        return await self._request_approval(tool_name, tool_args)
```

---

#### 模式 7: 多阶段规划执行

**来源**: Devin、Copilot Workspace

```python
class Phase(Enum):
    """执行阶段"""
    UNDERSTAND = "understand"  # 理解需求
    PLAN = "plan"             # 制定计划
    EXECUTE = "execute"       # 执行计划
    VERIFY = "verify"         # 验证结果

class PhasePipeline:
    """阶段管道"""
    
    def __init__(self):
        self.phases: dict[Phase, PhaseHandler] = {}
    
    async def run(self, input_data: dict) -> dict:
        """执行完整流程"""
        context = input_data.copy()
        
        for phase in Phase:
            handler = self.phases.get(phase)
            if handler:
                context = await handler.execute(context)
                
                # Phase 级别的检查点
                await self._save_checkpoint(phase, context)
        
        return context

class PlanPhase(PhaseHandler):
    """计划阶段"""
    
    async def execute(self, context: dict) -> dict:
        plan = await self.llm.generate_plan(context["issue"])
        
        # 计划应该是可审查的
        return {
            **context,
            "plan": plan,
            "plan_approved": await self.human.review_plan(plan),
        }
```

---

### 7.3 P2 增强模式

#### 模式 8: 检查点与回滚

**来源**: Cline、SWE-Agent

```python
class CheckpointManager:
    """检查点管理器"""
    
    def __init__(self, storage: StorageBackend):
        self.storage = storage
    
    async def create(self, name: str, state: dict) -> str:
        """创建检查点"""
        checkpoint_id = str(uuid.uuid4())
        
        checkpoint = {
            "id": checkpoint_id,
            "name": name,
            "timestamp": time.time(),
            "state": state,
            "file_hashes": await self._hash_files(),
        }
        
        await self.storage.set(f"checkpoint:{checkpoint_id}", checkpoint)
        return checkpoint_id
    
    async def restore(self, checkpoint_id: str) -> dict:
        """恢复检查点"""
        checkpoint = await self.storage.get(f"checkpoint:{checkpoint_id}")
        
        # 恢复文件状态
        await self._restore_files(checkpoint["state"]["files"])
        
        return checkpoint["state"]
    
    async def compare(self, id1: str, id2: str) -> dict:
        """比较两个检查点"""
        pass
```

---

#### 模式 9: MCP 工具扩展

**来源**: Cline、AgentScope

```python
class MCPClient:
    """MCP 客户端"""
    
    def __init__(self):
        self.servers: dict[str, MCPServer] = {}
    
    async def connect(self, server_config: dict) -> None:
        """连接 MCP 服务器"""
        server = MCPServer(server_config)
        await server.connect()
        self.servers[server.name] = server
    
    async def list_tools(self) -> list[dict]:
        """列出所有可用工具"""
        tools = []
        for server in self.servers.values():
            tools.extend(await server.list_tools())
        return tools
    
    async def call_tool(self, name: str, arguments: dict) -> Any:
        """调用工具"""
        for server in self.servers.values():
            if name in await server.list_tools():
                return await server.call_tool(name, arguments)
        raise ToolNotFoundError(name)
```

---

#### 模式 10: 观测性与轨迹记录

**来源**: SWE-Agent、OpenHands、AgentScope

```python
class TrajectoryRecorder:
    """轨迹记录器"""
    
    def __init__(self, output_dir: str):
        self.output_dir = output_dir
        self.steps: list[dict] = []
    
    async def record_step(self, step: dict) -> None:
        """记录一步"""
        self.steps.append({
            **step,
            "timestamp": time.time(),
            "step_number": len(self.steps),
        })
    
    async def save(self, filename: str) -> None:
        """保存轨迹"""
        trajectory = {
            "steps": self.steps,
            "metadata": {
                "total_steps": len(self.steps),
                "start_time": self.steps[0]["timestamp"] if self.steps else None,
                "end_time": self.steps[-1]["timestamp"] if self.steps else None,
            },
        }
        
        path = os.path.join(self.output_dir, filename)
        with open(path, "w") as f:
            json.dump(trajectory, f, indent=2)
```

---

### 7.4 P3 高级模式

#### 模式 11: 多 Agent 角色协作

**来源**: MetaGPT、ChatDev

```python
class Role(BaseModel):
    """Agent 角色"""
    name: str
    system_prompt: str
    tools: list[str]
    subscriptions: list[str]  # 订阅的消息类型

class Team:
    """团队编排器"""
    
    def __init__(self):
        self.members: dict[str, Agent] = {}
        self.topology: str = "chain"  # chain | star | mesh
    
    async def execute(self, task: str) -> str:
        """执行团队任务"""
        if self.topology == "chain":
            return await self._chain_execute(task)
        elif self.topology == "star":
            return await self._star_execute(task)
```

---

#### 模式 12: 消息中心通信

**来源**: AgentScope

```python
class MsgHub:
    """消息中心"""
    
    def __init__(self):
        self.participants: dict[str, Agent] = {}
        self.message_history: list[Message] = []
    
    async def broadcast(self, message: Message) -> None:
        """广播消息"""
        self.message_history.append(message)
        
        for agent in self.participants.values():
            if self._should_deliver(agent, message):
                await agent.receive(message)
    
    def add_participant(self, agent: Agent) -> None:
        """添加参与者"""
        self.participants[agent.name] = agent
    
    def remove_participant(self, name: str) -> None:
        """移除参与者"""
        self.participants.pop(name, None)
```

---

## 八、优先级总结

| 优先级 | 模式 | 状态 |
|--------|------|------|
| **P0** | ACI 工具设计 | 待实现 |
| **P0** | 渐进式上下文构建 | 待实现 |
| **P0** | 自我修正循环 | 待实现 |
| **P1** | YAML 配置驱动 | 待实现 |
| **P1** | 沙箱执行环境 | 待实现 |
| **P1** | Human-in-the-Loop | 待实现 |
| **P1** | 多阶段规划执行 | 待实现 |
| **P2** | 检查点与回滚 | 待实现 |
| **P2** | MCP 工具扩展 | 待实现 |
| **P2** | 观测性与轨迹记录 | 待实现 |
| **P3** | 多 Agent 角色协作 | 待实现 |
| **P3** | 消息中心通信 | 待实现 |
| **P3** | 工具自生成 | 待实现 |

---

**文档状态**: 已更新  
**下一步**: 确认方案后开始 Phase 1 开发
