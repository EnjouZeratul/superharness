# SuperHarness 设计理念与代码规范

> 版本: v1.0
> 日期: 2026-05-10
> 核心理念: 优雅、简洁、可维护

---

## 一、核心设计原则

### 1.1 六层架构原则

```
Layer N 只依赖 Layer N-1
├── 单向依赖：永远向下，永不向上
├── 无循环：任何形式的循环依赖都是禁止的
├── 接口隔离：每层只暴露必要接口
└── 安全边界：Layer 0 是所有外部输入的入口
```

### 1.2 Rust vs Python 分工

| Rust 负责 | Python 负责 |
|-----------|-------------|
| 性能关键路径 | 业务逻辑 |
| 并发安全 | 配置管理 |
| 内存敏感 | 快速迭代 |
| 安全敏感 | 接口友好 |

---

## 二、代码质量规范

### 2.1 单文件单职责

```
每个文件只做一件事

✅ 正确示例:
├── input_validator.rs    # 只负责输入验证
├── pii_scrubber.rs      # 只负责PII清洗
├── rate_limiter.rs      # 只负责速率限制

❌ 错误示例:
├── security.rs          # 包含验证、清洗、限制等
├── utils.rs             # 大杂烩工具函数
├── helpers.rs           # 不明确的辅助函数
```

**判断标准**：
- 文件名能清楚描述职责
- 文件内容可以用一句话概括
- 修改该功能只需修改一个文件

### 2.2 无创世文件

```
禁止创建"上帝文件"或"万能类"

❌ 禁止:
├── mod.rs 包含大量实现代码
├── lib.rs 直接实现功能而非导出
├── Manager/Handler/Utils 命名的万能类
└── 超过500行的单个文件

✅ 正确:
├── mod.rs 仅用于导出和重导出
├── lib.rs 仅用于模块组织
├── 功能拆分到独立文件
└── 超过300行考虑拆分
```

**拆分策略**：
```rust
// ❌ 错误：一个大文件
// session.rs (1000行)

// ✅ 正确：按职责拆分
// session/
// ├── mod.rs          # 导出
// ├── manager.rs      # 会话管理
// ├── state.rs        # 状态处理
// ├── storage.rs      # 存储逻辑
// └── concurrency.rs  # 并发控制
```

### 2.3 复用可复用接口

```
优先组合，其次继承
优先接口，其次实现

// ✅ 正确：定义trait接口
pub trait Validator {
    fn validate(&self, input: &str) -> Result<ValidationResult>;
}

// ✅ 正确：组合使用
pub struct CompositeValidator {
    validators: Vec<Box<dyn Validator>>,
}

// ❌ 错误：硬编码依赖
pub struct InputHandler {
    validator: InputValidator,  // 直接依赖具体类型
}
```

**接口设计原则**：
```rust
// 接口应该小而专注
pub trait Reader {
    fn read(&self) -> Result<String>;
}

pub trait Writer {
    fn write(&self, data: &str) -> Result<()>;
}

// 而不是大而全
pub trait Storage {  // ❌ 太大
    fn read(&self) -> Result<String>;
    fn write(&self, data: &str) -> Result<()>;
    fn delete(&self) -> Result<()>;
    fn exists(&self) -> bool;
    // ...更多方法
}
```

---

## 三、命名规范

### 3.1 文件命名

```
Rust: snake_case.rs
├── input_validator.rs
├── session_manager.rs
├── llm_client.rs

Python: snake_case.py
├── input_validator.py
├── session_manager.py
├── llm_client.py

禁止:
├── InputValidator.rs    # PascalCase
├── inputValidator.rs    # camelCase
├── input-validator.rs   # kebab-case
```

### 3.2 类型命名

```rust
// 结构体：PascalCase
pub struct SessionManager {}
pub struct InputValidator {}

// 枚举：PascalCase
pub enum SessionState {}
pub enum ValidationResult {}

// trait：PascalCase，通常是能力描述
pub trait Read {}      // 能力
pub trait Validate {}  // 能力

// 函数/方法：snake_case
pub fn create_session() {}
pub fn validate_input() {}

// 常量：SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: usize = 3;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
```

### 3.3 模块命名

```
// Cargo.toml 中的包名
sh-layer0  # sh = SuperHarness 缩写，layer0 表示层级
sh-layer1
sh-core

// Python 模块
superharness_sdk
superharness_cli
```

---

## 四、错误处理规范

### 4.1 错误类型定义

```rust
// 每个模块定义自己的错误类型
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Input too long: {actual} bytes (max {max})")]
    TooLong { actual: usize, max: usize },

    #[error("Forbidden pattern detected: {pattern}")]
    ForbiddenPattern { pattern: String },

    #[error("Input is empty")]
    Empty,
}

// 使用 anyhow 作为上层错误传播
pub type Result<T> = std::result::Result<T, anyhow::Error>;
```

### 4.2 错误传播

```rust
// ✅ 正确：使用 ? 操作符
pub fn process(&self, input: &str) -> Result<String> {
    let validated = self.validator.validate(input)?;
    let cleaned = self.scrubber.scrub(&validated)?;
    Ok(cleaned)
}

// ❌ 错误：unwrap/expect
pub fn process(&self, input: &str) -> String {
    self.validator.validate(input).unwrap()  // 危险！
}

// ✅ 正确：提供上下文
pub fn process(&self, input: &str) -> Result<String> {
    let validated = self.validator.validate(input)
        .map_err(|e| anyhow::anyhow!("Validation failed for input: {}", e))?;
    Ok(validated)
}
```

---

## 五、测试规范

### 5.1 测试文件组织

```
// 单元测试：在源文件内的 #[cfg(test)] 模块
// src/input_validator.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_input() { ... }
}

// 集成测试：在 tests/ 目录
tests/
├── integration/
│   ├── test_session_lifecycle.rs
│   └── test_tool_execution.rs
└── e2e/
    └── test_cli_basic.rs
```

### 5.2 测试命名

```rust
// 测试函数命名：test_<功能>_<场景>_<预期结果>
#[test]
fn test_validate_valid_input_returns_ok() {}

#[test]
fn test_validate_empty_input_returns_error() {}

#[test]
fn test_scrubber_detects_email() {}
```

---

## 六、文档规范

### 6.1 模块文档

```rust
//! # 模块名称
//!
//! 模块的简短描述。
//!
//! ## 示例
//!
//! ```rust
//! use sh_layer0::InputValidator;
//!
//! let validator = InputValidator::new();
//! let result = validator.validate("Hello")?;
//! ```
```

### 6.2 函数文档

```rust
/// 验证输入字符串
///
/// # 参数
///
/// * `input` - 要验证的输入字符串
///
/// # 返回
///
/// 验证结果，包含是否有效和错误信息
///
/// # 错误
///
/// 当输入超过最大长度时返回错误
///
/// # 示例
///
/// ```rust
/// let validator = InputValidator::new();
/// let result = validator.validate("Hello")?;
/// assert!(result.valid);
/// ```
pub fn validate(&self, input: &str) -> Result<ValidationResult> {
    // ...
}
```

---

## 七、代码审查清单

### 7.1 提交前检查

```
□ 文件名符合 snake_case 规范
□ 每个文件不超过 500 行
□ 没有 unwrap/expect（除非测试）
□ 公开 API 有文档注释
□ 新功能有测试覆盖
□ 错误处理使用 thiserror/anyhow
□ 没有 todo!/unimplemented!
□ 导入按标准/第三方/本地分组
□ clippy 无警告
□ fmt 格式化通过
```

### 7.2 架构检查

```
□ 遵守层级依赖规则
□ 新模块放在正确的 layer
□ 公开接口最小化
□ 没有循环依赖
□ 安全敏感代码在 Layer 0
```

---

## 八、版本控制规范

### 8.1 提交信息

```
<类型>(<范围>): <描述>

类型:
├── feat: 新功能
├── fix: 修复bug
├── docs: 文档变更
├── style: 格式调整（不影响功能）
├── refactor: 重构
├── test: 测试相关
└── chore: 构建/工具相关

示例:
├── feat(layer0): add input validation module
├── fix(session): handle concurrent access properly
├── docs(architecture): update layer dependencies
└── refactor(llm-client): extract retry logic
```

### 8.2 分支策略

```
main          # 稳定发布
├── develop   # 开发集成
│   ├── feature/layer0-security
│   ├── feature/layer1-foundation
│   └── feature/cli-tui
└── release/  # 发布准备
```

---

## 九、性能规范

### 9.1 避免的性能问题

```rust
// ❌ 避免：频繁堆分配
for item in items {
    let s = item.to_string();  // 循环内分配
}

// ✅ 正确：减少分配
let s = items.iter().map(|i| i.as_str()).collect::<Vec<_>>();

// ❌ 避免：不必要的克隆
fn process(s: String) { }  // 每次调用都克隆

// ✅ 正确：借用
fn process(s: &str) { }  // 零成本

// ❌ 避免：阻塞异步运行时
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));  // 阻塞！
}

// ✅ 正确：使用异步睡眠
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

### 9.2 并发安全

```rust
// ✅ 正确：使用 parking_lot 的 RwLock
use parking_lot::RwLock;

pub struct SessionStore {
    sessions: RwLock<HashMap<String, Session>>,
}

// 读取
let sessions = self.sessions.read();
let session = sessions.get(id);

// 写入
let mut sessions = self.sessions.write();
sessions.insert(id, session);
```

---

## 十、总结

### 核心公式

```
优雅代码 = 单一职责 + 清晰命名 + 最小接口 + 充分测试
```

### 一句话原则

> **每个文件做一件事，每个函数说一句话，每个接口暴露一个能力。**

---

**文档状态**: v1.0 设计理念完成
**适用范围**: SuperHarness 全部代码
