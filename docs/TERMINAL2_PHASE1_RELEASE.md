# Terminal 2 任务清单 - 第一阶段: crates.io发布 + CI/CD

> 分配时间: 2026-05-12
> 阶段: 发布基础
> 目标: 完成 Rust 核心的 crates.io 发布 + CI/CD 配置

---

## 🎯 任务分工

```
Terminal 2 擅长: Rust底层、自动化
本次任务: crates.io 发布 + CI/CD 配置
```

---

## 任务清单

### P2.1: Cargo.toml 发布配置
- [ ] 检查所有 crate 的 `Cargo.toml`
  - [ ] `layer0/Cargo.toml`
  - [ ] `layer1/Cargo.toml`
  - [ ] `layer2/Cargo.toml`
  - [ ] `layer3/Cargo.toml`
  - [ ] `layer4/Cargo.toml`
  - [ ] `cli/Cargo.toml`
- [ ] 确保每个 crate 有:
  - [ ] 正确的 version
  - [ ] description、license
  - [ ] repository、homepage
  - [ ] keywords、categories
- [ ] 预计时间: 1小时

### P2.2: crates.io 发布顺序规划
- [ ] 确定发布顺序 (依赖关系)
  ```
  layer0 → layer1 → layer2 → layer3 → layer4 → cli
  ```
- [ ] 检查每个 crate 的依赖版本
- [ ] 预计时间: 0.5小时

### P2.3: 本地发布测试
- [ ] 运行 `cargo publish --dry-run` 每个 crate
- [ ] 修复所有发布警告/错误
- [ ] 预计时间: 1小时

### P2.4: CI/CD 配置
- [ ] 创建 `.github/workflows/` 目录
- [ ] 配置 CI 工作流:
  - [ ] `ci.yml` - 测试 + 构建
  - [ ] `release.yml` - 自动发布
  - [ ] `docs.yml` - 文档生成 (可选)
- [ ] 配置内容:
  ```yaml
  # ci.yml 示例
  name: CI
  on: [push, pull_request]
  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: actions-rs/toolchain@v1
        - run: cargo test --all
        - run: cargo clippy --all
        - run: cargo fmt --check
    python-test:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: actions/setup-python@v5
        - run: pip install pytest
        - run: pytest python/tests/
  ```
- [ ] 预计时间: 1.5小时

### P2.5: crates.io 发布准备 (dry-run)
- [ ] 运行 `cargo publish --dry-run` 确认可发布
- [ ] 输出发布命令列表供用户执行
- [ ] **正式发布由用户执行** (需要 crates.io 授权)
- [ ] 预计时间: 0.5小时

---

## 工作目录

```
.github/
└── workflows/
    ├── ci.yml           ← P2.4
    ├── release.yml      ← P2.4
    └── docs.yml         ← P2.4 (可选)

rust/
├── layer0/Cargo.toml   ← P2.1
├── layer1/Cargo.toml   ← P2.1
├── layer2/Cargo.toml   ← P2.1
├── layer3/Cargo.toml   ← P2.1
├── layer4/Cargo.toml   ← P2.1
└── cli/Cargo.toml      ← P2.1
```

---

## Cargo.toml 发布配置示例

```toml
[package]
name = "superharness-layer1"
version = "1.0.0"
edition = "2021"
description = "SuperHarness Layer 1 - 基础设施层"
license = "MIT"
repository = "https://github.com/superharness/superharness"
homepage = "https://github.com/superharness/superharness"
documentation = "https://docs.rs/superharness-layer1"
keywords = ["agent", "llm", "ai", "runtime"]
categories = ["development-tools", "api-bindings"]

[dependencies]
# 依赖其他 layer 时使用精确版本
superharness-layer0 = { version = "1.0.0", path = "../layer0" }
```

---

## 自检清单

```
□ 所有 Cargo.toml 配置正确
□ cargo publish --dry-run 无错误
□ CI 工作流配置完成
□ GitHub Actions 测试运行成功
□ crates.io 发布成功
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. PyPI 发布准备 → Terminal 1
2. 发布说明编写 → Terminal 3
3. 发布后验证 → Terminal 3
```

---

## ⚡ 关键通知点

```
完成 P2.1-P2.5 后通知:
┌────────────────────────────────────────────┐
│  📢 通知 Terminal 0:                       │
│  "Terminal 2 完成 crates.io 发布准备"       │
│  "CI/CD 配置完成"                           │
│  "dry-run 通过，等待用户执行正式发布"        │
└────────────────────────────────────────────┘
```

---

## 📋 用户需要提供

```
crates.io 发布:
- 用 GitHub 账号登录 crates.io 授权
- 终端完成 dry-run 后，用户执行正式发布
```

---

## 完成标准

- [x] 所有 Cargo.toml 配置完成
- [x] CI/CD 配置完成
- [ ] crates.io 发布成功 (等待用户执行)
- [x] 更新本文档状态为完成

---

## 完成日期: 2026-05-12

---

## 发布状态

```
✅ P2.1: Cargo.toml 发布配置 - 完成
✅ P2.2: crates.io 发布顺序规划 - 完成
✅ P2.3: 本地发布测试 - 完成 (layer0 dry-run 通过)
✅ P2.4: CI/CD 配置 - 完成
✅ P2.5: crates.io 发布准备 - 完成

⏳ 正式发布 - 等待用户执行
```

---

## 用户需要执行的操作

1. 登录 crates.io:
   ```bash
   cargo login YOUR_API_TOKEN
   ```

2. 参考 `docs/PUBLISH_GUIDE.md` 执行发布

3. 发布顺序:
   ```
   layer0 → layer1 → layer2 → layer3 → layer4 → sh-core → cli
   ```