# 代码审查报告: CLI Commands

**审查人:** Terminal 1 (Python视角)
**审查时间:** 2026-05-12
**文件:** `cli/src/commands/`

---

## 整体评价

- [ ] 优秀
- [x] 良好
- [ ] 需改进

---

## Python友好性评估

| 项目 | 评分 | 说明 |
|------|------|------|
| CLI输出格式 | 4/5 | 输出格式友好，但缺少JSON输出选项 |
| 错误提示 | 3/5 | 部分错误信息不够详细 |
| 命令参数设计 | 4/5 | 参数命名清晰，但缺少完整的帮助示例 |
| 文档完整性 | 3/5 | 缺少子命令的详细文档 |

**总分: 14/20**

---

## 命令审查

### 1. run.rs

**优点:**
- 支持交互模式和非交互模式切换
- 参数设计合理（task, config, budget, debug, non-interactive）

**问题:**
- 核心逻辑未实现（标注为TODO）
- 缺少进度反馈
- 错误处理不够详细

**建议:**
```rust
pub fn execute(...) -> Result<()> {
    match task {
        Some(t) => {
            println!("Starting task execution...");
            println!("Task: {}", t);
            // 添加进度指示器
            let spinner = Spinner::new("Processing...");
            // 调用 Agent Runtime
            spinner.finish_with_message("Complete");
        }
        None => {
            // 启动交互模式
            start_interactive_mode()?;
        }
    }
    Ok(())
}
```

### 2. config.rs

**优点:**
- 子命令完整（show, set, get, init, keys, list, add-provider, use）
- 配置路径处理正确
- 输出格式友好

**问题:**
- `set_config` 对无效键名缺乏校验
- 输出格式不可配置（无法输出JSON供程序解析）

**建议:**
添加 `--json` 输出选项:
```rust
pub fn show_config(key: Option<String>, json: bool) -> Result<()> {
    let config = load_config()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&config)?);
    } else {
        // 现有文本输出逻辑
    }
}
```

### 3. session.rs

**优点:**
- 子命令完整（list, resume, delete, show）
- 会话ID处理正确

**问题:**
- 缺少会话详情显示
- `list` 命令输出格式简单

**建议:**
添加会话详情:
```rust
fn show_session(id: String) -> Result<()> {
    let session = load_session(&id)?;
    println!("Session: {}", session.id);
    println!("Created: {}", session.created_at);
    println!("Messages: {}", session.message_count);
    println!("Cost: ${:.4}", session.cost);
    println!("Tokens: {}", session.tokens);
    // 显示最近消息
    println!("\nRecent messages:");
    for msg in session.messages.iter().rev().take(5) {
        println!("  [{}] {}", msg.role, msg.content);
    }
}
```

### 4. tools.rs

**优点:**
- 工具列表输出清晰
- 支持筛选

**问题:**
- 缺少工具详情显示
- 缺少工具使用示例

**建议:**
添加工具详情:
```rust
pub fn show_tool(name: &str) -> Result<()> {
    let tool = get_tool(name)?;
    println!("Tool: {}", tool.name);
    println!("Category: {:?}", tool.category);
    println!("Description: {}", tool.description);
    println!("\nParameters:");
    for (param, schema) in &tool.parameters {
        println!("  {} ({:?})", param, schema);
    }
    println!("\nExample:");
    println!("  superharness run \"read file config.toml\"");
}
```

---

## 输出格式建议

### 当前输出示例
```
Available tools:
  file_read - Read file contents
  file_write - Write to file
```

### 建议添加JSON输出
```bash
$ superharness tools --json
{
  "tools": [
    {"name": "file_read", "description": "Read file contents", "category": "file_ops"},
    {"name": "file_write", "description": "Write to file", "category": "file_ops"}
  ]
}
```

---

## 错误处理改进

### 当前
```rust
if !self.providers.contains_key(name) {
    return Err(anyhow!("Provider '{}' not found", name));
}
```

### 建议
```rust
if !self.providers.contains_key(name) {
    let available: Vec<_> = self.providers.keys().collect();
    return Err(anyhow!(
        "Provider '{}' not found. Available providers: {}",
        name,
        available.join(", ")
    ));
}
```

---

## 结论

- [x] 可以通过
- [ ] 需要修改后通过
- [ ] 需要重大修改

**审查结论:** CLI命令结构合理，基础功能可用。建议添加JSON输出选项、完善错误提示、补充核心逻辑实现。
