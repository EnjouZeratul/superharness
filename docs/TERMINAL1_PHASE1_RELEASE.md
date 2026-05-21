# Terminal 1 任务清单 - 第一阶段: PyPI发布准备

> 分配时间: 2026-05-12
> 阶段: 发布基础
> 目标: 完成 Python SDK 的 PyPI 发布准备

---

## 🎯 任务分工

```
Terminal 1 擅长: Python SDK、用户接口
本次任务: PyPI 发布准备 + SDK 文档基础
```

---

## 任务清单

### P1.1: pyproject.toml 完善发布配置 ✅
- [x] 检查 `python/pyproject.toml` 发布配置
  - [x] name: `continuum`
  - [x] version: `1.0.0`
  - [x] description、readme、license
  - [x] authors、classifiers
  - [x] dependencies、optional-dependencies
- [x] 添加构建配置 (hatchling)
- [x] 实际时间: ~0.3小时

### P1.2: Python 包结构整理 ✅
- [x] 确保 `python/continuum_sdk/` 结构正确
- [x] 检查 `__init__.py` 导出
- [x] 添加 README.md
- [x] 检查类型提示文件 (`py.typed`) - 已添加到 continuum/ 和 continuum_sdk/
- [x] 实际时间: ~0.3小时

### P1.3: 本地安装测试 ✅
- [x] 构建 wheel: `hatchling build`
- [x] 本地安装测试: `pip install ./dist/continuum-1.0.0-py3-none-any.whl`
- [x] 验证导入: `python -c "from continuum import Agent"`
- [x] 验证功能: 79 测试全部通过
- [x] 实际时间: ~0.5小时

### P1.4: SDK API 文档基础 ✅
- [x] 添加 docstring 到核心模块
  - [x] `agent/runtime.py`
  - [x] `agent/session.py`
  - [x] `config/__init__.py`
- [x] 确保 `help(Agent)` 显示有用信息
- [x] 实际时间: ~0.5小时

### P1.5: TestPyCI 发布验证 (可选) ⏭️
- [ ] **需要**: 用户提供 PyPI API Token
- [x] 如无 Token: 跳过此步骤，由用户手动验证
- [ ] 状态: 等待用户 Token

---

## 工作目录

```
python/
├── pyproject.toml      ← P1.1
├── MANIFEST.in         ← P1.2 (如需)
├── continuum_sdk/
│   ├── __init__.py     ← P1.2 导出
│   └── py.typed        ← P1.2 类型标记
└── tests/              ← P1.3 验证
```

---

## pyproject.toml 示例

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "continuum"
version = "1.0.0"
description = "简洁可靠的 Agent 运行时"
readme = "README.md"
license = {text = "MIT"}
requires-python = ">=3.8"
authors = [
    {name = "Continuum Team"}
]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Rust",
]
dependencies = [
    "pydantic>=2.0",
    "toml>=0.10",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "pytest-asyncio>=0.21",
]

[project.urls]
Homepage = "https://github.com/continuum/continuum"
Documentation = "https://continuum.readthedocs.io"
Repository = "https://github.com/continuum/continuum"

[tool.maturin]
features = ["python-extension"]
```

---

## 自检清单

```
[x] pyproject.toml 配置正确
[x] hatchling build 成功
[x] pip install 本地 wheel 成功
[x] from continuum import Agent 成功
[x] 核心模块 docstring 完整
[ ] TestPyPI 上传成功 (等待 Token)
[ ] TestPyPI 安装验证通过 (等待 Token)
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. crates.io 发布 → Terminal 2
2. CI/CD 配置 → Terminal 2
3. 发布说明编写 → Terminal 3
4. 发布后验证 → Terminal 3
```

---

## ⚡ 关键通知点

```
完成 P1.1-P1.4 后通知:
┌────────────────────────────────────────────┐
│  📢 通知 Terminal 0:                       │
│  "Terminal 1 完成 PyPI 发布准备"            │
│  "本地测试通过，等待用户验证"               │
└────────────────────────────────────────────┘
```

---

## 📋 用户需要提供 (可选)

```
如需 TestPyCI 验证:
- PyPI API Token (从 pypi.org 获取)

如无 Token:
- 终端跳过 TestPyPI 步骤
- 由用户手动验证发布
```

---

## 完成标准

- [x] pyproject.toml 配置完成
- [x] 本地安装测试通过 (79 测试全部通过)
- [ ] TestPyPI 验证通过 (等待用户 Token)
- [x] SDK 文档基础完成
- [x] 更新本文档状态为完成 (P1.1-P1.4)

---
> **状态**: P1.1-P1.4 完成，等待用户验证或提供 PyPI Token
> **完成时间**: 2026-05-12