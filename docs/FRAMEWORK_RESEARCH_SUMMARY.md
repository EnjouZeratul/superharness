# Agent 框架研究报告

> 版本: v2.0
> 日期: 2026-05-09
> 目标: 为 Continuum 提供设计参考
> 研究范围: 12 个主流 Agent 框架

---

## 框架总览

| 框架 | 核心贡献 | 许可证 | 推荐借鉴 |
|------|----------|--------|----------|
| **SWE-agent** | ACI 工具设计 | MIT | ★★★★★ |
| **Aider** | Repository Map + Token 预算 | Apache-2.0 | ★★★★★ |
| **Cline** | Hooks + MCP | Apache-2.0 | ★★★★★ |
| **LangGraph** | StateGraph 工作流 | MIT | ★★★★☆ |
| **MetaGPT** | 多 Agent SOP | MIT | ★★★★☆ |
| **OpenHands** | Docker 沙箱 | MIT | ★★★☆☆ |
| **AutoGen** | Actor 模型 + Handoff | MIT | ★★★★☆ |
| **AutoGPT** | 多规划策略 | MIT | ★★★★☆ |
| **CrewAI** | Crew + Task + Guardrail | MIT | ★★★★☆ |
| **Semantic Kernel** | 过滤器中间件 | MIT | ★★★☆☆ |
| **LlamaIndex** | RAG 索引系统 | MIT | ★★★☆☆ |
| **LangChain** | Runnable 抽象 | MIT | ★★☆☆☆ (避免使用) |

---

## 一、SWE-agent: Agent-Computer Interface

### 1.1 核心贡献

**ACI (Agent-Computer Interface)** 是 SWE-agent 最重要的概念: 工具应该专门为 LLM Agent 设计，而非简单地暴露系统 API。

### 1.2 ACI 工具设计原则

```python
class ACITool:
    """Agent-Computer Interface 工具"""

    # 1. 简洁的输出格式
    async def search_files(self, pattern: str) -> str:
        """返回结构化、易解析的文本"""
        results = glob.glob(pattern)
        return "\n".join(f"[{i}] {r}" for i, r in enumerate(results))

    # 2. 内置错误恢复
    async def edit_file(self, path: str, content: str) -> str:
        """自动创建备份，支持回滚"""
        backup = f"{path}.bak"
        shutil.copy(path, backup)
        try:
            with open(path, 'w') as f:
                f.write(content)
            return f"Success. Backup: {backup}"
        except Exception as e:
            shutil.copy(backup, path)  # 回滚
            return f"Error: {e}. Rolled back."

    # 3. 防护栏
    async def run_command(self, cmd: str) -> str:
        """限制危险操作"""
        blocked = ["rm -rf", "sudo", "chmod"]
        if any(b in cmd for b in blocked):
            return "Error: Command blocked for safety"
```

### 1.3 YAML 配置驱动

```yaml
# agent.yaml
agent:
  templates:
    system_template: "You are a helpful assistant..."
  tools:
    - name: search_file
      signature: "search_file <pattern> [--path <path>]"
      timeout: 30
  error_handling:
    max_retries: 3
    auto_submit_on_error: true
```

---

## 二、Aider: Repository Map + Token 预算

### 2.1 Repository Map (PageRank)

Aider 使用 PageRank 算法计算代码重要性:

```python
class RepoMap:
    def build_graph(self):
        """构建依赖图"""
        for file in self.repo_root.rglob("*.py"):
            imports = self._parse_imports(file)
            for imp in imports:
                self.graph.add_edge(file, imp, weight=1)

    def get_ranking(self) -> dict[str, float]:
        """PageRank 排序"""
        return nx.pagerank(self.graph)

    def get_context_map(self, max_tokens: int) -> str:
        """生成上下文地图"""
        ranked = sorted(self.get_ranking().items(), key=lambda x: x[1], reverse=True)
        result = []
        current_tokens = 0
        for file, score in ranked:
            if current_tokens + self._count_tokens(file) > max_tokens:
                break
            result.append(self._format_file(file, score))
        return "\n".join(result)
```

### 2.2 Token 预算二分搜索

```python
class TokenBudgetManager:
    def find_max_context(self, messages: list) -> int:
        """二分搜索最大可用上下文"""
        low, high = 0, self.max_tokens
        while low < high:
            mid = (low + high + 1) // 2
            truncated = self._truncate(messages, mid)
            try:
                response = await self.llm.chat(truncated)
                low = mid  # 成功，尝试更大
            except ContextTooLongError:
                high = mid - 1  # 失败，减小
        return low
```

---

## 三、Cline: Hooks + MCP + Human-in-the-Loop

### 3.1 Hooks 生命周期系统

```python
class HookType(Enum):
    BEFORE_TOOL = "before_tool"
    AFTER_TOOL = "after_tool"
    ON_ERROR = "on_error"
    BEFORE_LLM = "before_llm"
    AFTER_LLM = "after_llm"
    ON_CONTEXT_COMPACT = "on_context_compact"

class PermissionHook(Hook):
    @property
    def hook_type(self) -> HookType:
        return HookType.BEFORE_TOOL

    async def execute(self, context: HookContext) -> Optional[HookContext]:
        if context.tool_name in self.blocked_tools:
            return None  # 阻止执行
        return context
```

### 3.2 Human-in-the-Loop 三级权限

```python
class ApprovalLevel(Enum):
    AUTO = "auto"              # 自动执行
    AUTO_WITH_PLAN = "auto_with_plan"  # 规划后自动
    ASK = "ask"                # 每次询问
```

### 3.3 MCP 集成

```python
class MCPClient:
    async def connect(self, name: str, command: str, args: list[str]):
        """连接 MCP 服务器"""
        self.process = await asyncio.create_subprocess_exec(
            command, *args, stdin=PIPE, stdout=PIPE
        )
        await self._send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "clientInfo": {"name": "continuum", "version": "1.0"}
        })
```

---

## 四、LangGraph: StateGraph 工作流

### 4.1 StateGraph 核心设计

```python
from typing import TypedDict, Annotated
import operator

class WorkflowState(TypedDict):
    messages: Annotated[list[Message], operator.add]
    current_step: str
    results: dict

class StateGraph:
    def add_conditional_edge(self, source: str, condition: Callable, targets: dict):
        """条件分支"""
        for value, target in targets.items():
            self.edges.append(ConditionalEdge(source, target,
                lambda s, v=value: condition(s) == v))

    def add_parallel(self, source: str, targets: list[str]):
        """并行执行"""
        for target in targets:
            self.edges.append(Edge(source, target, parallel=True))
```

### 4.2 Pregel BSP 执行

```python
class PregelExecutor:
    async def execute(self, graph: StateGraph, initial_state: dict):
        """BSP (Bulk Synchronous Parallel) 执行"""
        state = initial_state.copy()
        for superstep in range(self.max_supersteps):
            active = self._get_active_nodes(graph, state)
            if not active: break
            results = await asyncio.gather(*[node.run(state) for node in active])
            for result in results:
                self._merge_state(state, result)
        return state
```

---

## 五、MetaGPT: 多 Agent SOP

### 5.1 Role 设计

```python
class Role:
    def __init__(self, name: str, profile: str):
        self.name = name
        self.profile = profile
        self.actions: list[Action] = []
        self.observe: list[str] = []  # 订阅的消息类型

    async def run(self, message: Message) -> Message:
        """主循环: think -> act"""
        context = await self.think({"message": message})
        return await self.act(context)
```

### 5.2 消息路由 (cause_by)

```python
class Environment:
    async def publish(self, message: Message):
        """发布消息到环境"""
        for role in self.roles.values():
            if message.cause_by in role.observe:
                await role.run(message)
            elif role.name in message.sent_to:
                await role.run(message)
```

### 5.3 ActionNode 结构化输出

```python
class ActionNode:
    def __init__(self, name: str, schema: dict):
        self.schema = schema  # JSON Schema

    async def run(self, context: dict) -> dict:
        response = await self.llm.chat(self._build_prompt(context))
        return self._parse_structured(response, self.schema)
```

---

## 六、OpenHands: Docker 沙箱 + 事件流

### 6.1 Docker 沙箱

```python
class DockerSandbox:
    async def start(self):
        self.container = await docker.containers.run(
            self.image, detach=True, working_dir="/workspace",
            cpu_quota=50000,  # 50% CPU
            mem_limit="1g",
        )

    async def execute(self, command: str) -> ExecutionResult:
        result = await self.container.exec(command)
        return ExecutionResult(exit_code=result.exit_code, stdout=result.output)
```

### 6.2 Agent Runtime 状态机

```python
class AgentStatus(Enum):
    IDLE = "idle"
    THINKING = "thinking"
    ACTING = "acting"
    WAITING = "waiting"
    ERROR = "error"
```

---

## 七、AutoGen: Actor 模型 + Handoff (微软)

### 7.1 核心: Actor 模型

```python
class RoutedAgent(BaseAgent):
    @event
    async def handle_event(self, message: Message, ctx: MessageContext):
        """处理事件消息（无需返回）"""

    @rpc
    async def handle_rpc(self, message: Request, ctx: MessageContext) -> Response:
        """处理 RPC 消息（需要返回响应）"""
```

### 7.2 四种 Team 协作模式

| 模式 | 选择机制 | 适用场景 |
|------|----------|----------|
| **RoundRobinGroupChat** | 轮询 | 简单顺序协作 |
| **SelectorGroupChat** | LLM + 选择器函数 | 动态智能调度 |
| **Swarm** | Handoff 消息驱动 | 串联工作流 |
| **MagenticOneGroupChat** | 编排器协调 | 复杂多步骤任务 |

### 7.3 Handoff 机制 (独特创新)

```python
class Handoff(BaseModel):
    target: str       # 目标 Agent 名称
    message: str      # 移交消息
    description: str  # 触发条件描述

    @property
    def handoff_tool(self) -> BaseTool:
        """自动生成 Handoff 工具"""
        return FunctionTool(_handoff_tool, name=self.name, description=self.description)
```

### 7.4 声明式配置

```yaml
provider: autogen_agentchat.teams.SelectorGroupChat
config:
  participants:
    - provider: autogen_agentchat.agents.AssistantAgent
      config:
        name: Travel_Advisor
        model_client: { provider: autogen_ext.models.openai.OpenAIChatCompletionClient, config: { model: "gpt-4o" } }
  termination_condition:
    provider: autogen_agentchat.conditions.TextMentionTermination
    config: { text: "TERMINATE" }
```

---

## 八、AutoGPT: 多规划策略

### 8.1 三种核心规划策略

| 策略 | 来源 | 特点 |
|------|------|------|
| **One-Shot** | 默认 | 每次独立决策 |
| **Plan-and-Execute** | 经典模式 | 先规划再执行 |
| **ReWOO** | arXiv | 提前规划 + 变量占位符 |

### 8.2 ReWOO 策略 (独特)

```python
class ReWOOPhase(str, Enum):
    PLANNING = "planning"      # 生成带变量占位符的完整计划
    EXECUTING = "executing"    # 执行所有工具
    SYNTHESIZING = "synthesizing"  # 合成最终结果

# 计划格式
Plan: First, list files.
#E1 = list_folder(folder=".")
Plan: Read the main file.
#E2 = read_file(filename="main.py")
Plan: Write the solution.
#E3 = write_to_file(filename="solution.txt", contents="#E2 result")
```

### 8.3 组件系统 (协议驱动)

```python
class DirectiveProvider(AgentComponent):
    def get_constraints(self) -> Iterator[str]: return iter([])
    def get_resources(self) -> Iterator[str]: return iter([])

class CommandProvider(AgentComponent):
    @abstractmethod
    def get_commands(self) -> Iterator["Command"]: ...

# 拓扑排序执行
def _topological_sort(self, components):
    """根据 run_after 依赖关系排序组件"""
```

---

## 九、CrewAI: Crew + Task + Guardrail

### 9.1 Crew 团队协作

```python
class Crew(BaseModel):
    tasks: list[Task]
    agents: list[BaseAgent]
    process: Process  # sequential | hierarchical
    manager_llm: BaseLLM  # hierarchical 模式必需

class Process(str, Enum):
    sequential = "sequential"      # 顺序执行
    hierarchical = "hierarchical"  # Manager 模式
```

### 9.2 Task 定义

```python
class Task(BaseModel):
    description: str
    expected_output: str
    agent: BaseAgent | None
    context: list[Task]        # 上下文依赖
    output_pydantic: type[BaseModel]  # 结构化输出
    guardrail: GuardrailType   # 输出验证器
    guardrail_max_retries: int
```

### 9.3 Guardrail 验证机制

```python
def validate_output(output: TaskOutput) -> Tuple[bool, str]:
    if len(output.raw) < 100:
        return (False, "Output too short")
    return (True, output.raw)

task = Task(
    description="Write a summary",
    guardrail=validate_output,
    guardrail_max_retries=3
)
```

### 9.4 Flow 工作流

```python
class MyFlow(Flow[dict]):
    @start()
    def begin(self): return "started"

    @listen("begin")
    def process(self): return "processed"

    @router("process")
    def route(self):
        if self.state["status"] == "success":
            return "SUCCESS"
        return "FAILURE"
```

---

## 十、Semantic Kernel: 过滤器中间件 (微软)

### 10.1 Kernel 核心

```csharp
public sealed class Kernel
{
    public IServiceProvider Services { get; }
    public KernelPluginCollection Plugins { get; }
    public IList<IFunctionInvocationFilter> FunctionInvocationFilters { get; }
    public IList<IPromptRenderFilter> PromptRenderFilters { get; }
    public Kernel Clone() { ... }  // 支持隔离执行
}
```

### 10.2 过滤器中间件模式

```csharp
public interface IFunctionInvocationFilter
{
    Task OnFunctionInvocationAsync(
        FunctionInvocationContext context,
        Func<FunctionInvocationContext, Task> next);
}

// 递归调用实现中间件链
private static async Task InvokeFilterOrFunctionAsync(
    NonNullCollection<IFunctionInvocationFilter> filters,
    Func<FunctionInvocationContext, Task> functionCallback,
    FunctionInvocationContext context,
    int index = 0)
{
    if (filters != null && index < filters.Count)
        await filters[index].OnFunctionInvocationAsync(context,
            (ctx) => InvokeFilterOrFunctionAsync(filters, functionCallback, ctx, index + 1));
    else
        await functionCallback(context);
}
```

### 10.3 FunctionChoiceBehavior

```csharp
[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
[JsonDerivedType(typeof(AutoFunctionChoiceBehavior), typeDiscriminator: "auto")]
[JsonDerivedType(typeof(RequiredFunctionChoiceBehavior), typeDiscriminator: "required")]
[JsonDerivedType(typeof(NoneFunctionChoiceBehavior), typeDiscriminator: "none")]
public abstract class FunctionChoiceBehavior
```

---

## 十一、LlamaIndex: RAG 索引系统

### 11.1 Index 核心架构

```python
class BaseIndex(Generic[IS], ABC):
    index_struct_cls: Type[IS]  # 索引结构类型

    def __init__(
        self,
        nodes: Optional[Sequence[BaseNode]] = None,
        storage_context: Optional[StorageContext] = None,
    ): ...

# 主要索引类型
VectorStoreIndex     # 向量相似度检索
SummaryIndex         # 文档摘要
PropertyGraphIndex   # 属性图谱（新版推荐）
```

### 11.2 QueryEngine 查询流程

```python
class RetrieverQueryEngine(BaseQueryEngine):
    def __init__(
        self,
        retriever: BaseRetriever,
        response_synthesizer: BaseSynthesizer,
        node_postprocessors: List[BaseNodePostprocessor],
    ): ...

    def _query(self, query_bundle: QueryBundle):
        nodes = self.retrieve(query_bundle)  # 1. 检索
        response = self._response_synthesizer.synthesize(query, nodes)  # 2. 综合
        return response
```

### 11.3 ResponseMode

```python
class ResponseMode(str, Enum):
    REFINE = "refine"              # 迭代精炼
    COMPACT = "compact"            # 压缩后精炼
    TREE_SUMMARIZE = "tree_summarize"  # 树形摘要
    ACCUMULATE = "accumulate"      # 逐块响应后拼接
```

---

## 十二、LangChain: Runnable 抽象 (避免使用)

### 12.1 Runnable 抽象 (唯一亮点)

```python
class Runnable(ABC, Generic[Input, Output]):
    def invoke(self, input: Input) -> Output
    async def ainvoke(self, input: Input) -> Output
    def batch(self, inputs: list[Input]) -> list[Output]
    def stream(self, input: Input) -> Iterator[Output]

    # 组合操作符
    def __or__(self, other) -> "RunnableSequence":  # | 操作符

    # 装饰器方法
    def with_retry(self, **kwargs) -> "RunnableRetry"
    def with_fallbacks(self, fallbacks: list) -> "RunnableWithFallbacks"
```

### 12.2 为什么避免使用 LangChain

| 问题 | 说明 |
|------|------|
| **过深继承** | `Chain -> RunnableSerializable -> Runnable -> ABC` |
| **大量弃用** | 几乎所有 `langchain_classic` 都标记为弃用 |
| **Memory 缺陷** | 不支持原生 tool calling |
| **字典传递** | 编译器无法检查键是否存在 |
| **性能开销** | 每层都有回调、配置、验证 |
| **版本分裂** | v0.x/v1.x/core/protocol 混乱 |

### 12.3 可借鉴的设计

- Runnable 管道组合语法
- `@tool` 装饰器的 schema 自动推断
- 回调和追踪系统

---

## 十三、设计模式总结

### 13.1 推荐模式

| 模式 | 来源 | 适用场景 |
|------|------|----------|
| **ACI 工具设计** | SWE-agent | 所有工具定义 |
| **Hooks 中间件** | Cline / Semantic Kernel | 生命周期扩展 |
| **StateGraph 工作流** | LangGraph | 复杂流程编排 |
| **Handoff 交接** | AutoGen | Agent 间控制权转移 |
| **Guardrail 验证** | CrewAI | 输出质量控制 |
| **ReWOO 规划** | AutoGPT | 提前规划 + 并行执行 |
| **Repository Map** | Aider | 大型代码库理解 |

### 13.2 避免模式

| 反模式 | 问题 | 替代方案 |
|--------|------|----------|
| 过深继承 | 理解成本高 | 组合优于继承 |
| 字典传递数据 | 类型不安全 | Pydantic 模型 |
| 隐式状态 | 难以追踪 | 显式状态机 |
| 魔法方法 | 意外行为 | 明确 API |

---

## 十四、Continuum 技术选型

### 14.1 核心依赖

```
httpx + pydantic (无 LangChain)
```

### 14.2 关键设计决策

| 功能 | 选择 | 参考 |
|------|------|------|
| 工具系统 | ACI 设计 + MCP | SWE-agent + Cline |
| 生命周期 | Hooks 中间件 | Cline + Semantic Kernel |
| 工作流 | StateGraph | LangGraph |
| 多 Agent | Role + Handoff | MetaGPT + AutoGen |
| Token 管理 | 二分搜索预算 | Aider |
| 输出验证 | Guardrail | CrewAI |
| 规划策略 | ReWOO / Plan-Execute | AutoGPT |

### 14.3 配置驱动

```yaml
# super.yaml (参考 SWE-agent + AutoGen)
model:
  provider: openai
  name: gpt-4-turbo

agent:
  planning_strategy: rewoo  # one_shot | plan_execute | rewoo
  max_iterations: 30
  hooks:
    - logging
    - metrics
    - permission

tools:
  builtin: [file_ops, shell, search]
  mcp:
    - name: filesystem
      command: uvx
      args: [mcp-server-filesystem, .]

memory:
  project: EGG.md
  auto: true

workflow:
  type: state_graph
  checkpoint: .continuum/checkpoints
```

---

**研究状态**: v2.0 已完成 (12 个框架)
**下一步**: 应用到 Continuum 开发
