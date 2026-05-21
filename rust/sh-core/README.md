# sh-core

Continuum Core - Unified re-exports for all layers

This crate provides a single entry point for all Continuum Rust components, with Python bindings via PyO3.

## Features

- Re-exports all layer modules
- Python bindings for SDK
- Async runtime integration

## Usage (Python)

```python
from sh_core import Agent, Session, ConfigManager

agent = Agent()
result = agent.run("hello")
```

## License

MIT