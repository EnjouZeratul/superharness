# sh-layer0

SuperHarness Layer 0: Security Gateway

This layer provides security validation and sanitization for all inputs and outputs in the SuperHarness agent runtime.

## Features

- Input validation and sanitization
- Path traversal protection
- Command injection prevention
- Rate limiting and budget tracking

## Usage

```rust
use sh_layer0::{SecurityGateway, InputValidator};

let gateway = SecurityGateway::new();
let validated = gateway.validate_input("user input")?;
```

## License

MIT