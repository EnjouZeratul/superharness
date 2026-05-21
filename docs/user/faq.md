# Continuum 常见问题解答 (FAQ)

> 版本: v1.0.0+
> 更新时间: 2026-05-21

---

## 目录

1. [安装与配置](#安装与配置)
2. [API 与模型](#api-与模型)
3. [使用问题](#使用问题)
4. [性能与成本](#性能与成本)
5. [错误排除](#错误排除)
6. [高级使用](#高级使用)

---

## 安装与配置

### Q1: 安装时提示 "Microsoft Visual C++ 14.0 is required"？

**原因**: Windows 系统缺少编译工具。

**解决方案**:

```bash
# 方案 A: 安装预编译包
pip install continuum --prefer-binary

# 方案 B: 安装 Visual Studio Build Tools
# 下载地址: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# 选择 "Desktop development with C++"
```

### Q2: pip 安装很慢或超时？

**原因**: 默认源在国外。

**解决方案**:

```bash
# 临时使用国内镜像
pip install continuum -i https://pypi.tuna.tsinghua.edu.cn/simple

# 永久配置
pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
```

### Q3: macOS 提示 "command not found: continuum"？

**原因**: PATH 环境变量未包含 pip 安装路径。

**解决方案**:

```bash
# 检查安装位置
pip show continuum | grep Location

# 添加到 PATH（添加到 ~/.zshrc 或 ~/.bash_profile）
export PATH="$HOME/.local/bin:$PATH"

# 重新加载配置
source ~/.zshrc
```

### Q4: 如何在虚拟环境中安装？

**解决方案**:

```bash
# 创建虚拟环境
python -m venv sh-env

# 激活（macOS/Linux）
source sh-env/bin/activate

# 激活（Windows）
sh-env\Scripts\activate

# 安装
pip install continuum
```

### Q5: 如何查看当前版本？

```bash
# CLI
continuum --version

# Python
python -c "import continuum_sdk; print(continuum_sdk.__version__)"
```

---

## API 与模型

### Q6: 支持哪些 AI 模型？

| 提供商 | 支持模型 |
|--------|----------|
| Anthropic | claude-3-haiku, claude-3-sonnet, claude-3-opus |
| OpenAI | gpt-4, gpt-4-turbo, gpt-3.5-turbo |
| Google | gemini-pro, gemini-flash |
| 本地模型 | Ollama 兼容模型 |

### Q7: 如何切换不同的 AI 提供商？

```bash
# 方式 A：配置命令
continuum config use openai

# 方式 B：环境变量
export SH_MODEL=gpt-4
continuum run "hello"

# 方式 C：配置文件
# 编辑 ~/.sh/config.toml
model = "gpt-4"
```

### Q8: API Key 如何安全存储？

**推荐方式**:

```bash
# 方式 A：环境变量（开发环境）
export ANTHROPIC_API_KEY="sk-xxx"

# 方式 B：环境变量文件（.env，不提交到 git）
# 创建 .env 文件
echo "ANTHROPIC_API_KEY=sk-xxx" > .env
echo ".env" >> .gitignore

# 方式 C：系统密钥管理器（生产环境）
# macOS Keychain / Windows Credential Manager / Linux libsecret
```

### Q9: 如何使用自定义 API 端点？

```bash
# 添加自定义提供商
continuum config add-provider custom \
  --api-key "your-key" \
  --url "https://your-api.example.com/v1" \
  --model "your-model"

# 切换使用
continuum config use custom
```

### Q10: 如何设置请求超时？

```bash
# 环境变量
export SH_TIMEOUT=120000  # 120 秒

# 配置文件
# ~/.sh/config.toml
timeout = 120000
```

---

## 使用问题

### Q11: 如何让 Agent 记住信息？

使用记忆系统：

```python
from continuum_sdk import Agent
from continuum_sdk.memory import Memory

agent = Agent(memory=Memory())

# 添加记忆
agent.memory.project().add("project_name", "MyProject")

# 检索记忆
name = agent.memory.recall("project_name")
```

### Q12: 如何限制 Token 使用量？

```python
from continuum_sdk import Agent, Config

config = Config(
    max_tokens=4096,
    budget_limit=100000  # 100k tokens
)

agent = Agent(config=config)
```

### Q13: 如何保存和恢复会话？

```python
from continuum_sdk import Session

# 创建会话
session = Session(name="my-session")
session.save()

# 恢复会话
session = Session.load("my-session")
```

### Q14: 如何查看 Agent 的执行过程？

```bash
# 启用详细模式
continuum run --verbose "你的任务"

# 或设置环境变量
export RUST_LOG=debug
continuum run "你的任务"
```

### Q15: 如何使用自定义工具？

```python
from continuum_sdk.tools import tool, get_registry

@tool(name="my_tool", description="自定义工具")
async def my_tool(input: str) -> str:
    return f"处理结果: {input}"

# 注册并使用
registry = get_registry()
registry.register(my_tool)

agent = Agent(tools_enabled=True)
agent.run("使用 my_tool 处理 hello")
```

---

## 性能与成本

### Q16: 如何估算 API 成本？

```python
from continuum_sdk import Agent

agent = Agent()
result = agent.run("任务")

# 查看 Token 使用
print(f"输入: {agent.input_tokens}")
print(f"输出: {agent.output_tokens}")
print(f"总成本: ${agent.cost_estimate:.4f}")
```

**价格参考** (2026年，实际以官方为准):

| 模型 | 输入价格 | 输出价格 |
|------|----------|----------|
| Claude 3 Haiku | $0.25/1M | $1.25/1M |
| GPT-4 Turbo | $10/1M | $30/1M |

### Q17: 如何优化响应速度？

**建议**:

1. 使用更快的模型 (Haiku, GPT-3.5)
2. 减少上下文长度
3. 启用流式响应

```python
config = Config(
    model="claude-3-haiku",
    streaming=True
)
```

### Q18: 内存占用过高怎么办？

**解决方案**:

```bash
# 限制会话历史
export SH_MAX_HISTORY=50

# 定期清理旧会话
continuum session prune --older-than 7d
```

### Q19: 如何批量处理任务？

```python
from continuum_sdk import Agent

agent = Agent()
tasks = ["任务1", "任务2", "任务3"]

for task in tasks:
    result = agent.run(task)
    print(result)
```

---

## 错误排除

### Q20: 提示 "API key not found"？

**检查步骤**:

```bash
# 1. 检查环境变量
echo $ANTHROPIC_API_KEY  # macOS/Linux
echo %ANTHROPIC_API_KEY%  # Windows CMD

# 2. 检查配置文件
continuum config show

# 3. 验证配置
continuum config validate
```

### Q21: 提示 "Rate limit exceeded"？

**原因**: API 调用频率超限。

**解决方案**:

```bash
# 增加重试等待时间
export SH_RETRY_DELAY=60

# 降低并发数
export SH_MAX_CONCURRENT=1
```

### Q22: 提示 "Context length exceeded"？

**原因**: 输入太长超出模型限制。

**解决方案**:

```python
# 减少历史记录
config = Config(
    max_history_turns=10
)

# 或清空会话
session.clear_messages()
```

### Q23: TUI 界面显示乱码？

**解决方案**:

```bash
# 检查终端编码
echo $LANG  # 应为 UTF-8

# 设置编码
export LANG=en_US.UTF-8

# 重置终端
reset
```

### Q24: 工具执行失败？

**调试步骤**:

```bash
# 启用调试模式
export RUST_LOG=debug

# 检查工具状态
continuum tools list

# 测试工具
continuum run --verbose "执行测试任务"
```

---

## 高级使用

### Q25: 如何构建工作流？

```python
from continuum_sdk.workflow import DAG, Node

# 创建工作流
dag = DAG("analysis-workflow")

# 添加节点
dag.add(Node("fetch", func=fetch_data))
dag.add(Node("process", func=process_data))
dag.add(Node("save", func=save_data))

# 设置依赖
dag.depends_on("process", "fetch")
dag.depends_on("save", "process")

# 执行
result = await dag.execute()
```

### Q26: 如何集成到 CI/CD？

```yaml
# GitHub Actions 示例
name: AI Review
on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      
      - name: Install Continuum
        run: pip install continuum
      
      - name: Run AI Review
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          continuum run "审查当前 PR 的代码变更" > review.md
      
      - name: Post Review
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const review = fs.readFileSync('review.md', 'utf8');
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: review
            });
```

### Q27: 如何自定义 Agent 行为？

```python
from continuum_sdk import Agent

class MyAgent(Agent):
    def before_run(self, task: str) -> str:
        # 预处理任务
        return f"[Custom] {task}"
    
    def after_run(self, result: str) -> str:
        # 后处理结果
        return f"{result}\n\n--- Generated by MyAgent"

agent = MyAgent()
```

### Q28: 如何处理多语言？

Continuum 原生支持多语言：

```bash
# 直接使用任何语言
continuum run "Hello, how are you?"
continuum run "你好，最近怎么样？"
continuum run "こんにちは、お元気ですか？"
```

### Q29: 如何贡献代码？

```bash
# 1. Fork 仓库
# 2. 克隆你的 Fork
git clone https://github.com/YOUR_USERNAME/continuum.git

# 3. 创建分支
git checkout -b feature/my-feature

# 4. 安装开发依赖
pip install -e ".[dev]"

# 5. 运行测试
pytest tests/

# 6. 提交 PR
git push origin feature/my-feature
```

---

## 更多问题？

如果以上 FAQ 没有解决你的问题：

1. 查看 [完整文档](../USER_MANUAL.md)
2. 搜索 [GitHub Issues](https://github.com/anthropics/continuum/issues)
3. 提交新的 [Issue](https://github.com/anthropics/continuum/issues/new)

---

*Continuum - 让 AI Agent 开发变得简单*
