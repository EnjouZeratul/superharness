# 审查报告: 整体架构

## 审查人: Terminal 3 (测试视角)

## 审查范围

整体分层架构审查，关注测试友好性和可维护性。

---

## 整体评价

- [x] 优秀 / [ ] 良好 / [ ] 需改进

---

## 架构概述

Continuum 采用6层分层架构：

| 层级 | 名称 | 职责 | 代码量 |
|------|------|------|--------|
| Layer 0 | Security Gateway | 安全网关、输入验证、PII清洗 | ~500 lines |
| Layer 1 | Foundation | 基础设施（配置、缓存、LLM客户端） | ~2000 lines |
| Layer 2 | Core Engine | Agent运行时、Session管理、工具注册 | ~3000 lines |
| Layer 3 | Capabilities | 扩展能力（文档加载、搜索等） | ~5500 lines |
| Layer 4 | Integration | MCP集成、Plugin加载 | ~800 lines |
| Layer 5 | Interface | Python SDK、CLI | ~700 lines (Python) |

**总代码量**: ~18551 lines Rust + ~700 lines Python SDK

---

## 测试友好性评分

| 项目 | 评分 | 说明 |
|------|------|------|
| 可测试性 | 4/5 | 各层模块独立，接口清晰，便于单元测试 |
| 错误可追溯 | 4/5 | 统一 ShError 类型，层次分明 |
| 状态可观测 | 3/5 | 部分模块缺少状态导出接口 |
| 文档完整性 | 4/5 | 模块注释清晰，公开 API 有文档 |

---

## 架构优点

### 1. 清晰的分层职责
```
Layer 0: 安全边界 - 所有输入先经过验证
Layer 1: 基础设施 - 无业务逻辑，纯技术能力
Layer 2: 核心引擎 - Agent/Session/Tool 核心抽象
Layer 3: 扩展能力 - 可选模块，不影响核心
Layer 4: 外部集成 - MCP/Plugin，隔离外部依赖
Layer 5: 用户接口 - SDK/CLI，面向用户
```

### 2. Trait-based 设计
每个核心组件都定义了 Trait：
- `AgentRuntimeTrait`
- `SessionManagerTrait`
- `ToolRegistryTrait`
- `WorkflowEngineTrait`
- `HookSystemTrait`
- `CheckpointSystemTrait`

**测试优势**: 可轻松 Mock 各组件进行单元测试

### 3. 模块化导出
每层 lib.rs 清晰导出：
```rust
// Layer 2 示例
pub mod traits {
    pub use super::agent_runtime::AgentRuntimeTrait;
    pub use super::session_manager::SessionManagerTrait;
    // ...
}
```

**测试优势**: 测试可直接导入 Trait 而不依赖具体实现

---

## 发现的问题

### 问题 1: Layer 间依赖未显式声明
当前各层通过 `use` 直接引用，未在 Cargo.toml 中声明依赖关系。

**建议**: 在 Cargo.toml 中显式声明：
```toml
[dependencies]
sh-layer0 = { path = "../layer0" }
sh-layer1 = { path = "../layer1" }
```

### 问题 2: 状态可观测性不足
部分组件缺少 `stats()` 或 `status()` 方法：
- `SecurityGateway`: 无状态导出
- `ToolRegistry`: 无注册统计
- `HookSystem`: 无回调计数

**测试影响**: 无法观测内部状态变化

### 问题 3: Python SDK 缺少 Rust 绑定
当前 Python SDK 模块为纯 Python 实现，未通过 PyO3 绑定 Rust 核心。

**测试影响**: SDK 测试无法验证 Rust 层调用链

---

## 测试架构建议

### 1. 分层测试策略
```
Layer 0: 纯单元测试，无外部依赖
Layer 1: 单元测试 + 集成测试（Mock LLM）
Layer 2: 单元测试 + 并发测试
Layer 3: 功能测试 + 性能测试
Layer 4: 集成测试（需要外部服务）
Layer 5: E2E测试 + API验证
```

### 2. 测试覆盖率目标
| 层级 | 单元测试 | 集成测试 | 目标覆盖率 |
|------|----------|----------|------------|
| Layer 0 | 必须 | 可选 | 80% |
| Layer 1 | 必须 | 必须 | 70% |
| Layer 2 | 必须 | 必须 | 75% |
| Layer 5 | 可选 | 必须 | 60% |

### 3. Mock 边界
```
测试边界:
- LLM Client → Mock
- Storage Engine → 内存实现
- MCP Server → Mock
- API Endpoint → Mock
```

---

## 结论

- [x] 可以通过
- [ ] 需要修改后通过
- [ ] 需要重大修改

架构设计清晰，分层合理，测试友好性良好。

**建议改进项**:
1. 补充状态可观测接口
2. 显式声明 Layer 依赖
3. 完成 Python SDK PyO3 绑定

---

*审查完成时间: 2026-05-11*
*审查人: Terminal 3*