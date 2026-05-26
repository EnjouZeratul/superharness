# 审查标准 - 非DEMO/MVP验证

## 核心原则

**所有代码必须是生产级实现，禁止：**
- ❌ TODO/FIXME/STUB 标记
- ❌ 硬编码返回值
- ❌ 空函数体 (pass, todo!(), unimplemented!())
- ❌ 模拟数据替换真实实现
- ❌ 简化版逻辑（如用正则代替真实解析器）
- ❌ 缺少错误处理
- ❌ 缺少测试覆盖

---

## 审查员职责矩阵

| 审查员 | 审查范围 | 验证点 |
|--------|----------|--------|
| reviewer-rust | Rust 核心实现 | 1. 无 todo!/unimplemented!()
2. LLM API 真实调用（抓包验证）
3. 错误处理完整（所有 Result 都有处理）
4. 并发安全（Send/Sync 正确）
5. 性能达标（无阻塞操作） |
| reviewer-py | Python SDK 实现 | 1. 无 pass/NotImplementedError
2. 无硬编码测试数据
3. 类型注解完整
4. 异常处理完善
5. 文档字符串完整 |
| reviewer-cli | CLI/TUI 实现 | 1. 无 mock UI
2. 真实用户交互
3. 错误提示友好
4. 快捷键完整
5. 跨平台兼容 |
| reviewer-integration | 集成测试 | 1. 端到端流程通过
2. 真实 API 调用成功
3. 数据持久化验证
4. 多平台测试通过 |

---

## DEMO/MVP 特征检测清单

### 代码层面

```rust
// ❌ DEMO特征 - 硬编码
fn get_model() -> String {
    "gpt-4".to_string()  // 应从配置读取
}

// ✅ 生产级 - 配置驱动
fn get_model(config: &Config) -> String {
    config.model.clone()
}
```

```rust
// ❌ DEMO特征 - 模拟响应
fn call_llm(&self, prompt: &str) -> Result<String> {
    Ok(format!("Response to: {}", prompt))  // 未调用真实API
}

// ✅ 生产级 - 真实调用
fn call_llm(&self, prompt: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client.post(&self.api_url)
        .header("Authorization", format!("Bearer {}", self.api_key))
        .json(&json!({"prompt": prompt}))
        .send()?
        .json()?;
    Ok(resp)
}
```

```python
# ❌ DEMO特征 - 无错误处理
def read_file(path):
    with open(path) as f:
        return f.read()

# ✅ 生产级 - 完整错误处理
def read_file(path: str) -> Result[str, FileError]:
    try:
        with open(path, encoding='utf-8') as f:
            return Ok(f.read())
    except FileNotFoundError:
        return Err(FileError::NotFound(path))
    except PermissionError:
        return Err(FileError::PermissionDenied(path))
```

### 功能层面

| 功能 | DEMO特征 | 生产级特征 |
|------|----------|------------|
| LLM调用 | 返回固定字符串 | 真实API调用，有重试、超时、限流 |
| 向量搜索 | 内存中遍历 | 使用真实向量数据库或高效索引 |
| 文件操作 | 只读不写 | 完整CRUD，原子写入，锁机制 |
| CLI | 单命令执行 | TUI交互，历史记录，快捷键 |
| 错误处理 | print错误 | 结构化错误，用户友好提示，日志 |
| 配置 | 硬编码常量 | 配置文件，环境变量，命令行参数 |
| 测试 | 只有Happy Path | 边界条件，错误路径，并发，性能 |

---

## 审查员验收流程

### Step 1: 静态分析（自动化）

```bash
# 搜索所有DEMO特征
grep -r "todo!()" rust/
grep -r "unimplemented!()" rust/
grep -r "pass$" python/
grep -r "NotImplementedError" python/
grep -r "# TODO\|# FIXME\|# STUB" .
grep -r "simulate\|mock\|fake\|dummy" --include="*.rs" --include="*.py"
```

### Step 2: 功能验证（手动）

1. **LLM调用验证**
   - 运行测试用例，检查是否真实调用API
   - 检查API密钥是否被读取
   - 检查请求日志是否完整

2. **数据持久化验证**
   - 写入数据后重启进程
   - 验证数据是否保留

3. **错误处理验证**
   - 故意触发各种错误条件
   - 检查错误信息是否友好
   - 检查进程是否崩溃

### Step 3: 集成验证

```bash
# 端到端测试
continuum run "分析当前目录结构" --model claude-3-5-sonnet
# 验证：
# 1. 是否真实调用Claude API
# 2. 返回结果是否合理
# 3. 错误处理是否正确
# 4. 日志是否完整
```

---

## 审查报告模板

```markdown
## [模块名] 实现完整性审查报告

### 1. DEMO特征检测
- [ ] 无 todo!/unimplemented!/pass
- [ ] 无硬编码数据
- [ ] 无模拟函数

### 2. 功能完整性
- [ ] 所有功能点已实现
- [ ] 错误处理完整
- [ ] 边界条件覆盖

### 3. 测试覆盖
- [ ] 单元测试 ≥ 80%
- [ ] 集成测试通过
- [ ] 错误路径测试

### 4. 生产就绪
- [ ] 配置化完成
- [ ] 日志完善
- [ ] 文档完整

### 5. 问题清单
| 问题 | 严重程度 | 位置 | 状态 |
|------|----------|------|------|
| 示例：硬编码API Key | P0 | file.rs:42 | 待修复 |

### 结论
- [ ] 通过
- [ ] 需修改
- [ ] 不通过（重写）
```

---

## 团队配置建议

### 当前配置 (12人)

| 角色 | 人数 | 负责范围 |
|------|------|----------|
| 开发-Rust核心 | 3 | LLM调用、流式响应、向量存储 |
| 开发-Python SDK | 3 | Fallback实现、持久化、绑定 |
| 开发-CLI/TUI | 3 | 界面框架、高亮、交互 |
| 审查员 | 3 | 各方向一人 |

### 是否需要更多成员？

**分析：**

| 因素 | 当前状态 | 需要调整 |
|------|----------|----------|
| 审查员工作量 | 每人审查4-5个PR | ⚠️ 可能过载 |
| 代码质量风险 | 高（需确保非DEMO） | 🔴 需加强 |
| 并行开发 | 3方向并行 | ✅ 合理 |
| 审查深度 | 需验证非DEMO/MVP | 🔴 需深度审查 |

**建议：增加到16人**

| 新增角色 | 人数 | 职责 |
|----------|------|------|
| 审查员-Rust深度 | 1 | 专门审查Rust实现完整性 |
| 审查员-Python深度 | 1 | 专门审查Python实现完整性 |
| 审查员-集成测试 | 1 | 专门负责端到端验证 |
| QA工程师 | 1 | 负责自动化测试和质量门禁 |

### 最终团队配置 (16人)

```
Team Lead (1人)
    │
    ├── Rust核心组 (4人)
    │   ├── 开发 x3
    │   └── 审查员 x1 (深度审查)
    │
    ├── Python SDK组 (4人)
    │   ├── 开发 x3
    │   └── 审查员 x1 (深度审查)
    │
    ├── CLI/TUI组 (4人)
    │   ├── 开发 x3
    │   └── 审查员 x1
    │
    └── 质量保障组 (3人)
        ├── 集成审查员 x1
        ├── QA工程师 x1
        └── DevOps x1
```

---

**文档版本**: v1.0  
**更新时间**: 2026-05-25
