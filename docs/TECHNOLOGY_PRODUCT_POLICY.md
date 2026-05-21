# Continuum 技术与产品方针

> 最后更新: 2026-05-09
> 基于: 十三轮专家评审 + 开源精神定位

---

## 一、核心定位

**Continuum 是简洁可靠的 Agent 运行时。**

**一句话定位**："让你的 Agent 不再'盲开'"

### 开源精神共识

核心转变：
```
修正前              →    修正后
────────────          ────────────
"让用户付费"        →    "让用户喜欢"
"追求商业壁垒"      →    "追求技术卓越"
"功能堆砌"          →    "一件事做到95分"
"封闭护城河"        →    "开放社区贡献"
```

### 与 SmolAgents 定位差异

| 功能方向 | SmolAgents 定位 | Continuum 定位 |
|----------|----------------|-------------------|
| 极简设计 | 核心目标 | 非核心目标 |
| 生产可靠性 | 非核心目标 | 核心目标 |
| 成本控制 | 非核心目标 | 核心目标 |
| 企业合规 | 非核心目标 | 核心目标 |

---

## 二、创新点

### 核心创新点

| 创新点 | 用户价值 | 实现难度 |
|--------|----------|----------|
| Agent 执行录像与回放 | 极高 | 中（4-5周） |
| Agent 基础诊断系统 | 高 | 中（3-4周） |
| Token 用量追踪与预测 | 高 | 低（1-2周） |
| 合规审计日志系统 | 高 | 中（2-3周） |
| 手动Checkpoint机制 | 中 | 中高（3-4周） |

### MVP核心创新点（基于GitHub真实痛点）

| 创新点 | MVP阶段 | 热度数据 | 真实性评分 | GitHub证据 |
|--------|---------|----------|-----------|------------|
| 会话无缝延续 | ✅ MVP核心 | ⚠️ 低(1👍) | 7/10* | #43262 |
| 终端滚动锁定 | ⚠️ 非MVP | 🔥 极高(686👍, 总反应818) | 10/10 | #826 |
| Agent失控检测 | ⚠️ Phase 2 | 中等 | 9/10 | 多Issue |
| 用量追踪预测 | ⚠️ Phase 2 | 待验证 | 8/10 | 用户调研 |

> **热度诚实说明**：
> - 会话延续热度低(1赞)，但选择理由是**技术定位正确**——这是Agent运行时的核心能力
> - 滚动问题热度极高(686赞)，但**技术归属存疑**——这是CLI层问题，非Agent框架可解决
> - *真实性评分下调：从8.5→7，因用户验证不足

### 为何不选滚动问题作为MVP核心？

| 评估维度 | 分析结论 |
|----------|----------|
| 热度验证 | ✅ 充分：686赞，第一大痛点 |
| 技术归属 | ❌ 存疑：滚动是CLI层问题，非Agent框架职责 |
| 框架可控性 | 部分：可提供OutputCapture钩子，但无法解决根本问题 |
| 差异化价值 | 低：其他框架可同样提供钩子接口 |

**决策**：滚动问题在"集成示例"中体现，提供`OutputCapture`钩子让用户自行控制输出流。

---

## 三、目标用户

**新定位**: 面向产品开发者的可配置 Agent 引擎

| 用户类型 | 需求 | Continuum 价值 |
|----------|------|-------------------|
| SaaS 产品开发者 | 集成 Agent 到产品 | SDK 形态 + 可嵌入 |
| 原型开发者 | 快速实验 | YAML 配置 + 模板库 |
| 企业集成开发者 | 定制需求 | Hooks 扩展 + 完全开源 |
| 教育研究者 | 教学/研究 | 清晰架构 + 可观测 |

**不应作为主要目标用户**:
- 想用 Agent 提高编程效率的个人开发者 → 推荐 Cursor/Aider
- 需要企业级平台的大型团队 → 推荐 LangChain + LangSmith

---

## 四、竞品对比

### 真正独特的优势

| 优势 | 壁垒强度 | 说明 |
|------|---------|------|
| 双层 Memory 系统 | 高 | 项目记忆 + 自动记忆组合设计独特 |
| YAML + Python 双模式 | 中 | 配置驱动 + SDK 形态组合 |

### 遗漏的竞争对手

| 竞品 | 定位 | 相似度 |
|------|------|--------|
| PydanticAI | 轻量 Agent 框架 | 高 |
| SmolAgents | 极简 Agent 框架 | 高 |

### 竞争策略

**不应正面竞争**:
- 企业级平台 (LangSmith 主导)
- RAG 系统 (LlamaIndex 专业)
- 多 Agent 协作 (AutoGen 成熟)

**应聚焦独特定位**:
- 轻量 SDK (最小依赖，最快启动)
- 配置驱动 (YAML 定义 Agent)
- 深度定制 (完全开源，无锁定)
- 调试体验 (开源免费 vs LangSmith 商业付费)

---

## 五、产品路线图

### 时间表

| 阶段 | 估算 | 说明 |
|------|------|------|
| Phase 0 | 1 周 | 项目初始化 |
| Phase 1 | 4-5 周 | LLM Provider |
| Phase 2 | 4-5 周 | Agent Runtime |
| Phase 3 | 5-6 周 | Memory + Hooks |
| Phase 4 | 6-8 周 | MCP 兼容性 |
| Phase 5 | 3-4 周 | 文档 + 示例 + 测试 |

### MVP核心（聚焦单一功能）

**会话无缝延续**（MVP唯一核心功能）：
- LLM Provider (OpenAI 仅)
- Agent Runtime (Tool Calling Loop)
- Tool Registry + 1 个示例工具
- SimpleTracer (仅控制台打印)
- SessionManager（自动checkpoint + /continue指令）

**一句话定位**："让你的 Agent 不再'盲开'"

### 建议延后（Phase 2+）

- Anthropic Provider (Phase 3)
- ProjectMemory (Phase 3)
- Hooks (Phase 3)
- AutoMemory、Workflow Engine、Agent Planner
- 终端滚动锁定 (Phase 2)
- Agent失控检测 (Phase 2)
- 用量追踪预测 (Phase 2)

---

## 六、技术设计（MVP核心）

> 本章节定义MVP核心组件的关键技术设计，确保实现一致性。

### 6.1 SessionManager 设计（MVP核心）

SessionManager 是 MVP 的核心组件，负责会话状态的持久化与恢复。

#### 6.1.1 Checkpoint 触发时机

| 触发点 | 触发条件 | 说明 |
|--------|----------|------|
| Tool调用前 | 每次Tool Calling前 | 记录即将执行的动作 |
| Tool调用后 | Tool返回后 | 记录执行结果 |
| 用户中断 | Ctrl+C / 信号 | 保存当前状态 |
| 显式请求 | `/checkpoint` 指令 | 手动触发 |
| 周期性 | 每 N 轮对话（可配置，默认5轮） | 防止数据丢失 |

**设计原则**：宁可多checkpoint，不可丢失状态。存储成本远低于重新执行成本。

#### 6.1.2 Checkpoint 格式定义

```python
@dataclass
class Checkpoint:
    """单次checkpoint的完整状态快照"""
    # 元数据
    checkpoint_id: str          # UUID v4
    session_id: str             # 会话标识
    created_at: datetime        # 创建时间（ISO 8601）
    trigger: CheckpointTrigger  # 触发来源：tool_call / user / periodic / manual

    # Agent状态
    agent_state: AgentState     # 当前状态机位置
    iteration: int              # 当前迭代轮次

    # 执行上下文
    messages: List[Dict]        # 完整消息历史（OpenAI格式）
    tool_calls_pending: List[Dict]  # 待处理的Tool Call
    tool_results: List[Dict]    # 已完成的Tool结果

    # 用量追踪
    tokens_used: int            # 累计Token消耗
    cost_estimate: float       # 预估成本（美元）

    # 恢复元信息
    resume_hint: Optional[str] # 恢复时的提示信息
```

#### 6.1.3 存储位置

```
~/.continuum/
├── sessions/
│   └── {session_id}/
│       ├── checkpoints/
│       │   ├── cp_{timestamp}_{id}.json  # 按时间排序
│       │   └── latest.json -> cp_xxx.json # 符号链接指向最新
│       ├── session_meta.json             # 会话元数据
│       └── config_snapshot.yaml          # 使用的配置快照
└── global_config.yaml                    # 全局配置
```

**存储策略**：
- 每个session独立目录
- checkpoint文件名包含时间戳，便于人工检查
- `latest.json` 符号链接指向最新checkpoint
- 最大保留数：默认保留最近50个checkpoint（可配置）
- 清理策略：LRU，超过阈值删除最旧checkpoint

#### 6.1.4 恢复逻辑

```
恢复流程：
1. 检测 /continue 指令或上次异常退出
2. 定位最新checkpoint（latest.json）
3. 加载 checkpoint 数据
4. 验证完整性（schema校验）
5. 恢复 ExecutionContext
6. 输出恢复信息给用户
7. 从断点继续执行
```

**恢复信息示例**：
```
已恢复会话: session_abc123
最后checkpoint: 2026-05-09 14:32:15
恢复点: 第12轮，Tool 'search_files' 调用后
已消耗: 15,420 tokens (~$0.03)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
继续执行...
```

#### 6.1.5 /continue 指令处理

```python
class ContinueHandler:
    """处理 /continue 指令"""

    def handle(self, session_id: Optional[str] = None):
        if session_id:
            # 指定session
            return self._load_session(session_id)
        else:
            # 查找最近session
            recent = self._find_recent_session()
            if recent:
                return self._load_session(recent)
            else:
                return "未找到可恢复的会话"

    def _load_session(self, session_id: str):
        checkpoint = self._load_latest_checkpoint(session_id)
        if not checkpoint:
            return f"Session {session_id} 无有效checkpoint"

        # 验证并恢复
        context = ExecutionContext.from_checkpoint(checkpoint)
        return context
```

---

### 6.2 核心数据结构

#### 6.2.1 AgentState 状态机

```
状态机定义：
                                    ┌──────────────┐
                                    │   IDLE       │ ← 初始状态
                                    └──────┬───────┘
                                           │ start()
                                           ▼
                                    ┌──────────────┐
                          ┌────────│  RUNNING     │◄───────┐
                          │        └──────┬───────┘        │
                          │ tool_call    │                 │ next_iteration
                          ▼             │ complete()      │
                   ┌──────────────┐     │                 │
                   │ TOOL_CALLING │     │                 │
                   └──────┬───────┘     │                 │
                          │ tool_result │                 │
                          ▼             │                 │
                   ┌──────────────┐     │                 │
                   │ WAITING_TOOL │─────┴─────────────────┘
                   └──────────────┘

                                    ┌──────────────┐
                                    │   STOPPED     │ ← 用户中断
                                    └──────────────┘

                                    ┌──────────────┐
                                    │   ERROR       │ ← 异常终止
                                    └──────────────┘

                                    ┌──────────────┐
                                    │  COMPLETED    │ ← 正常结束
                                    └──────────────┘
```

```python
class AgentState(Enum):
    """Agent执行状态机"""
    IDLE = "idle"              # 空闲，未启动
    RUNNING = "running"        # 正在执行LLM调用
    TOOL_CALLING = "tool_calling"  # 正在调用Tool
    WAITING_TOOL = "waiting_tool"  # 等待Tool结果
    STOPPED = "stopped"        # 用户中断
    ERROR = "error"            # 异常终止
    COMPLETED = "completed"    # 正常结束

# 状态转换验证
VALID_TRANSITIONS = {
    AgentState.IDLE: [AgentState.RUNNING],
    AgentState.RUNNING: [AgentState.TOOL_CALLING, AgentState.COMPLETED, AgentState.STOPPED, AgentState.ERROR],
    AgentState.TOOL_CALLING: [AgentState.WAITING_TOOL, AgentState.ERROR],
    AgentState.WAITING_TOOL: [AgentState.RUNNING, AgentState.STOPPED],
    AgentState.STOPPED: [AgentState.IDLE],  # 可恢复重启
    AgentState.ERROR: [AgentState.IDLE],    # 可恢复重启
    AgentState.COMPLETED: [AgentState.IDLE] # 可开始新会话
}
```

#### 6.2.2 ExecutionContext 执行上下文

```python
@dataclass
class ExecutionContext:
    """Agent执行的完整上下文，支持序列化"""
    # 标识
    session_id: str
    agent_id: str

    # 状态
    state: AgentState
    iteration: int
    max_iterations: int

    # 消息历史（OpenAI格式）
    messages: List[Dict[str, Any]]

    # Tool管理
    tools_registered: List[str]       # 已注册Tool名称
    tool_calls_pending: List[Dict]    # 待处理Tool Call
    tool_results_cache: Dict[str, Any] # Tool结果缓存

    # 配置快照
    model: str
    temperature: float
    system_prompt: str

    # 追踪数据
    tokens_total: int
    tokens_prompt: int
    tokens_completion: int
    cost_estimate: float

    # 元数据
    created_at: datetime
    last_updated: datetime
    checkpoint_count: int

    def to_checkpoint(self) -> Checkpoint:
        """转换为可持久化的checkpoint"""
        ...

    @classmethod
    def from_checkpoint(cls, cp: Checkpoint) -> 'ExecutionContext':
        """从checkpoint恢复"""
        ...
```

#### 6.2.3 SessionState 会话状态

```python
@dataclass
class SessionState:
    """会话级别状态，跨checkpoint持久化"""
    # 基本信息
    session_id: str
    created_at: datetime
    config_path: str

    # 用户信息
    user_id: Optional[str]
    project_name: Optional[str]

    # 累计统计（跨所有checkpoint）
    total_iterations: int
    total_tokens: int
    total_cost: float
    total_tool_calls: int

    # Checkpoint管理
    checkpoint_ids: List[str]      # 按时间顺序
    latest_checkpoint_id: str
    latest_checkpoint_at: datetime

    # 状态
    is_active: bool
    termination_reason: Optional[str]

    # 恢复历史
    resume_count: int
    last_resumed_at: Optional[datetime]
```

---

### 6.3 依赖诚实声明

> **重要澄清**：此前文档声称"核心依赖仅2个"，该表述不准确。

#### 6.3.1 实际依赖清单

**直接依赖**（setup.py中显式声明）：

| 依赖包 | 用途 | 版本要求 | 是否核心 |
|-------|------|----------|----------|
| httpx | HTTP客户端（LLM API调用） | >=0.25.0 | **核心** |
| pydantic | 数据验证与序列化 | >=2.0.0 | **核心** |
| pyyaml | YAML配置解析 | >=6.0 | 推荐（配置驱动） |
| rich | 终端美化输出 | >=13.0 | 推荐（调试体验） |
| python-dotenv | 环境变量管理 | >=1.0.0 | 可选（开发便利） |

**传递依赖**（间接引入，数量约3-5个）：

| 依赖包 | 来源 | 说明 |
|-------|------|------|
| annotated-types | pydantic | 类型注解支持 |
| pydantic-core | pydantic | 核心验证逻辑 |
| typing-extensions | httpx/pydantic | 类型扩展 |
| certifi | httpx | SSL证书 |
| idna | httpx | 国际化域名 |
| sniffio | httpx | 异步检测 |

#### 6.3.2 依赖声明修正

```
修正前              →    修正后
────────────           ────────────
"核心依赖仅2个"       →    "核心依赖2个，推荐依赖3个，传递依赖约5个"
"极致轻量"           →    "轻量，但非极简"
```

#### 6.3.3 与竞品依赖对比（诚实版）

| 框架 | 直接依赖数 | 总依赖数（含传递） |
|------|-----------|------------------|
| SmolAgents | ~3 | ~8 |
| Continuum | ~5 | ~10 |
| LangChain | ~15+ | 50+ |
| AutoGen | ~20+ | 80+ |

**结论**：Continuum 相比 SmolAgents 略重，但仍远轻于 LangChain/AutoGen。我们应在文档中诚实表述为"轻量级，非极简"。

---

## 七、质量保证策略

### 7.1 测试策略

| 测试类型 | 覆盖率目标 | 优先级 |
|----------|-----------|--------|
| 单元测试 | 80% | P0 |
| 集成测试 | 核心路径100% | P0 |
| E2E测试 | 3个关键场景 | P1 |

**测试工具**：pytest + pytest-asyncio + pytest-cov

### 7.2 质量门槛

MVP发布前必须满足：
- [ ] 单元测试覆盖率 >= 80%
- [ ] 所有代码示例可运行
- [ ] Quick Start文档经外部用户验证
- [ ] 无P0级Bug

---

## 八、推广策略

### 推广时机

| 时机 | 项目状态 | 推荐动作 |
|------|----------|----------|
| 现在 | 无代码 | 不推广 |
| MVP 完成 | 核心功能可用 | 灰度邀请10-20位种子用户 |
| Phase 3 完成 | Agent + Memory + Hooks | Beta 发布 |
| Phase 5 完成 | 完整功能 + 文档 | 全渠道推广 |

### Beta发布前检查清单

- 核心功能可用且稳定
- Quick Start 文档经外部用户验证
- 至少 3 个实用示例
- README、Issues模板、CONTRIBUTING、LICENSE 就绪
- 至少 5 位种子用户确认愿意背书

---

## 九、社区运营机制

### 9.1 贡献者等级体系

| 等级 | 名称 | 晋升条件 | 权益 |
|------|------|----------|------|
| Level 1 | 用户 | 使用项目 | 社区支持、文档访问 |
| Level 2 | Issue贡献者 | 提交≥3个有效Issue | Issue优先处理、贡献者名单展示 |
| Level 3 | 代码贡献者 | 合并≥1个PR | PR优先审查、Dev频道访问 |
| Level 4 | 核心贡献者 | 合并≥5个PR + 持续参与≥3月 | 代码审查权、Roadmap投票、@continuum.dev邮箱 |
| Level 5 | 维护者 | 核心贡献者 + 维护者提名 | 合并权限、发布决策、项目代表权 |

**晋升评估周期**：每季度末进行等级评估

### 9.2 Code of Conduct

本项目采用 [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) 行为准则。

**核心原则**：
- 尊重所有社区成员，无论背景、经验水平
- 接受建设性批评，聚焦项目改进
- 对社区有积极贡献

**禁止行为**：
- 骚扰、歧视性言论或行为
- 人身攻击或政治攻击
- 未经许可发布他人私人信息
- 其他不道德或不专业的行为

**举报方式**：conduct@continuum.dev

### 9.3 安全披露政策

**安全漏洞报告流程**：

1. **报告渠道**：security@continuum.dev
2. **加密通信**：PGP公钥可在项目Wiki获取
3. **响应承诺**：确认收到后48小时内回复
4. **修复流程**：维护者修复→验证→发布安全版本→公开披露

**CVE披露原则**：
- 先修复，后公开
- 公开前通知受影响的下游项目
- 披露时包含修复版本号和升级指南

**安全公告发布渠道**：GitHub Security Advisory、邮件列表

### 9.4 响应分级标准

| 优先级 | 类型 | 首次响应 | 解决目标 |
|--------|------|----------|----------|
| P0 | 安全漏洞 | < 12小时 | 48小时内发布补丁 |
| P1 | 阻塞问题（核心功能不可用） | < 24小时 | 72小时内修复或提供workaround |
| P2 | 一般问题 | < 72小时 | 下一版本修复 |
| P3 | 功能请求 | < 1周 | 纳入Roadmap评估 |

**响应定义**：
- "首次响应"：维护者确认收到并给出初步评估
- "解决目标"：提供修复、workaround或明确的处理计划

### 9.5 社区信任红线（绝对禁止）

以下行为将立即、不可逆地摧毁社区信任，**绝对禁止**：

| 红线 | 具体表现 | 替代做法 |
|------|----------|----------|
| 虚假功能承诺 | README写功能但代码未实现 | 功能状态标签：[已实现]/[开发中]/[计划中] |
| 竞品恶意贬低 | 歪曲事实、断章取义 | 实事求是对比，承认竞品优势 |
| 隐瞒已知缺陷 | 文档回避已知Bug | 已知问题公开追踪，定期更新 |
| 过度包装头衔 | 夸大专家背书 | 诚实说明评审性质和范围 |
| 隐瞒商业关联 | 推荐功能有商业利益 | 商业关联需明确披露 |

### 9.6 信任损害响应协议

当发现损害信任的行为时，按以下流程处理：

1. **立即承认**（24小时内公开承认错误）
2. **不找借口**（不说"但是"，不说"技术上"）
3. **具体修正**（说明如何修正，给出时间线）
4. **预防复发**（说明如何防止类似问题）
5. **主动跟进**（修正后主动报告）

---

## 十、商业模式

### 许可证

**Apache-2.0**（企业友好，专利授权保护）

### 商业模式

```
开源版 (Apache-2.0)
├── 核心 Agent 运行时
├── 基础工具系统
└── 社区支持

可选 Pro 功能
├── 高级 Memory 系统
├── 更多规划策略
├── 审计日志
└── 企业级功能
```

### 产品迭代路径

| 阶段 | 时间 | 目标 |
|------|------|------|
| 开源积累期 | 1-6月 | 建立社区信任 |
| Pro版迭代期 | 7-12月 | 提供高级功能 |
| 企业版拓展期 | 13-18月 | 企业级支持 |

---

## 十一、行动计划

### 立即执行

| 行动 | 状态 |
|------|------|
| 修正竞品对比矩阵 | 已完成 |
| 删除争议性口号 | 已完成 |
| 补充遗漏竞品 | 已完成 |
| 核查 MCP 集成数据 | 待执行 |
| 开始 Phase 0 项目初始化 | 待执行 |

### 短期（1-3月）

- 完成 MVP 核心
- 建立基础文档
- 创建 3 个实用示例
- 建立种子用户池

---

## 十二、关键风险

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| 推广过早 | 高 | 等产品有实质功能再推广 |
| 与 SmolAgents 同质化 | 极高 | 聚焦差异化方向 |
| 功能蔓延 | 高 | ✅ 已聚焦单一MVP功能 |
| 用户画像存在幻想 | 中 | 增加技术能力/失败容忍度维度 |
| **MVP热度验证不足** | **高** | **会话延续仅1赞，需种子用户验证后再推广** |

---

## 十三、总结

### 最终定位

> **Continuum 是简洁可靠的 Agent 运行时**
>
> MVP核心功能：会话无缝延续（⚠️ 热度低但技术定位正确）
>
> 一句话定位：**"让你的 Agent 不再'盲开'"**
>
> **诚实说明**：MVP选择基于技术定位而非热度数据。会话延续仅1赞，需种子用户验证需求真实性后再大规模推广。

### 成功关键

- **MVP聚焦**：单一功能做到95分（会话无缝延续）
- 产品质量：功能完整 + 文档好
- 用户体验：可调试 + 配置驱动 + 成本透明
- 社区建设：贡献者生态
- 诚实传播：避免过度承诺
- 快速迭代：第一版目标是"可用"

### 开源成功公式

```
开源成功 = 质量 × 简洁 × 真实需求 × 社区信任
```

---

**相关文档**:
- [INNOVATION_HONEST_REVIEW.md](INNOVATION_HONEST_REVIEW.md) - 创新点诚实评审报告
- [REAL_PAIN_POINTS_INNOVATION.md](REAL_PAIN_POINTS_INNOVATION.md) - GitHub真实痛点调研
- [COST_CONTROL_DESIGN.md](COST_CONTROL_DESIGN.md) - Token用量追踪设计
