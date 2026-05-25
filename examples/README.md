# Continuum SDK 示例代码

本目录包含 Continuum Agent SDK 的使用示例，帮助开发者快速上手。

## 目录结构

```
examples/
├── basic/          # 基础示例
│   ├── hello_agent.py      # Agent 基础使用
│   ├── streaming.py        # 流式响应
│   └── tools.py            # 自定义工具
│
├── advanced/       # 高级示例
│   ├── intelligent_agent.py    # 智能 Agent (规划/校正/进度)
│   ├── workflow_dag.py         # DAG 工作流
│   └── checkpoint_recovery.py  # 检查点恢复
│
└── integrations/   # 集成示例
    ├── mcp_server.py          # MCP 服务器集成
    └── custom_llm.py          # 自定义 LLM 配置
```

## 基础示例

### hello_agent.py

最简单的 Agent 使用示例，展示如何创建 Agent 并发送任务。

```bash
python examples/basic/hello_agent.py
```

### streaming.py

流式响应示例，展示如何实时接收 AI 输出。

```bash
python examples/basic/streaming.py
```

### tools.py

自定义工具注册示例，展示如何扩展 Agent 能力。

```bash
python examples/basic/tools.py
```

## 高级示例

### intelligent_agent.py

智能 Agent 功能演示：
- 任务自动规划
- 自校正机制
- 进度跟踪

```bash
python examples/advanced/intelligent_agent.py
```

### workflow_dag.py

DAG 工作流示例：
- 任务依赖管理
- 并行执行
- 循环检测
- ASCII 可视化

```bash
python examples/advanced/workflow_dag.py
```

### checkpoint_recovery.py

检查点系统演示：
- 执行状态保存
- 崩溃后恢复
- 状态完整性验证

```bash
python examples/advanced/checkpoint_recovery.py
```

## 集成示例

### mcp_server.py

MCP (Model Context Protocol) 集成：
- 标准 tool provider 接口
- 资源访问能力
- 外部服务连接

```bash
python examples/integrations/mcp_server.py
```

### custom_llm.py

自定义 LLM 配置：
- OpenAI 兼容接口
- 本地模型接入
- 自定义端点
- 故障转移

```bash
python examples/integrations/custom_llm.py
```

## 运行要求

1. 安装 SDK：
   ```bash
   pip install continuum-agent-sdk
   ```

2. 配置 API Key（环境变量或代码中）：
   ```bash
   export ANTHROPIC_API_KEY=your-key
   ```

3. 运行示例：
   ```bash
   python examples/basic/hello_agent.py
   ```

## 注意事项

- 示例代码中的 API Key 需替换为真实值
- 流式示例需要异步运行环境
- 部分高级功能需要完整的 SDK 安装