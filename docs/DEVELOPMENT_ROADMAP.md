# Continuum 开发路线图

> 版本: v3.0
> 日期: 2026-05-09
> 基于: 12 大 Agent 框架研究成果 + SPEC.md + 六位专家评审

---

## 〇、Phase 0: 项目初始化 (Week 1)

**目标**: 建立项目基础设施

| 任务 | 说明 | 优先级 |
|------|------|--------|
| **项目结构** | 创建标准 Python 项目结构 | P0 |
| **pyproject.toml** | 配置依赖、构建工具 | P0 |
| **CI/CD** | GitHub Actions 基础配置 | P1 |
| **代码规范** | ruff/black/mypy 配置 | P0 |
| **文档结构** | docs/ 目录初始化 | P1 |
| **LICENSE** | Apache-2.0 许可证 | P0 |

**交付物**:
- 可 clone 的项目骨架
- 基础 CI 流程
- 开发环境配置说明

---

## 一、"Super" 品牌定位

### 1.0 品牌语义

**"Super" 的三重含义**:

```
Super = Superior（超越）+ Supersonic（极速）+ Superstructure（超级架构）

具体体现：
- Superior：能力超越基础框架（企业级功能）
- Supersonic：执行效率超越（并行执行、智能调度）
- Superstructure：架构设计超越（分层设计、可扩展）
```

### 1.1 核心卖点（匹配 "Super" 品牌）

| 卖点 | "Super" 关联 | 一句话价值主张 |
|------|--------------|----------------|
| **超级控制力** | Super Control | 对 Agent 执行的超级控制能力 |
| **超级透明度** | Super Visibility | 比任何框架都更透明的执行过程 |
| **超级稳定性** | Super Reliability | 企业级生产稳定性保证 |
| **超级效率** | Super Efficiency | 并行执行、智能调度 |
| **超级扩展性** | Super Extensibility | MCP 标准化工具扩展 |
| **超级学习能力** | Super Learning | 项目记忆 + 跨会话学习 |

### 1.2 与 SmolAgents 的关系

```
SmolAgents：极简框架
- 核心代码约1000行
- 设计哲学：简洁、透明、最小化抽象
- HuggingFace 官方定位：适用于研究和生产环境

Continuum：企业级框架
- 设计目标：完整的企业级基础设施
- 核心能力：可观测性、成本控制、错误恢复

两者关系：设计哲学不同，各有其价值定位
```

**一句话定位**:
> **"Continuum 是简洁可靠的 Agent 运行时"**

---

## 二、框架研究总结

### 2.1 研究来源 (12 个框架)

| 框架 | 核心贡献 | 可借鉴点 |
|------|----------|----------|
| **SWE-agent** | ACI (Agent-Computer Interface) | 工具设计哲学、YAML 配置驱动、错误分类 |
| **Aider** | Repository Map | PageRank 代码重要性、Token 预算二分搜索 |
| **Cline** | Hooks + MCP | 生命周期钩子、权限控制、MCP 集成 |
| **LangGraph** | StateGraph | 工作流编排、条件分支、并行执行 |
| **MetaGPT** | 多 Agent SOP | 角色协作、消息路由、ActionNode 结构化输出 |
| **OpenHands** | 沙箱执行 | Docker 隔离、资源限制、事件流 |
| **AutoGen** | Actor 模型 | Handoff 交接、声明式配置、Team 协作模式 |
| **AutoGPT** | 多规划策略 | ReWOO、Plan-Execute、组件系统拓扑排序 |
| **CrewAI** | Crew + Task | Guardrail 验证、Flow 工作流、结构化输出 |
| **Semantic Kernel** | 过滤器中间件 | 函数调用拦截、Kernel.Clone 隔离 |
| **LlamaIndex** | RAG 索引 | QueryEngine、ResponseMode、向量存储抽象 |
| **LangChain** | Runnable 抽象 | 管道组合语法（但整体避免使用） |

### 2.2 核心设计决策

基于研究，Continuum 采用以下核心设计：

```
┌─────────────────────────────────────────────────────────────────┐
│                      Continuum 架构                          │
├─────────────────────────────────────────────────────────────────┤
│  1. 轻量依赖: httpx + pydantic (参考 Aider/SWE-agent)          │
│  2. ACI 工具设计: 为 Agent 优化的工具接口 (参考 SWE-agent)       │
│  3. Hooks 系统: 生命周期扩展点 (参考 Cline)                      │
│  4. StateGraph 工作流: 状态图编排 (参考 LangGraph)               │
│  5. Memory System: 项目记忆 + 自动学习 (参考 Claude Code)         │
│  6. MCP 支持: 标准化工具扩展 (参考 Cline/OpenHands)              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、开发阶段规划

### Phase 1: 核心基础 (Week 2-5)

**目标**: 建立最小可运行的核心

| 模块 | 任务 | 优先级 | 参考 |
|------|------|--------|------|
| **LLM Provider** | 实现 OpenAI/Anthropic 适配器 | P0 | Aider 直接调用模式 |
| **Error Handler** | 错误分类 + 重试执行器 | P0 | SWE-agent 错误模式 |
| **Token Budget** | 预算管理器 | P0 | Aider 二分搜索 |
| **Config System** | YAML + 环境变量 | P0 | SWE-agent 配置 |
| **Context Manager** | 消息管理 + 预算集成 | P0 | - |

**交付物**:
- 可调用 LLM 的基础框架
- 错误重试机制
- Token 预算管理

```python
# Phase 1 完成后的能力
from continuum import Harness

harness = Harness(config="super.yaml")
response = await harness.llm.chat("Hello")
# 自动重试、预算检查、错误分类
```

---

### Phase 2: Agent 运行时 (Week 6-8)

**目标**: 实现完整的 Agent 循环，体现"Super"核心价值

| 模块 | 任务 | 优先级 | 参考 |
|------|------|--------|------|
| **Agent Runtime** | Tool Calling Loop | P0 | SWE-agent 执行循环 |
| **Tool Registry** | 注册 + Schema 生成 | P0 | Aider 工具系统 |
| **Tool Validation** | 输入验证 + 超时 | P1 | - |
| **SuperTracer** | 超级透明度追踪 | P1 | SWE-agent 追踪 |
| **SuperMetrics** | 超级监控指标 | P1 | - |
| **内置工具** | 文件、Shell、搜索（ACI 设计） | P1 | SWE-agent ACI |

**交付物**:
- 完整的 Agent 执行循环（超级控制力）
- 工具注册和调用（ACI 设计）
- 可观测性基础（超级透明度）

```python
# Phase 2 完成后的能力 - 体现"Super"品牌价值
@harness.tool
async def search_files(pattern: str, path: str = ".") -> list[str]:
    """搜索文件 - ACI 设计，为 Agent 优化"""
    import glob
    import os
    safe_path = os.path.abspath(path)
    if not safe_path.startswith(WORKSPACE_ROOT):
        return ["Error: Cannot search outside workspace"]
    return glob.glob(os.path.join(safe_path, pattern))

agent = harness.create_agent(
    name="assistant",
    tools=["search_files"],
    tracer=ConsoleTracer(),  # 超级透明度
)

response = await agent.run("搜索 Python 文件")
# 自动追���、自动重试、错误分类 - 超级控制力与稳定性
```

---

### Phase 3: 核心扩展 (Week 9-12)

**目标**: 增强智能性和持久化

| 模块 | 任务 | 优先级 | 参考 |
|------|------|--------|------|
| **Memory System** | ProjectMemory (EGG.md) | P0 | Claude Code |
| **Hooks System** | BEFORE_TOOL/AFTER_TOOL | P0 | Cline |
| **Streaming** | 流式输出 | P1 | - |
| **Self-Correction** | 自动修正循环 | P2 | Cline + SWE-agent |
| **AutoMemory** | 跨会话学习 | P2 | Claude Code |
| **Context 摘要** | 智能压缩 | P2 | Aider |

**交付物**:
- 项目记忆系统 (ProjectMemory) - 超级学习能力
- Hooks 系统 (BEFORE_TOOL/AFTER_TOOL 等) - 超级控制力
- 流式输出支持 - 超级响应性

```python
# Phase 3 完成后的能力
# 项目记忆
project_memory = ProjectMemory("./")

# 钩子系统
hooks = HookSystem()
hooks.register(LoggingHook(logger))
hooks.register(PermissionHook(blocked_tools=["run_command"]))

agent = harness.create_agent(
    name="assistant",
    memory=project_memory,
    hooks=hooks,
)
```

---

### Phase 4: 高级特性 (Week 13-18)

**目标**: 工作流和多 Agent 能力

| 模块 | 任务 | 优先级 | 参考 |
|------|------|--------|------|
| **MCP Client** | 标准化工具扩展 | P0 | Cline/OpenHands |
| **多 Provider** | DashScope/ZhipuAI/DeepSeek | P1 | - |
| **Agent Planner** | 任务规划器 | P2 | - |
| **StateGraph** | 工作流引擎 | P2 | LangGraph |
| **Checkpoint** | 检查点管理 | P2 | LangGraph |

**交付物**:
- MCP 工具集成 - 超级扩展性
- 多模型支持 (除 OpenAI/Anthropic) - 超级兼容性
- 工作流编排能力 - 超级架构

```python
# Phase 4 完成后的能力
# MCP 集成
mcp = MCPClient()
await mcp.connect("filesystem", "uvx", ["mcp-server-filesystem", "."])

harness = Harness(mcp_client=mcp)
agent = harness.create_agent(
    tools=["read_file", "github_search_code"],  # 混合本地和 MCP 工具
)
```

---

### Phase 5: 集成与文档 (Week 19-20)

**目标**: 开源准备

| 任务 | 说明 |
|------|------|
| **CLI 接口** | 命令行工具 |
| **文档** | API 文档、使用指南、快速开始 |
| **示例** | 3-5 个完整示例代码 |
| **测试覆盖** | 单元测试 >80%，关键路径集成测试 |
| **CI/CD** | 完整 GitHub Actions 流程 |

---

## 里程碑与发布

### M0: 项目就绪 (Phase 0 结束)
- 项目骨架完成
- CI 流程可用
- 开发环境文档

### M1: Hello World (Phase 1 结束)
```python
harness = Harness()
response = await harness.llm.chat("Hello")
assert response.content
```

### M2: Tool Calling (Phase 2 结束) - MVP 发布

**"Super" 品牌价值体现**:
- **超级控制力**: 完整的工具生命周期管理
- **超级透明度**: 控制台追踪，执行过程完全可见
- **超级稳定性**: 错误分类 + 自动重试机制

```python
# 超级控制力: 完整的工具定义
@harness.tool
async def search_files(pattern: str, path: str = ".") -> list[str]:
    """搜索文件 - ACI 设计，为 Agent 优化"""
    safe_path = os.path.abspath(path)
    if not safe_path.startswith(WORKSPACE_ROOT):
        return ["Error: Cannot search outside workspace"]
    return glob.glob(os.path.join(safe_path, pattern))

# 超级透明度: 自动追踪
agent = harness.create_agent(
    tools=["search_files"],
    tracer=ConsoleTracer()  # 执行过程完全可见
)
response = await agent.run("Search for Python files")

# 超级稳定性: 自动错误重试
# 内置: 错误分类 + 智能重试 + 超时控制
```

### M3: Memory + Hooks (Phase 3 结束) - Beta 发布
```python
memory = ProjectMemory(".")
hooks = HookSystem()
hooks.register(LoggingHook())

agent = harness.create_agent(memory=memory, hooks=hooks)
response = await agent.run("What's in memory?")
```

### M4: MCP + Multi-Provider (Phase 4 结束)
```python
mcp = MCPClient()
await mcp.connect("filesystem", ...)

harness = Harness(mcp_client=mcp, provider="dashscope")
```

### M5: Open Source Ready (Phase 5 结束)
- 文档完整
- 测试覆盖 > 80%
- CLI 可用
- 示例丰富

## 四、关键技术实现细节

### 3.1 ACI 工具设计 (来自 SWE-agent)

SWE-agent 的核心洞察: **工具应该为 Agent 设计，而非暴露系统 API**

```python
# 错误示例: 暴露系统 API
@tool
async def execute_command(cmd: str) -> str:
    """执行任意命令 - Agent 容易滥用"""
    return subprocess.run(cmd, shell=True)

# 正确示例: ACI 设计
@tool
async def search_file(pattern: str, path: str = ".") -> list[str]:
    """搜索文件 - 专门为 Agent 优化的接口"""
    # 限制搜索范围
    safe_path = os.path.abspath(path)
    if not safe_path.startswith(WORKSPACE_ROOT):
        return ["Error: Cannot search outside workspace"]
    return glob.glob(os.path.join(safe_path, pattern))
```

### 3.2 Token 预算管理 (来自 Aider)

Aider 使用二分搜索确定最大可用上下文:

```python
class TokenBudgetManager:
    def find_max_context(self, messages: list) -> int:
        """二分搜索最大可用上下文"""
        low, high = 0, self.max_tokens

        while low < high:
            mid = (low + high + 1) // 2
            truncated = self._truncate(messages, mid)

            try:
                # 尝试实际调用
                response = await self.llm.chat(truncated)
                low = mid  # 成功，尝试更大的值
            except ContextTooLongError:
                high = mid - 1  # 失败，减小

        return low
```

### 3.3 Hooks 系统 (来自 Cline)

Cline 的钩子系统提供细粒度控制:

```python
class HookType(Enum):
    BEFORE_TOOL = "before_tool"
    AFTER_TOOL = "after_tool"
    ON_ERROR = "on_error"
    BEFORE_LLM = "before_llm"
    AFTER_LLM = "after_llm"
    ON_CONTEXT_COMPACT = "on_context_compact"

# 权限钩子示例
class PermissionHook(Hook):
    async def execute(self, context: HookContext) -> Optional[HookContext]:
        if context.tool_name in self.blocked_tools:
            return None  # 阻止执行
        return context
```

### 3.4 StateGraph 工作流 (来自 LangGraph)

LangGraph 的状态图设计:

```python
from typing import TypedDict

class WorkflowState(TypedDict):
    query: str
    results: list[str]
    analysis: str

workflow = StateGraph(WorkflowState)
workflow.add_node("search", search_node)
workflow.add_node("analyze", analyze_node)

# 条件分支
workflow.add_conditional_edge(
    "search",
    lambda s: "complex" if len(s["results"]) > 10 else "simple",
    {
        "complex": "detailed_analyze",
        "simple": "quick_analyze",
    }
)

# 并行执行
workflow.add_parallel("start", ["search", "fetch"])
```

### 3.5 多 Agent 协作 (来自 MetaGPT + AutoGen + CrewAI)

**MetaGPT Role 设计:**
```python
class Role:
    def __init__(self, name: str, profile: str):
        self.actions: list[Action] = []
        self.observe: list[str] = []  # 订阅的消息类型

    async def run(self, message: Message) -> Message:
        context = await self.think({"message": message})
        return await self.act(context)
```

**AutoGen Handoff 机制 (独特创新):**
```python
class Handoff(BaseModel):
    target: str       # 目标 Agent 名称
    message: str      # 移交消息
    description: str  # 触发条件描述

    @property
    def handoff_tool(self) -> BaseTool:
        """自动生成 Handoff 工具"""
        return FunctionTool(_handoff_tool, name=self.name)
```

**CrewAI Guardrail 验证:**
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

### 3.6 规划策略 (来自 AutoGPT)

| 策略 | 特点 | Token 效率 |
|------|------|------------|
| **One-Shot** | 每次独立决策 | 基准 |
| **Plan-Execute** | 先规划再执行 | - |
| **ReWOO** | 提前规划 + 变量占位符 + 并行执行 | 5x 提升 |

**ReWOO 计划格式:**
```
Plan: First, list files.
#E1 = list_folder(folder=".")
Plan: Read the main file.
#E2 = read_file(filename="main.py")
Plan: Write the solution.
#E3 = write_to_file(filename="solution.txt", contents="#E2 result")
```

### 3.7 过滤器中间件 (来自 Semantic Kernel)

```python
class IFunctionInvocationFilter(ABC):
    @abstractmethod
    async def on_function_invocation(
        self, context: FunctionInvocationContext, next: Callable
    ) -> None:
        pass

# 递归调用实现中间件链
async def invoke_filter_chain(filters, callback, context, index=0):
    if index < len(filters):
        await filters[index].on_function_invocation(context,
            lambda ctx: invoke_filter_chain(filters, callback, ctx, index + 1))
    else:
        await callback(context)
```

### 3.6 MCP 集成 (来自 Cline/OpenHands)

MCP (Model Context Protocol) 标准化工具扩展:

```python
class MCPClient:
    async def connect(self, name: str, command: str, args: list[str]):
        """连接 MCP 服务器"""
        # 启动子进程
        self.process = await asyncio.create_subprocess_exec(
            command, *args,
            stdin=PIPE, stdout=PIPE
        )

        # 初始化握手
        await self._send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "clientInfo": {"name": "continuum", "version": "1.0"}
        })

    async def list_tools(self) -> list[MCPTool]:
        """获取服务器提供的工具"""
        response = await self._send_request("tools/list", {})
        return [MCPTool(**t) for t in response.get("tools", [])]
```

---

## 五、测试策略

### 4.1 测试层级

```
tests/
├── unit/               # 单元测试: Mock 外部依赖
│   ├── test_llm_provider.py
│   ├── test_context_manager.py
│   ├── test_hooks.py
│   └── test_token_budget.py
├── integration/        # 集成测试: 模块间协作
│   ├── test_agent_flow.py
│   ├── test_memory_integration.py
│   └── test_mcp_integration.py
└── e2e/               # 端到端测试: 真实 API
    ├── test_real_llm.py
    └── test_full_workflow.py
```

### 4.2 覆盖率目标

| 模块 | 目标覆盖率 | 说明 |
|------|-----------|------|
| LLM Provider | 90% | 核心模块 |
| Context Manager | 85% | 含压缩逻辑 |
| Tool System | 85% | 含验证超时 |
| Memory System | 80% | 持久化逻辑 |
| Hooks System | 80% | 生命周期 |
| Error Handler | 90% | 重试逻辑 |
| Workflow Engine | 75% | 状态图复杂 |
| MCP Client | 70% | 外部依赖 |

---

## 六、协作分工建议

### 6.1 角色分工

| 角色 | 负责模块 | 优先任务 |
|------|----------|----------|
| **Enjou** | 核心架构、Memory、Hooks | Phase 1-2 核心模块 |
| **egg** | 工具系统、MCP、测试 | Phase 2-3 扩展模块 |
| **协作** | Workflow、文档 | Phase 4-5 集成 |

### 6.2 协作流程

1. **Issue 驱动**: 每个 Phase 创建 Issue 清单
2. **PR Review**: 关键模块需要双方 Review
3. **集成测试**: 每个 Phase 结束运行完整测试
4. **文档同步**: 功能完成即更新文档

---

## 七、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| API 变更 | LLM Provider 失效 | 抽象层隔离，快速适配 |
| Token 估算不准 | 预算失效 | 保守估算 + 动态调整 |
| MCP 兼容性 | 工具扩展受限 | 兼容性测试套件 |
| 性能瓶颈 | 用户体验差 | 异步优先，性能基准 |

---

## 八、参考资源

### 8.1 框架源码
- [SWE-agent](https://github.com/SWE-agent/SWE-agent) - ACI 设计
- [Aider](https://github.com/Aider-AI/aider) - Repository Map
- [Cline](https://github.com/cline/cline) - Hooks + MCP
- [LangGraph](https://github.com/langchain-ai/langgraph) - StateGraph
- [MetaGPT](https://github.com/geekan/MetaGPT) - 多 Agent SOP
- [OpenHands](https://github.com/All-Hands-AI/OpenHands) - 沙箱执行
- [AutoGen](https://github.com/microsoft/autogen) - Actor 模型 + Handoff
- [AutoGPT](https://github.com/Significant-Gravitas/AutoGPT) - 多规划策略
- [CrewAI](https://github.com/crewAIInc/crewAI) - Guardrail 验证
- [Semantic Kernel](https://github.com/microsoft/semantic-kernel) - 过滤器中间件
- [LlamaIndex](https://github.com/run-llama/llama_index) - RAG 索引

### 8.2 协议规范
- [MCP Specification](https://modelcontextprotocol.io/)
- [OpenAPI Tools Spec](https://platform.openai.com/docs/guides/function-calling)

### 8.3 学术论文
- ReWOO: Reasoning Without Observation (arXiv)
- Tree of Thoughts (arXiv:2305.10601)
- LATS: Language Agent Tree Search (arXiv:2310.04406)

---

## 九、配置系统设计

基于 AutoGen 和 SWE-agent 的声明式配置模式:

```yaml
# super.yaml
model:
  provider: openai
  name: gpt-4-turbo
  temperature: 0.7

agent:
  planning_strategy: rewoo  # one_shot | plan_execute | rewoo
  max_iterations: 30
  max_corrections: 3

hooks:
  - type: logging
  - type: metrics
  - type: permission
    config:
      blocked_tools: [run_command, delete_file]

tools:
  builtin: [file_ops, shell, search]
  mcp:
    - name: filesystem
      command: uvx
      args: [mcp-server-filesystem, .]

memory:
  project: EGG.md
  auto: true
  storage: sqlite

workflow:
  type: state_graph
  checkpoint_dir: .continuum/checkpoints
```

### 9.1 组件注册 (参考 AutoGen)

```python
# 声明式组件加载
class ComponentModel(BaseModel):
    provider: str              # "continuum.agents.AssistantAgent"
    component_type: str        # "agent", "tool", "hook"
    component_version: int    # 版本号，支持向后兼容
    config: dict[str, Any]

# 从 YAML 加载
harness = Harness.from_config("super.yaml")
```

### 9.2 环境变量覆盖

```python
# 优先级: 环境变量 > 配置文件 > 默认值
SUPER_MODEL_NAME=gpt-3.5-turbo python app.py
```

---

**文档状态**: v3.1 已完成 (修正逻辑谬误，强化"Super"品牌语义)
**时间线调整**: 10 周 → 16-20 周（更现实的估算）
**下一步**: 执行 Phase 0 项目初始化

---

## 十、核心修正原则

### 10.1 品牌定位一致性

| 原则 | 说明 |
|------|------|
| **Super = 强大** | 不是"更小"，而是更强、更完善 |
| **独立定位** | 不依附 SmolAgents，走差异化路线 |
| **品牌语义一致** | 所有定位表述匹配"Super"含义 |
| **层级互补** | 与 SmolAgents 是不同定位层级 |

### 10.2 核心卖点命名规范

所有核心功能命名需体现"Super"品牌语义：

| 功能 | 命名规范 |
|------|----------|
| 工具追踪 | SuperTracer (超级透明度) |
| 指标监控 | SuperMetrics (超级监控) |
| 错误处理 | 超级稳定性 (自动重试 + 错误分类) |
| 工具系统 | ACI 设计 (为 Agent 优化的超级控制力) |
| MCP 集成 | 超级扩展性 |
| Memory | 超级学习能力 |
