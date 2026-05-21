# Continuum v1.0.0 Release Notes

**Release Date**: 2026-05-12

## Overview

Continuum is a terminal agent framework with a 6-layer architecture, supporting multiple AI providers and providing both Python SDK and CLI interfaces.

## Highlights

### 🏗️ 6-Layer Architecture

```
Layer 0: Security Gateway    → Input validation, PII scrubbing
Layer 1: Foundation          → Config, cache, LLM client
Layer 2: Core Engine         → Agent runtime, session manager
Layer 3: Capabilities        → Document loaders, search
Layer 4: Integration         → MCP bridge, audit logger
Layer 5: Interface           → Python SDK, CLI
```

### 🤖 Multi-Provider Support

- Anthropic Claude (Haiku/Opus)
- OpenAI GPT (GPT-4/GPT-3.5)
- Google Gemini (Pro/Flash)
- Custom endpoints (腾讯云、阿里云 etc.)

### 🐍 Python SDK - 3 Step Quick Start

```python
from continuum_sdk import Agent

agent = Agent()
result = agent.run("hello")
```

### 💻 CLI Product

```bash
# Initialize
sh config init

# Add provider
sh config add-provider anthropic --api-key $KEY

# Run agent
sh run "分析这个项目"
```

---

## Installation

### Python SDK

```bash
pip install continuum
```

### CLI (from source)

```bash
git clone https://github.com/xxx/continuum
cd continuum
cargo install --path cli
```

---

## Features

| Feature | Description |
|---------|-------------|
| Agent Runtime | Async execution, tool calling, streaming |
| Session Manager | Concurrent sessions, checkpoint rollback |
| Tool Registry | Built-in tools + custom registration |
| Workflow Engine | DAG execution, parallel support |
| MCP Bridge | Model Context Protocol integration |
| Audit Logger | Action logging, secret tracking |

---

## Testing

| Category | Tests | Status |
|----------|-------|--------|
| Rust Core | 228 | ✅ Pass |
| Python SDK | 79 | ✅ Pass |
| Integration | 123 | ✅ Pass |
| E2E Scenarios | 23 | ✅ Pass |
| Config System | 95 | ✅ Pass |

**Total**: 500+ tests passing

---

## Quick Start

### 1. Configuration

```bash
# Create config file
sh config init

# Set API key via environment
export ANTHROPIC_API_KEY=your-key

# Or add provider directly
sh config add-provider anthropic --api-key your-key
```

### 2. Python SDK

```python
from continuum_sdk import Agent, Session

# Simple usage
agent = Agent()
response = agent.run("Write a Python function")

# With session management
session = Session(name="my-session")
agent = Agent(session_id=session.id)
response = agent.run("Remember this: I like Python")

# Checkpoint
checkpoint_id = session.save_checkpoint()
session.rollback(checkpoint_id)
```

### 3. CLI Usage

```bash
# Interactive mode
sh run

# With prompt
sh run "帮我分析项目结构"

# Session management
sh session list
sh session resume my-session

# Configuration
sh config show
sh config use openai
```

---

## Configuration Priority

```
Environment Variables  >  TOML Config  >  Defaults

SH_ANTHROPIC_API_KEY   >  config file  >  built-in
```

---

## Known Issues

None at this time. Please report issues at [GitHub Issues](https://github.com/xxx/continuum/issues).

---

## Contributors

Thank you to all contributors:

- **Terminal 1**: Python SDK, Config API, PyPI packaging
- **Terminal 2**: Rust Core, CLI, crates.io packaging
- **Terminal 3**: Testing, Review, Documentation

---

## What's Next

- [ ] PyO3 bindings for Python SDK
- [ ] Plugin marketplace
- [ ] Web UI
- [ ] More AI providers

---

## Feedback

We welcome your feedback! Please reach out via:
- GitHub Issues: [Report bugs](https://github.com/xxx/continuum/issues)
- GitHub Discussions: [Ask questions](https://github.com/xxx/continuum/discussions)

---

*Continuum - Making AI Agents Easy*