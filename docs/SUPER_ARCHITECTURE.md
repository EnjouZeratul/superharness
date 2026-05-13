# SuperHarness：Super 架构设计理念

> 版本: v3.0
> 日期: 2026-05-09
> 核心理念: **"Super" = 强大/超越**
> 修正: 不再贬低对手，实事求是描述

---

## 一、Super 的含义

### 1.1 Super 的三重含义

```
Super = Superior（超越）
       + Supersonic（极速）
       + Superstructure（超级架构）
```

| 含义 | 说明 | SuperHarness 体现 |
|------|------|-------------------|
| **Superior** | 能力超越 | 生产级功能完整性 |
| **Supersonic** | 执行极速 | 并行执行、流式处理、自适应调度 |
| **Superstructure** | 架构优越 | 分层设计、可插拔组件、企业级扩展 |

### 1.2 SuperHarness 是什么

**SuperHarness 是简洁可靠的 Agent 运行时。**

核心设计理念：
- 可观测的执行过程
- 可控制的成本预算
- 可恢复的失败状态
- 可追溯的决策链路
| **规划能力** | 无内置 | 多策略可选 |
| **记忆系统** | 基础步骤记忆 | 分层记忆（项目+自动） |
| **可观测性** | 需集成外部 | 内置 Dashboard |
| **错误处理** | 基础重试 | 自愈容错 |
| **企业功能** | 无 | 审计日志、权限控制 |

---

## 三、Super 架构核心设计

### 3.1 分层架构设计

```
┌────────────────────────────────────────────────────────────────┐
│                     Application Layer                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │ CLI App  │  │ Web App  │  │ API Server│  │IDE Plugin│      │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘      │
├────────────────────────────────────────────────────────────────┤
│                     Framework Layer                             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    SuperHarness Core                      │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │  │
│  │  │ Orchestrator│  │ State Engine│  │ Memory Hub │        │  │
│  │  │ (编排引擎)   │  │ (状态引擎)   │  │ (记忆中心)  │        │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘        │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │  │
│  │  │ Tool Router  │  │ Checkpointer│  │ Observer   │        │  │
│  │  │ (工具路由)    │  │ (检查点)     │  │ (观测器)    │        │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘        │  │
│  └──────────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────────┤
│                     Integration Layer                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │  LLM API │  │MCP Server│  │ Vector DB│  │Sandbox   │      │
│  │ Providers│  │集成      │  │ 存储后端 │  │运行时    │      │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘      │
└────────────────────────────────────────────────────────────────┘
```

### 3.2 核心组件设计

#### Orchestrator（编排引擎）

**职责**：统一编排 Agent 执行流程

```python
class Orchestrator:
    """
    Super 编排引擎 - 支持多种执行模式

    能力：
    - 顺序执行
    - 并行执行
    - 条件分支
    - 循环控制
    - 错误恢复
    """

    async def execute(self, plan: ExecutionPlan) -> Result:
        # 1. 解析执行计划
        dag = self._build_dag(plan)

        # 2. 并行调度
        tasks = self._schedule_parallel(dag)

        # 3. 状态管理
        results = await self._run_with_checkpoint(tasks)

        # 4. 错误自愈
        if results.has_errors():
            results = await self._self_heal(results)

        return results
```

#### State Engine（状态引擎）

**职责**：管理 Agent 执行状态，支持 Checkpoint

```python
class StateEngine:
    """
    Super 状态引擎 - 支持 Checkpoint 和恢复

    能力：
    - 状态快照
    - 断点续传
    - 状态回滚
    - 状态分支
    """

    async def checkpoint(self) -> StateSnapshot:
        """保存当前状态快照"""
        return StateSnapshot(
            agent_state=self.agent.serialize(),
            tool_outputs=self.tools.outputs,
            memory_state=self.memory.serialize(),
            timestamp=time.now()
        )

    async def restore(self, snapshot_id: str) -> None:
        """恢复到指定快照"""
        snapshot = await self.storage.load(snapshot_id)
        self.agent.deserialize(snapshot.agent_state)
        self.tools.restore_outputs(snapshot.tool_outputs)
        self.memory.deserialize(snapshot.memory_state)
```

#### Memory Hub（记忆中心）

**职责**：统一管理多层级记忆

```python
class MemoryHub:
    """
    Super 记忆中心 - 分层记忆架构

    记忆层级：
    - Working Memory: 当前任务上下文（有限窗口）
    - Session Memory: 会话级记忆（本次对话）
    - Project Memory: 项目级记忆（EGG.md）
    - Long-term Memory: 向量数据库持久化
    """

    async def remember(self, content: str, level: MemoryLevel):
        """写入记忆"""
        if level == MemoryLevel.WORKING:
            self.working.add(content)
        elif level == MemoryLevel.SESSION:
            self.session.add(content)
        elif level == MemoryLevel.PROJECT:
            self.project.append(content)
        elif level == MemoryLevel.LONG_TERM:
            await self.vector_db.insert(content)

    async def recall(self, query: str) -> List[Memory]:
        """召回相关记忆（跨层级）"""
        results = []
        results.extend(self.working.search(query))
        results.extend(self.session.search(query))
        results.extend(self.project.search(query))
        results.extend(await self.vector_db.search(query))
        return self._rank(results)
```

#### Tool Router（工具路由）

**职责**：工具选择和编排

```python
class ToolRouter:
    """
    Super 工具路由 - 智能工具选择

    能力：
    - 语义匹配（不依赖精确名称）
    - 工具组合（自动组合多个工具）
    - 性能优化（选择最优工具）
    - 冲突检测（避免工具冲突）
    """

    async def select_tools(self, intent: str) -> List[Tool]:
        # 1. 语义匹配
        candidates = await self._semantic_match(intent)

        # 2. 检查冲突
        compatible = self._resolve_conflicts(candidates)

        # 3. 性能排序
        optimal = self._optimize_selection(compatible)

        return optimal

    async def compose_tools(self, tools: List[Tool]) -> ToolChain:
        """将多个工具组合成工具链"""
        return ToolChain(tools).with_parallel_execution()
```

#### Observer（观测器）

**职责**：全链路可观测性

```python
class Observer:
    """
    Super 观测器 - 内置可观测性

    能力：
    - 执行追踪
    - Token 统计
    - 成本计算
    - 性能分析
    - 异常检测
    """

    async def trace(self, execution: Execution) -> Trace:
        """追踪执行过程"""
        trace = Trace(execution.id)

        for step in execution.steps:
            # 记录每一步
            trace.add_step(
                input=step.input,
                output=step.output,
                tokens=step.usage,
                cost=self._calculate_cost(step.usage),
                duration=step.duration
            )

            # 异常检测
            if self._is_anomaly(step):
                trace.add_warning(step)

        return trace

    async def replay(self, trace_id: str) -> ReplaySession:
        """回放执行过程"""
        trace = await self.storage.load_trace(trace_id)
        return ReplaySession(trace)
```

---

## 四、Super 能力清单

### 4.1 核心能力对比

| 能力类别 | SmolAgents | SuperHarness | 提升 |
|----------|------------|---------------|------|
| **Agent 类型** | 2种 | 5种+（可扩展） | 2.5x |
| **执行模式** | 顺序 | 顺序/并行/条件/循环 | 4x |
| **规划策略** | 无 | 3种+（可扩展） | ∞ |
| **记忆层级** | 1层 | 4层 | 4x |
| **可观测性** | 需外部集成 | 内置 Dashboard | 原生 |
| **错误恢复** | 基础重试 | Checkpoint + 自愈 | 3x |
| **工具系统** | 手动指定 | 自动路由 | 智能 |
| **企业功能** | 无 | 审计日志/权限 | 企业级 |

### 4.2 Super 独有能力

```
SuperHarness 独有:
├── 多策略规划引擎（One-Shot / Plan-Execute / ReWOO）
├── 分层记忆架构（Working / Session / Project / Long-term）
├── 状态快照与恢复（Checkpoint）
├── 执行录像回放（Replay）
├── 自动工具路由（Semantic Matching）
├── 并行执行引擎（DAG Scheduling）
├── 内置可观测性（本地 Dashboard）
├── 企业级审计（SOC2/HIPAA）
└── Token 预算控制（Budget Management）
```

---

## 五、架构优势分析

### 5.1 生产级设计

| 维度 | SuperHarness 设计 |
|------|-------------------|
| **设计理念** | 生产优先，稳定性第一 |
| **目标用户** | 产品开发者、企业团队 |
| **架构层次** | 分层可扩展 |
| **定制能力** | 配置 + 插件 |
| **部署就绪** | 开箱即用 |

### 5.2 核心能力

SuperHarness 提供：
- 多策略规划引擎
- 分层记忆架构
- 状态快照与恢复
- 执行录像回放
- 智能工具路由
- 并行执行引擎
- 内置可观测性
- 企业级审计
- Token 预算控制
| **生产就绪** | 需自建基础设施 | 开箱即用 |

**结论**: SmolAgents 选择极简设计，SuperHarness 选择企业级基础设施。两者服务不同的设计目标。

### 5.2 vs LangChain（企业级）

| 维度 | LangChain | SuperHarness |
|------|-----------|--------------|
| **复杂度** | 高（100+ 抽象层） | 中（清晰分层） |
| **学习曲线** | 陡峭 | 平缓 |
| **可观测性** | LangSmith（付费） | 内置（免费） |
| **API 稳定性** | 频繁 breaking | 稳定承诺 |
| **依赖数量** | 核心~20+ | 核心2 |

**结论**: LangChain 适合大企业，SuperHarness 适合中小团队

### 5.3 vs PydanticAI（类型安全）

| 维度 | PydanticAI | SuperHarness |
|------|------------|---------------|
| **核心价值** | 类型安全 | 架构强大 |
| **执行模式** | 单一 | 多种 |
| **记忆系统** | 需自建 | 内置 |
| **可观测性** | 需 Logfire | 内置 |
| **规划能力** | 无 | 多策略 |

**结论**: PydanticAI 是"类型系统优先"，SuperHarness 是"能力优先"

---

## 六、Super 的实现路径

### 6.1 MVP 核心（Phase 1-2）

```
必须实现（体现"Super"）:
├── 多策略规划引擎（至少 2 种策略）
├── 分层记忆系统（Working + Project）
├── 并行执行引擎
├── Token 预算控制
└── 基础可观测性
```

### 6.2 完整 Super（Phase 3-4）

```
完整能力:
├── 状态 Checkpoint 与恢复
├── 执行录像回放
├── 智能工具路由
├── 企业级审计日志
├── Dashboard（Web UI）
└── 更多 Agent 类型
```

---

## 七、品牌定位

### 7.1 核心口号

```
主口号: "Super 架构，生产就绪"

副口号:
├── "不是更小，而是更强"
├── "SmolAgents 是演示，SuperHarness 是框架"
├── "从原型到生产，一条链路"
└── "SmolAgents 让你理解 Agent，SuperHarness 让你生产 Agent"

品牌语义澄清:
├── Super ≠ Smaller（更小）
├── Super = Superior（超越）+ Stronger（更强）+ Richer（更完善）
└── "Super" 不与 SmolAgents 比体量，而是比能力
```

### 7.2 核心卖点（匹配 "Super" 品牌语义）

| 卖点 | "Super" 关联 | 一句话价值主张 |
|------|--------------|----------------|
| **超级控制力** | Super Control | 对 Agent 执行的超级控制能力 |
| **超级透明度** | Super Visibility | 比任何框架都更透明的执行过程 |
| **超级稳定性** | Super Reliability | 企业级生产稳定性保证 |
| **超级效率** | Super Efficiency | 并行执行、自适应调度 |

### 7.3 差异化定位图

```
                    企业级
                      ↑
        LangChain ●  │
                      │
                      │    SuperHarness ● ← 定位在此
                      │    （生产级 + 架构强）
                      │
        PydanticAI ●  │
                      │
        SmolAgents ● ─┼─→ 极简
                      │
                    个人级
```

---

## 八、总结

### Super 的三重价值

| 价值 | 说明 |
|------|------|
| **Superior** | 能力超越（功能更强、更完善） |
| **Supersonic** | 执行极速（并行执行、自适应调度） |
| **Superstructure** | 架构优越（分层设计、可扩展） |

### 一句话总结

> **"SuperHarness 是简洁可靠的 Agent 运行时"**

---

**文档状态**: v2.0 Super 架构理念完成
**下一步**: 基于此架构设计具体实现方案
