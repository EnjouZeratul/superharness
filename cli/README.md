# superharness-cli

SuperHarness CLI - Terminal Agent Product

A command-line interface for the SuperHarness agent runtime.

## Installation

```bash
cargo install superharness-cli
```

## Usage

```bash
# Initialize configuration
superharness config init

# Add API provider
superharness config add-provider anthropic --key YOUR_API_KEY

# Run agent
superharness run "your task"
```

## Features

- Multi-provider support (Anthropic, OpenAI, Gemini)
- TUI mode (terminal UI)
- Session management
- Checkpoint restore

## License

MIT