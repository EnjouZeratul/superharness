# 审查报告: 错误处理链

## 审查人: Terminal 3 (测试视角)

## 审查范围

错误从 Rust 到 Python 到 CLI 的传递链路。

---

## 整体评价

- [x] 良好 / [ ] 优秀 / [ ] 需改进

---

## 错误处理架构

### Rust 层错误定义

**Layer 1 - ShError 统一错误类型**:
```rust
#[derive(Debug, Error)]
pub enum ShError {
    #[error("Layer 0 error: {0}")]
    Layer0(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("LLM API error: {0}")]
    LlmApi(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Timeout error after {seconds} seconds")]
    Timeout { seconds: u64 },

    // ...
}
```

**优点**:
- 统一错误类型
- 层级分明
- 使用 thiserror 宏简化

### Python 层错误处理

**SDK 异常定义**:
```python
# tools/builtin.py
class ToolExecutionError(Exception):
    """工具执行错误"""
    pass

# workflow/dag.py
class DAGValidationError(Exception):
    """DAG 验证错误"""
    pass
```

**问题**: 缺少与 Rust ShError 对应的统一异常类型

### CLI 层错误展示

**输出格式化**:
- `output/format.rs` 提供格式化
- 错误信息应经过处理

---

## 测试友好性评分

| 项目 | 评分 | 说明 |
|------|------|------|
| Rust 错误可追溯 | 4/5 | ShError 类型统一 |
| Python 异常可追溯 | 3/5 | 缺少统一异常层次 |
| CLI 错误友好性 | 3/5 | 待验证实际输出 |
| 错误测试覆盖 | 2/5 | 错误场景测试不足 |

---

## 错误传递链分析

### 设计的错误传递链

```
Rust Layer:
  ShError (Layer 1)
    ↓ PyO3
Python SDK:
  PythonException
    ↓
CLI:
  Formatted Output
```

### 实际情况

| 链路 | 状态 | 问题 |
|------|------|------|
| Rust → Rust | ✅ 完整 | anyhow → ShError 转换 |
| Rust → Python | ⏸️ 未完成 | PyO3 绑定未实现 |
| Python → Python | ⚠️ 部分 | 缺少统一异常基类 |
| Python → CLI | ⚠️ 部分 | CLI 错误处理待验证 |

---

## 发现的问题

### 问题 1: Python 缺少统一异常层次

当前各模块独立定义异常：
- `ToolExecutionError` (tools)
- `DAGValidationError` (workflow)
- 无 `SessionError`, `AgentError`

**建议**: 创建统一异常层次：
```python
class ShError(Exception):
    """Continuum 基础异常"""
    pass

class ConfigError(ShError):
    """配置错误"""
    pass

class SessionError(ShError):
    """会话错误"""
    pass

class ToolError(ShError):
    """工具错误"""
    pass
```

### 问题 2: PyO3 异常绑定缺失

Rust ShError 未通过 PyO3 映射到 Python。

**建议**: 在 sh-core 中添加：
```rust
#[pymodule]
fn sh_errors(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("ShError", py.get_type::<PyShError>())?;
    Ok(())
}
```

### 问题 3: 错误场景测试不足

错误测试覆盖率低：
- `test_sdk_tools.py`: 仅 3 错误测试
- `test_sdk_agent.py`: 仅 3 错误测试

**建议**: 增加错误场景测试：
- API 超时
- API 认证失败
- 工具执行失败
- Session 恢复失败

### 问题 4: CLI 错误提示待验证

CLI 错误输出格式未在测试中验证。

**建议**: 添加 CLI 错误输出测试：
```python
def test_cli_api_error_output():
    result = subprocess.run(["sh", "run"], capture_output=True)
    assert "API error" in result.stderr
```

---

## 错误处理建议

### 1. Rust 层改进

```rust
// 添加错误上下文
impl ShError {
    pub fn with_context(self, ctx: &str) -> Self {
        match self {
            ShError::Session(s) => ShError::Session(format!("{}: {}", ctx, s)),
            // ...
        }
    }
}
```

### 2. Python 层改进

```python
# errors.py
class ShError(Exception):
    def __init__(self, message, layer=None, code=None):
        self.layer = layer  # 来源层级
        self.code = code    # 错误码
        super().__init__(message)

    def __str__(self):
        if self.layer:
            return f"[{self.layer}] {super().__str__()}"
        return super().__str__()
```

### 3. CLI 层改进

```rust
// output/format.rs
fn format_error(e: &ShError) -> String {
    match e {
        ShError::Config(msg) => format!("❌ 配置错误: {}", msg),
        ShError::LlmApi(msg) => format!("❌ API 错误: {}", msg),
        ShError::Timeout { seconds } => format!("⏱️ 超时 ({}秒)", seconds),
        // ...
    }
}
```

---

## 结论

- [x] 可以通过（良好，有改进空间）
- [ ] 需要修改后通过
- [ ] 需要重大修改

Rust 错误设计良好，Python 和 CLI 需补充。

**优先改进项**:
1. 创建 Python 统一异常层次
2. 完成 PyO3 异常绑定
3. 增加错误场景测试

---

*审查完成时间: 2026-05-11*
*审查人: Terminal 3*