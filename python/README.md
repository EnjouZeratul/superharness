# SuperHarness Python SDK

A production-grade agent framework with crash safety guarantees.

## Quick Start (3 steps)

```python
from superharness import Agent

agent = Agent()  # Auto-loads config from environment
result = agent.run("your task")
```

## Installation

```bash
pip install superharness
```

## Configuration

### Environment Variables

```bash
export SUPERHARNESS_API_KEY=your_api_key
export SUPERHARNESS_PROVIDER=anthropic  # or openai, google
export SUPERHARNESS_MODEL=claude-sonnet-4-6
```

### Config File

Create `~/.superharness/config.toml`:

```toml
[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com/v1"
model = "claude-sonnet-4-6"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"
model = "gpt-4"

[settings]
session_auto_save = true
checkpoint_enabled = true
audit_enabled = true
```

## Features

- **Agent**: One-line agent creation with automatic configuration
- **Session**: Conversation history management with checkpoint support
- **Tools**: Built-in and custom tool registration
- **Memory**: Multi-layer memory system (episodic, semantic, procedural)
- **Config**: Multi-provider configuration with environment variable support

## API Reference

```python
from superharness import Agent, Session, Config

# Agent
agent = Agent(name="my-agent", model="claude-sonnet-4-6")
agent.run("task description")  # One-shot task execution
agent.chat("message")  # Interactive chat
agent.start()  # Start agent runtime

# Session
session = agent.create_session()
session.add_message("user", "Hello")
session.save()  # Persist to storage
session.load(session_id)  # Resume session

# Config
config = Config.from_env()  # Load from environment
config = Config.from_file("~/.superharness/config.toml")  # Load from file
config.use("openai")  # Switch provider
```

## License

MIT License