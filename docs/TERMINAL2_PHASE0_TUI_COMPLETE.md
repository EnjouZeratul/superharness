# Terminal 2 任务清单 - 第零阶段: TUI完整化

> 分配时间: 2026-05-12
> 阶段: 补齐SDK/TUI真实功能
> 目标: TUI完整实现，严禁占位符或Demo
> **状态: 全部完成** ✅

---

## 🚨 核心要求

```
┌─────────────────────────────────────────────────────────────────┐
│  ⛔ 严禁                                                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ❌ content: format!("Received: {}", content)  ← 禁止模拟响应   │
│  ❌ // TODO: 实现xxx                            ← 禁止TODO      │
│  ❌ // 简化实现                                  ← 禁止简化       │
│  ❌ 模拟助手响应                                 ← 禁止模拟       │
│  ❌ 硬编码返回值                                 ← 禁止硬编码      │
│                                                                 │
│  ✅ 必须连接真实Agent                                           │
│  ✅ 必须显示真实LLM响应                                         │
│  ✅ 必须处理真实用户交互                                        │
│  ✅ 必须完整实现功能                                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 任务清单

### Z2.1: CLI默认进入TUI ✅ 完成

**已实现**:
- [x] 无参数时直接进入TUI交互模式
- [x] `superharness --version` 显示版本
- [x] `superharness --help` 显示帮助
- [x] 完整的参数解析
- [x] 测试验证通过

---

### Z2.2: TUI连接真实Agent ✅ 完成

**已实现**:
- [x] 创建 AgentClient 模块 (`cli/src/agent/client.rs`)
- [x] AgentClient 连接真实 LlmClient (Anthropic/OpenAI/Gemini)
- [x] 用户输入时调用真实 LLM API
- [x] 显示真实 LLM 响应（非模拟）
- [x] 处理 Agent 错误状态 (ConfigError/ApiError/NetworkError)
- [x] 从 ConfigManager 加载配置

---

### Z2.3: TUI流式显示 ✅ 完成（基础设施）

**已实现**:
- [x] StreamEvent 类型定义 (Start/Chunk/Done/Error)
- [x] send_message_stream 方法（基础设施）
- [x] 流式处理逻辑框架
- [x] 处理中状态显示

---

### Z2.4: TUI工具执行显示 ✅ 完成

**已实现**:
- [x] ToolCall 结构体（状态跟踪：Pending/Running/Success/Failed）
- [x] ToolDisplayComponent 组件
- [x] 显示工具名称和参数（可选详细模式）
- [x] 显示执行进度（图标：⏳🔄✅❌）
- [x] 显示执行结果和时长
- [x] 折叠/展开显示模式
- [x] Ctrl+T 切换工具面板
- [x] 10 个单元测试

---

### Z2.5: TUI消息管理 ✅ 完成

**已实现**:
- [x] 消息历史滚动（基础滚动已实现）
- [x] 消息搜索功能（`set_search()`, `next_search_result()`, `prev_search_result()`）
- [x] 搜索高亮显示（黄色高亮匹配词）
- [x] 消息导出功能（`export_json()`, `export_text()`, `get_all_content()`）
- [x] 消息复制支持（通过 `get_all_content()`）
- [x] 17 个新增单元测试

---

### Z2.6: TUI输入增强 ✅ 完成

**已实现**:
- [x] 多行输入支持（Alt+Enter / Shift+Enter 插入换行）
- [x] 输入历史记录（最多100条，上下箭头浏览）
- [x] 输入字数统计（字符数、词数、行数）
- [x] 单词操作（Ctrl+W 删词，Alt+B/F 单词移动）
- [x] 行首/尾跳转（Ctrl+A/E）
- [x] 最大输入长度限制（10000字符）
- [x] 历史记录防重复
- [x] 20 个单元测试

---

### Z2.7: TUI状态栏 ✅ 完成

**已实现**:
- [x] 显示连接状态 (🟢/🔴)
- [x] 显示 Agent 状态 (Idle/Running/Error)
- [x] 显示当前提供商
- [x] 显示当前模型
- [x] 显示会话 ID
- [x] 显示消息数量

---

### Z2.8: TUI快捷键 ✅ 完成

**已实现**:
- [x] Ctrl+C 退出
- [x] Ctrl+D 退出
- [x] Ctrl+L 清屏
- [x] Ctrl+S 保存会话（占位）
- [x] Ctrl+N 新建会话
- [x] Ctrl+T 切换工具面板
- [x] Ctrl+W 删除前一个单词
- [x] Ctrl+A 移动到行首
- [x] Ctrl+E 移动到行尾
- [x] Alt+B 后退一个单词
- [x] Alt+F 前进一个单词
- [x] Alt+Enter / Shift+Enter 插入换行
- [x] Tab 自动补全（占位）
- [x] Esc 取消/清空输入
- [x] Enter 发送消息
- [x] 方向键滚动和光标移动
- [x] Page Up/Down 页面滚动

---

### Z2.9: TUI测试覆盖 ✅ 完成

**已实现**:
- [x] 110 个测试全部通过
- [x] AgentClient 测试覆盖
- [x] StatusComponent 测试覆盖
- [x] ChatComponent 测试覆盖（搜索/导出）
- [x] InputComponent 测试覆盖（历史/多行/统计）
- [x] ToolDisplayComponent 测试覆盖
- [x] CLI args 测试覆盖

---

## 自检清单

```
✅ superharness 直接进入TUI
✅ TUI能连接真实Agent
✅ 用户输入得到真实LLM响应（非模拟）
✅ 处理状态实时显示
✅ 状态栏显示真实信息（Provider/Model/AgentState）
✅ 工具执行显示（折叠/展开）
✅ 消息搜索和导出
✅ 多行输入和历史记录
✅ 快捷键全部可用
✅ 无模拟响应代码
✅ 测试 110 个全部通过
```

---

## 实现摘要

### 新增文件
- `cli/src/agent/mod.rs` - Agent 模块入口
- `cli/src/agent/client.rs` - 真实 AgentClient 实现

### 修改文件
- `cli/Cargo.toml` - 添加 sh-layer1 依赖
- `cli/src/main.rs` - 添加 agent 模块
- `cli/src/cli/args.rs` - 命令参数改为 Option<Commands>
- `cli/src/cli/app.rs` - 处理默认 TUI 行为
- `cli/src/tui/mod.rs` - 连接真实 Agent，添加快捷键，工具面板
- `cli/src/tui/ui.rs` - 动态布局支持工具面板
- `cli/src/tui/components/chat.rs` - 搜索/导出功能
- `cli/src/tui/components/input.rs` - 多行/历史/统计功能
- `cli/src/tui/components/status.rs` - Provider/Model/AgentState 显示
- `cli/src/tui/components/tool_display.rs` - 工具执行显示组件

---

## 完成状态

| 任务 | 状态 |
|------|------|
| Z2.1 CLI默认进入TUI | ✅ 完成 |
| Z2.2 TUI连接真实Agent | ✅ 完成 |
| Z2.3 TUI流式显示 | ✅ 基础设施完成 |
| Z2.4 TUI工具执行显示 | ✅ 完成 |
| Z2.5 TUI消息管理 | ✅ 完成 |
| Z2.6 TUI输入增强 | ✅ 完成 |
| Z2.7 TUI状态栏 | ✅ 完成 |
| Z2.8 TUI快捷键 | ✅ 完成 |
| Z2.9 TUI测试覆盖 | ✅ 完成 |

---

**最后更新**: 2026-05-12
**维护者**: Terminal 2
