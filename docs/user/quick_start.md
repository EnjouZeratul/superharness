# Continuum 快速入门指南

> 适用版本: v1.0.0+
> 更新时间: 2026-05-21

---

## 目录

1. [系统要求](#系统要求)
2. [安装步骤](#安装步骤)
3. [首次配置](#首次配置)
4. [基本使用](#基本使用)
5. [下一步学习](#下一步学习)

---

## 系统要求

### 必需环境

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Python | 3.8 | 3.11+ |
| pip | 21.0 | 最新 |
| 操作系统 | Windows 10 / macOS 10.15 / Linux | 最新稳定版 |

### 硬件要求

| 组件 | 最低配置 | 推荐配置 |
|------|----------|----------|
| 内存 | 4 GB | 8 GB+ |
| 存储 | 500 MB | 1 GB+ |
| CPU | 2 核心 | 4 核心+ |

### 网络要求

- 需要访问 AI 提供商 API (Anthropic/OpenAI/Google)
- 建议稳定网络连接

---

## 安装步骤

### 方式一：PyPI 安装（推荐）

```bash
# 1. 升级 pip
pip install --upgrade pip

# 2. 安装 Continuum
pip install continuum

# 3. 验证安装
continuum --version
# 输出: Continuum v1.0.0
```

### 方式二：从源码安装

```bash
# 1. 克隆仓库
git clone https://github.com/anthropics/continuum.git
cd continuum

# 2. 创建虚拟环境（推荐）
python -m venv .venv

# Windows
.venv\Scripts\activate

# macOS/Linux
source .venv/bin/activate

# 3. 安装依赖
pip install -e ".[dev]"

# 4. 验证安装
continuum --version
```

### 方式三：使用 pipx（隔离安装）

```bash
# 1. 安装 pipx（如果没有）
pip install pipx

# 2. 使用 pipx 安装
pipx install continuum

# 3. 验证
continuum --version
```

---

## 首次配置

### 步骤 1：获取 API Key

选择一个 AI 提供商并获取 API Key：

| 提供商 | 获取链接 | 免费额度 |
|--------|----------|----------|
| Anthropic | [console.anthropic.com](https://console.anthropic.com) | $5 新用户 |
| OpenAI | [platform.openai.com](https://platform.openai.com) | $18 新用户 |
| Google | [aistudio.google.com](https://aistudio.google.com) | 免费配额 |

### 步骤 2：配置 API Key

**方式 A：环境变量（推荐新手）**

```bash
# macOS/Linux - 添加到 ~/.bashrc 或 ~/.zshrc
export ANTHROPIC_API_KEY="your-api-key-here"

# Windows PowerShell - 添加到 $PROFILE
$env:ANTHROPIC_API_KEY = "your-api-key-here"

# Windows CMD - 添加到系统环境变量
setx ANTHROPIC_API_KEY "your-api-key-here"
```

**方式 B：配置文件**

```bash
# 初始化配置
continuum config init

# 这会创建 ~/.sh/config.toml 文件
```

编辑 `~/.sh/config.toml`：

```toml
model = "claude-3-haiku"

[providers.anthropic]
api_key = "your-api-key-here"
base_url = "https://api.anthropic.com"
```

**方式 C：CLI 命令**

```bash
# 添加提供商
continuum config add-provider anthropic \
  --api-key "your-api-key-here" \
  --url "https://api.anthropic.com" \
  --model "claude-3-haiku"

# 设为默认
continuum config use anthropic
```

### 步骤 3：验证配置

```bash
# 验证配置有效性
continuum config validate

# 查看当前配置
continuum config show
```

---

## 基本使用

### 第一次对话

```bash
# 启动 TUI（终端用户界面）
continuum

# 或直接运行单次任务
continuum run "你好，请介绍一下你自己"
```

### TUI 基本操作

启动后，你会看到如下界面：

```
┌─────────────────────────────────────────────────────────────┐
│ Status: idle | Model: claude-3-haiku | Tokens: 0           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Welcome to Continuum!                                   │
│  Type your message below and press Enter to send.           │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ > _                                                         │
└─────────────────────────────────────────────────────────────┘
```

**基本操作流程：**

1. 在输入框输入消息
2. 按 `Enter` 发送
3. 等待 AI 响应
4. 继续对话或按 `Ctrl+C` 退出

### 常用快捷键

| 快捷键 | 功能 |
|--------|------|
| `Enter` | 发送消息 |
| `Ctrl+Enter` | 换行（多行输入） |
| `Ctrl+C` | 退出程序 |
| `Tab` | 切换焦点区域 |
| `↑` / `↓` | 浏览历史消息 |
| `/` | 搜索消息 |
| `?` | 显示帮助 |

### 执行代码任务

Continuum 可以帮你写代码：

```bash
continuum run "写一个 Python 函数，计算斐波那契数列"
```

或者更复杂的任务：

```bash
continuum run "创建一个 Flask 应用，包含用户登录功能"
```

### 会话管理

```bash
# 查看所有会话
continuum session list

# 恢复之前的会话
continuum session resume <session-id>

# 删除会话
continuum session delete <session-id>
```

### 使用 Python SDK

```python
from continuum_sdk import Agent

# 创建 Agent
agent = Agent()

# 执行任务
result = agent.run("分析当前目录的项目结构")
print(result)

# 多轮对话
agent.run("记住：我的项目使用 Python 3.11")
agent.run("我之前说使用的什么版本？")  # 会记住上下文
```

---

## 下一步学习

### 推荐阅读顺序

1. **[工具使用指南](./tools_guide.md)** - 学习如何使用内置工具
2. **[常见问题解答](./faq.md)** - 解决常见问题
3. **[完整用户手册](../USER_MANUAL.md)** - 详细功能说明

### 进阶主题

| 主题 | 描述 |
|------|------|
| 自定义工具 | 创建自己的工具扩展 |
| 工作流 | 使用 DAG 构建复杂流程 |
| 记忆系统 | 让 Agent 记住重要信息 |
| 多模型切换 | 在不同 AI 模型间切换 |

### 示例项目

查看 `examples/` 目录获取更多示例：

```bash
ls examples/
# basic_usage.py      - 基础用法
# custom_tools.py     - 自定义工具
# workflows.py        - 工作流示例
# memory_demo.py      - 记忆系统演示
```

---

## 获取帮助

- **文档**: https://continuum.readthedocs.io
- **GitHub**: https://github.com/anthropics/continuum
- **问题反馈**: https://github.com/anthropics/continuum/issues

---

*Continuum - 让 AI Agent 开发变得简单*
