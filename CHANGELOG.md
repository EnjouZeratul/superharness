# Changelog

All notable changes to Continuum will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Intelligent Agent with task planning and self-correction (80% coverage)
- Progress tracker with real-time status updates
- Error classification and recovery strategies
- Multi-provider LLM client support

### Documentation
- Testing strategy document (`docs/TEST_STRATEGY.md`)
- Testing standards document (`docs/TESTING_STANDARDS.md`)

### CI/CD
- PyPI publishing workflow (`publish-pypi.yml`)
- Comprehensive coverage reporting for Python and Rust

## [1.0.0] - 2026-05-12

### Added

#### Architecture
- **Layer 0: Security Gateway** - Input validation, PII scrubbing, access control, rate limiting
- **Layer 1: Foundation** - Cache manager, config manager, LLM client, storage engine
- **Layer 2: Core Engine** - Agent runtime, session manager, tool registry, workflow engine
- **Layer 3: Capabilities** - Document loaders, code search, embeddings, task management
- **Layer 4: Integration** - MCP bridge, audit logger, plugin loader
- **Layer 5: Interface** - Python SDK, CLI product

#### Multi-Provider Support
- Anthropic Claude (Claude 3 Haiku/Opus)
- OpenAI GPT (GPT-4/GPT-3.5)
- Google Gemini (Pro/Flash)
- Custom endpoints (Tencent Cloud, Alibaba Cloud, etc.)

#### Python SDK
- 3-step quick start: `Agent()`, `run()`, done
- Tool API: Built-in tools + custom tool registration
- Memory API: 4-tier memory system (Working/Session/Project/LongTerm)
- Workflow API: DAG-based workflow execution
- Session management with checkpoint support

#### CLI Product
- `sh run` - Execute agent tasks
- `sh session` - Manage sessions (list/resume/delete)
- `sh config` - Configure providers (init/add-provider/use/show)
- TUI mode with interactive interface

#### Configuration System
- Environment variables support (`SH_*`)
- TOML configuration files
- Environment variable references (`${VAR}`)
- Priority chain: env > file > default

### Features

#### Agent Runtime
- Async agent execution
- Tool calling with confirmation for dangerous operations
- Streaming response support
- Hook system for lifecycle events

#### Session Manager
- Concurrent session management
- Checkpoint save/rollback
- Session persistence
- History tracking with stats

#### Tool Registry
- Built-in tools: read/write/edit files, grep, glob, bash
- Custom tool registration via decorator
- Tool schema auto-inference
- Category-based organization

#### Workflow Engine
- DAG execution with topological sort
- Parallel execution support
- Cycle detection
- ASCII visualization

#### MCP Bridge
- Model Context Protocol integration
- Server discovery
- Tool synchronization

#### Audit Logger
- Action logging
- Secret access tracking
- Audit report generation

### Testing

#### Rust Tests
- Layer 0: Input validation, PII scrubbing tests
- Layer 1: Config, cache, error handling tests
- Layer 2: Checkpoint system tests (atomic write, crash recovery)
- Layer 3: Document loader tests
- Total: 228 tests passing

#### Python Tests
- SDK tests: Agent, Session, Tool (79 tests)
- Integration tests: CLI run/session/config (123 tests)
- E2E scenarios: QA, conversation, tool calling (23 tests)
- Config tests: env vars, TOML, providers (95 tests)
- API validation: Anthropic/OpenAI/Gemini/Custom (28 tests)
- Total: 218+ tests passing

#### Performance Benchmarks
- Session creation: <1ms
- Checkpoint write: <10ms atomic
- Tool execution: <100ms average
- LLM response: provider dependent

### Documentation

- API Design Draft (`docs/API_DESIGN_DRAFT.md`)
- Architecture documentation
- Test reports (`docs/test/`)
- Review reports (`docs/review/`)
- Example code (`examples/`)

### Contributors

- **Terminal 1**: Python SDK, Config API, PyPI packaging
- **Terminal 2**: Rust Core (Layer 0-5), CLI, crates.io packaging
- **Terminal 3**: Testing, Review, Documentation

---

## [0.1.0] - 2026-05-09

### Added
- Initial project structure
- Basic Rust scaffolding
- Python SDK skeleton
- Multi-terminal workflow setup

---

## Release Notes Template

Each release includes:
- Version number and date
- Summary of changes
- Breaking changes (if any)
- Deprecation notices (if any)
- Security fixes (if any)
- Upgrade instructions