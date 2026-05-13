# sh-layer1

SuperHarness Layer 1: Foundation

This layer provides core infrastructure components including:
- LLM client (multi-provider support: Anthropic, OpenAI, Gemini)
- Configuration management
- Storage engine
- Cache manager
- Event bus
- Cost tracker

## Features

- Multi-provider LLM support
- Environment variable configuration
- TOML/JSON config file support
- Async-first design

## Usage

```rust
use sh_layer1::{ConfigManager, LlmClient, LlmProvider};

let config = ConfigManager::from_env();
let client = LlmClient::new(LlmProvider::Anthropic, api_key);
```

## License

MIT