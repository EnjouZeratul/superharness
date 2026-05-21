# continuum-cli

Continuum CLI - Terminal Agent Product

A command-line interface for the Continuum agent runtime.

## Installation

```bash
cargo install continuum-cli
```

## Usage

```bash
# Initialize configuration
continuum config init

# Add API provider
continuum config add-provider anthropic --key YOUR_API_KEY

# Run agent
continuum run "your task"
```

## Features

- Multi-provider support (Anthropic, OpenAI, Gemini)
- TUI mode (terminal UI)
- Session management
- Checkpoint restore

## License

MIT