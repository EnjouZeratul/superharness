# Contributing to Continuum Agent SDK

[中文](#中文版) | **English**

---

## Quick Start

### Development Setup

```bash
# 1. Clone repository
git clone https://github.com/EnjouZeratul/continuum.git
cd continuum

# 2. Install Rust
rustup install stable

# 3. Install Python dependencies
cd python
pip install -e ".[dev]"

# 4. Run tests
pytest tests/ -v --cov
cargo test
```

### Project Structure

```
continuum/
├── python/continuum_sdk/    # Python SDK
│   ├── agent/               # Agent core logic
│   ├── llm/                 # LLM clients
│   ├── tools/               # Built-in tools
│   ├── workflow/            # DAG workflow
│   └── config/              # Configuration
├── src/                     # Rust CLI core
├── docs/                    # Documentation
└── tests/                   # Tests
```

## Submitting Code

### Branch Naming

- `feature/xxx` - New features
- `fix/xxx` - Bug fixes
- `docs/xxx` - Documentation updates
- `test/xxx` - Test improvements

### Commit Message Format

```
<type>: <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Refactoring
- `chore`: Miscellaneous

**Example:**
```
feat: add session persistence support

- Add Session.save() and Session.load() methods
- Support JSON export/import
- Add session directory configuration

Closes #123
```

### PR Process

1. Create feature branch
2. Write code + tests
3. Run local checks:
   ```bash
   # Python
   ruff check python/
   pytest python/tests/ -v --cov
   
   # Rust
   cargo fmt --check
   cargo clippy
   cargo test
   ```
4. Submit PR, wait for CI
5. Merge after review approval

## Testing Standards

### Quality Requirements

- **No placeholder tests**: `assert True`, `pass`, no assertions are forbidden
- **AAA pattern**: Arrange-Act-Assert structure
- **Naming**: `test_<feature>_<scenario>` format
- **Coverage target**: Core modules ≥ 80%

See [`docs/TESTING_STANDARDS.md`](docs/TESTING_STANDARDS.md)

### Mock Usage

LLM client tests must use Mock, no real API calls:

```python
from unittest.mock import AsyncMock, patch

with patch.object(client, '_make_request', new_callable=AsyncMock) as mock:
    mock.return_value = expected_response
    result = await client.chat(messages)
```

## Code Style

### Python

- Ruff check: `ruff check python/`
- Type hints: Public functions must have type annotations
- Docstrings: Core classes/functions must have docstrings

### Rust

- Standard format: `cargo fmt`
- No Clippy warnings: `cargo clippy`
- Public APIs must have doc comments

## Reporting Issues

- Bug reports: [GitHub Issues](https://github.com/EnjouZeratul/continuum/issues)
- Feature requests: Open an Issue to discuss first
- Security issues: Contact maintainers privately

## License

This project uses MIT license. Contributed code will be under the same license.

---

# 中文版

**English** | [中文](#中文版)

---

## 快速开始

### 开发环境设置

```bash
# 1. 克隆仓库
git clone https://github.com/EnjouZeratul/continuum.git
cd continuum

# 2. 安装 Rust
rustup install stable

# 3. 安装 Python 依赖
cd python
pip install -e ".[dev]"

# 4. 运行测试
pytest tests/ -v --cov
cargo test
```

### 项目结构

```
continuum/
├── python/continuum_sdk/    # Python SDK
│   ├── agent/               # Agent 核心逻辑
│   ├── llm/                 # LLM 客户端
│   ├── tools/               # 内置工具
│   ├── workflow/            # DAG 工作流
│   └── config/              # 配置管理
├── src/                     # Rust CLI 核心
├── docs/                    # 文档
└── tests/                   # 测试
```

## 提交代码

### 分支命名

- `feature/xxx` - 新功能
- `fix/xxx` - Bug 修复
- `docs/xxx` - 文档更新
- `test/xxx` - 测试改进

### 提交消息格式

```
<type>: <subject>

<body>

<footer>
```

**Type 类型：**
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档
- `test`: 测试
- `refactor`: 重构
- `chore`: 杂项

**示例：**
```
feat: add session persistence support

- Add Session.save() and Session.load() methods
- Support JSON export/import
- Add session directory configuration

Closes #123
```

### PR 流程

1. 创建功能分支
2. 编写代码 + 测试
3. 运行本地检查：
   ```bash
   # Python
   ruff check python/
   pytest python/tests/ -v --cov
   
   # Rust
   cargo fmt --check
   cargo clippy
   cargo test
   ```
4. 提交 PR，等待 CI 通过
5. Review 通过后合并

## 测试标准

### 测试质量要求

- **禁止占位测试**：不允许 `assert True`、`pass`、无断言
- **AAA 模式**：Arrange-Act-Assert 结构
- **命名规范**：`test_<功能>_<场景>` 格式
- **覆盖率目标**：核心模块 ≥ 80%

详见 [`docs/TESTING_STANDARDS.md`](docs/TESTING_STANDARDS.md)

### Mock 使用

LLM 客户端测试必须使用 Mock，禁止真实 API 调用：

```python
from unittest.mock import AsyncMock, patch

with patch.object(client, '_make_request', new_callable=AsyncMock) as mock:
    mock.return_value = expected_response
    result = await client.chat(messages)
```

## 代码风格

### Python

- 使用 Ruff 检查：`ruff check python/`
- 类型注解：公开函数必须有类型注解
- Docstring：核心类/函数必须有 docstring

### Rust

- 使用标准格式：`cargo fmt`
- Clippy 无警告：`cargo clippy`
- 公开 API 必有文档注释

## 问题反馈

- Bug 报告：使用 [GitHub Issues](https://github.com/EnjouZeratul/continuum/issues)
- 功能建议：先开 Issue 讨论，再提交 PR
- 安全问题：私下联系维护者

## 许可证

本项目采用 MIT 许可证，贡献的代码将同样适用。