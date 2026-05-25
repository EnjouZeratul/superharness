# Terminal 4 任务清单 - 测试开发与覆盖

> 分配时间: 2026-05-24
> 负责范围: 测试覆盖、回归测试、性能测试、测试工具开发
> 目标: 确保产品质量，达到发布标准
> 前置依赖: T1-T3全部验收通过
> 等待机制: 收到T0通知或检测T1/T2/T3验收清单全部[x]后方可开始

---

## 零、依赖等待规则

```
┌─────────────────────────────────────────────────────────────────────┐
│  ⛔ T1-T3未完成时禁止行为                                            │
├─────────────────────────────────────────────────────────────────────┤
│  ❌ 编写测试代码（无被测对象）                                       │
│  ❌ 假设功能实现编写测试                                             │
│  ❌ 使用mock假装功能存在                                            │
│  ❌ "先写测试框架，等实现后再验证"                                   │
│  ❌ "先写文档框架，等功能完成再填充" - 已删除                        │
│                                                                     │
│  ✅ 正确等待方式                                                    │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 收到T0通知"T1-T3全部验收通过"                                   │
│  ✅ 检测T1/T2/T3验收清单全部[x]                                     │
│  ✅ 实际运行功能成功后才编写对应测试                                 │
│  ✅ 等待期间可做：测试计划、测试用例设计                              │
└─────────────────────────────────────────────────────────────────────┘
```

**可提前准备的任务（无功能依赖）：**
- 测试计划文档 - 可提前编写
- 任务4.7: CI/CD配置 - 可提前配置框架
- 任务4.8: CHANGELOG框架 - 可提前准备结构

**必须等待的任务（依赖真实���能）：**
- 任务4.1: 快速入门 - 等T1-T2完成
- 任务4.2: API文档 - 等T1完成
- 任务4.4: 示例代码 - 等T1-T2完成
- 任务4.5/4.6: 发布准备 - 等全部完成
- 任务4.9: 发布验证 - 等全部完成

---

## 一、当前状态

**文档状态**:
- README.md: 存在但需更新
- docs/user/: 存在框架
- API文档: 缺失完整docstring
- 发布配置: 部分

---

## 二、任务清单

### 任务 4.1: 用户快速入门指南 ✅

**文件**: `docs/user/quick_start.md`

**内容要求**:
- [x] 环境准备（Python版本、依赖）
- [x] 安装步骤（pip install）
- [x] 第一个Agent示例
- [x] 常见问题解答
- [x] 下一步学习路径

**完成证据**:
- 文件已创建: `docs/user/quick_start.md`
- 包含完整安装说明、Agent/intelligentAgent示例、FAQ

**示例结构**:
```markdown
# 快速入门

## 安装
pip install continuum-sdk

## 30秒上手
from continuum import Agent
agent = Agent()
result = agent.run("hello")

## 下一步
- 工具使用
- 任务规划
- 会话管理
```

---

### 任务 4.2: API完整文档 ✅

**文件**: 各模块的docstring

**需要完善**:
- [x] `continuum_sdk/agent/runtime.py` - Agent类完整API
- [x] `continuum_sdk/agent/intelligent.py` - IntelligentAgent API
- [x] `continuum_sdk/llm/client.py` - LLM客户端API
- [x] `continuum_sdk/tools/builtin.py` - 内置工具API
- [x] `continuum_sdk/workflow/dag.py` - 工作流API
- [x] `continuum_sdk/agent/planner.py` - 任务规划API
- [x] `continuum_sdk/agent/self_correction.py` - 自我纠错API
- [x] `continuum_sdk/agent/progress.py` - 进度追踪API
- [x] `continuum_sdk/agent/session.py` - 会话管理API
- [x] `continuum_sdk/agent/checkpoint.py` - 检查点API

**完成证据**:
- 各模块docstring已增强，包含Quick Start、Examples、See Also等

**docstring格式**:
```python
def run(self, task: str, **kwargs) -> str:
    """执行任务并返回结果。
    
    这是Agent的主要入口方法。它会：
    1. 解析任务意图
    2. 规划执行步骤
    3. 调用LLM生成响应
    4. 返回结果
    
    Args:
        task: 任务描述，自然语言形式
        **kwargs: 额外参数
            - temperature: 温度参数，默认0.7
            - max_tokens: 最大token数，默认4096
    
    Returns:
        Agent的响应文本
    
    Raises:
        AuthenticationError: API密钥无效
        NetworkError: 网络连接失败
    
    Example:
        >>> agent = Agent()
        >>> result = agent.run("hello")
        'Hello! How can I help you today?'
    """
```

---

### 任务 4.3: 架构说明文档 ⏳

**文件**: `docs/ARCHITECTURE_EXPLAINED.md`

**内容要求**:
- [ ] 六层架构详解
- [ ] 各层职责和边界
- [ ] 依赖关系说明
- [ ] 扩展点说明
- [ ] 与竞品架构对比

**状态**: 可提前准备，待完成

---

### 任务 4.4: 示例代码完善 ✅

**目录**: `examples/`

**需要示例**:
- [x] `basic/` - 基础使用
  - hello_agent.py
  - streaming.py
  - tools.py
- [x] `advanced/` - 高级用法
  - intelligent_agent.py
  - workflow_dag.py
  - checkpoint_recovery.py
- [x] `integrations/` - 集成示例
  - mcp_server.py
  - custom_llm.py
- [x] `examples/README.md` - 示例目录说明

**完成证据**:
- 所有示例文件已创建，包含完整注释和预期输出说明

---

### 任务 4.5: PyPI发布准备 ⏳

**文件**: `python/pyproject.toml`

**状态**: 等待T3完成（需要全部验收通过）

**验证**:
```bash
# 本地构建测试
cd python
python -m build
pip install dist/continuum_sdk-1.0.0-py3-none-any.whl

# 导入测试
python -c "from continuum import Agent; print('OK')"

# 上传测试（test pypi）
twine upload --repository testpypi dist/*
```

---

### 任务 4.6: crates.io发布准备 ⏳

**文件**: `rust/*/Cargo.toml`

**状态**: 等待T3完成（需要全部验收通过）

**验证**:
```bash
# 发布顺序
cargo publish --package sh-layer0
cargo publish --package sh-layer1
cargo publish --package sh-layer2
cargo publish --package sh-layer3
cargo publish --package sh-layer4
cargo publish --package sh-core
cargo publish --package sh-python
cargo publish --package continuum-cli
```

---

### 任务 4.7: CI/CD配置

**文件**: `.github/workflows/`

**工作流配置**:
- [ ] `test.yml` - 测试自动化
  - Python 3.10, 3.11, 3.12
  - Rust stable
  - 覆盖率报告
- [ ] `publish.yml` - 发布自动化
  - PyPI自动发布
  - crates.io自动发布
- [ ] `lint.yml` - 代码检查
  - ruff (Python)
  - clippy (Rust)

**test.yml示例**:
```yaml
name: Test
on: [push, pull_request]

jobs:
  python-test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: ${{ matrix.python-version }}
      - run: pip install -e ".[dev]"
      - run: pytest --cov=continuum_sdk
  
  rust-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo test --workspace
```

---

### 任务 4.8: CHANGELOG编写

**文件**: `CHANGELOG.md`

**内容**:
```markdown
# Changelog

## [1.0.0] - 2026-05-24

### Added
- 完整内置工具集
- 任务自检机制
- 检查点系统（Python集成）
- MCP协议完整支持
- Git深度集成

### Features
- 连续特性：任意中断点恢复
- 智能Agent：任务分解与自我纠错
- 分层记忆：Working/Session/Project/LongTerm
- 工作流DAG：可视化执行计划

### Compared to Claude Code
- ✅ 任务自检（超越）
- ✅ 检查点恢复（独特）
- ✅ 开源可定制（独特）
- ✅ 多提供商支持

### Compared to LangChain
- ✅ Rust性能核心（独特）
- ✅ 内置TUI（独特）
- ✅ 检查点恢复（独特）
- = Agent抽象（持平）
- = 工具系统（持平）
```

---

### 任务 4.9: 发布验证

**验证清单**:

#### PyPI验证
```bash
# 从PyPI安装
pip install continuum-sdk

# 验证导入
python -c "from continuum import Agent, IntelligentAgent; print('OK')"

# 验证CLI
continuum --version
continuum run "hello"
```

#### crates.io验证
```bash
# 安装
cargo install continuum-cli

# 验证运行
continuum --version
```

#### 功能验证
- [ ] 新环境pip install成功
- [ ] Agent.run()正常工作
- [ ] 内置工具可用
- [ ] CLI正常启动

---

## 三、禁止事项（违反即失败）

```
┌─────────────────────────────────────────────────────────────────────┐
│  ⛔ 绝对禁止                                                        │
├─────────────────────────────────────────────────────────────────────┤
│  ❌ 文档只有框架无内容                                              │
│     # 快速入门\n（空）\n## 安装\n（空）                              │
│                                                                     │
│  ❌ 示例代码不能运行                                                │
│     提供的示例有语法错误或依赖缺失                                   │
│                                                                     │
│  ❌ docstring不完整                                                 │
│     def func(x):\n    pass  # 无文档                               │
│                                                                     │
│  ❌ 假装发布                                                        │
│     "发布准备完成" - 但未真实上传测试                               │
│                                                                     │
│  ❌ CI配置不验证                                                    │
│     写了workflow但未确认能运行                                      │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 必须提供                                                       │
├─────────────────────────────────────────────────────────────────────┤
│  ✅ 可运行的示例代码                                               │
│  ✅ 真实安装测试结果                                               │
│  ✅ CI运行通过证据                                                 │
│  ✅ 完整API文档                                                    │
└─────────────────────────────────────────────────────────────────────┘
```

## 四、输出规范（必须提供）

每个任务完成后必须提供：

### 4.1 文档证据
```bash
# 必须提供文档构建结果：
$ mkdocs build
INFO    -  Cleaning site directory
INFO    -  Building documentation to directory: site
INFO    -  Documentation built in 2.34s

# 必须提供示例运行结果：
$ python examples/basic/hello_agent.py
Agent initialized...
Response: Hello! How can I help you?
```

### 4.2 发布验证证据
```bash
# 必须提供发布测试结果：
$ pip install dist/continuum_sdk-1.0.0-py3-none-any.whl
Successfully installed continuum-sdk-1.0.0

$ python -c "from continuum import Agent; print('OK')"
OK

$ continuum --version
continuum version 1.0.0
```

### 4.3 CI验证证据
```
必须提供GitHub Actions运行截图或日志：
- Workflow: Test / python-test (3.11) — Passed
- Workflow: Test / rust-test — Passed
- Workflow: Publish / pypi — Passed
```

### 4.4 禁止的输出格式
```
❌ "文档已完成"
❌ "发布准备OK"
❌ "CI配置完成"

以上声明若无具体证据，视为假装完成，任务失败。
```

---

## 五、文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `docs/user/quick_start.md` | 创建 | 快速入门 |
| 各模块py文件 | 修改 | 完善docstring |
| `docs/ARCHITECTURE_EXPLAINED.md` | 创建 | 架构说明 |
| `examples/` | 完善 | 示例代码 |
| `python/pyproject.toml` | 验证 | 发布配置 |
| `rust/*/Cargo.toml` | 验证 | 发布配置 |
| `.github/workflows/*.yml` | 创建 | CI/CD |
| `CHANGELOG.md` | 更新 | 变更日志 |

---

## 六、验收清单

```
□ 快速入门文档完整
□ API文档完整（所有公共API）
□ 架构说明清晰
□ 示例代码可运行
□ PyPI发布测试通过
□ crates.io发布测试通过
□ CI/CD工作流配置完整
□ CHANGELOG更新
□ 新环境安装验证通过
□ 功能验证通过
```

---

## 七、汇报流程

```
完成任务后：
1. 更新本文档，将 [ ] 改为 [x]
2. 在任务下方添加完成证据（文档链接、发布结果）
3. 通知T0审查

禁止：
❌ 通过PR提交代码
❌ 只写"完成"不提供证据
❌ 修改T0文档
```

---

**维护者**: T0-new
**最后更新**: 2026-05-24
