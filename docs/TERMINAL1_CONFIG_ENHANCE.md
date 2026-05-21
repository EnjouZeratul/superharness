# Terminal 1 任务清单 - 配置系统增强 (Python层)

> 分配时间: 2026-05-11
> 擅长方向: Python SDK、用户友好接口
> 前置条件: 项目开发完成 ✅

---

## 🎯 擅长匹配

```
Terminal 1 擅长: Python SDK、用户友好接口
本次任务: ✅ 完全匹配（Python配置API + SDK便捷接口）
```

---

## 任务清单

### T1.1: Python 配置 API
- [ ] 创建 `python/continuum_sdk/config/` 模块
- [ ] 实现 `Config` 类
  - [ ] 支持环境变量读取 (`CONTINUUM_*`)
  - [ ] 支持 TOML 配置文件加载
  - [ ] 支持环境变量引用 `${VAR_NAME}`
  - [ ] 支持多提供商管理
- [ ] 实现优先级: 环境变量 > 配置文件 > 默认值
- [ ] 预计时间: 2小时

### T1.2: SDK 配置便捷接口
- [ ] 更新 `continuum_sdk/__init__.py`
  ```python
  # 目标API
  from continuum import Agent, Config

  # 方式1: 环境变量自动读取
  agent = Agent()  # 自动从环境变量配置

  # 方式2: 显式配置
  config = Config.from_env()  # 从环境变量
  config = Config.from_file("~/.continuum/config.toml")
  config = Config(
      provider="anthropic",
      api_key="xxx",
      model="claude-sonnet-4-6"
  )
  agent = Agent(config=config)

  # 方式3: 快速切换
  config.use("openai")  # 切换提供商
  ```
- [ ] 预计时间: 1.5小时

### T1.3: 配置模板文件
- [ ] 创建 `templates/config.toml` 示例
- [ ] 创建 `templates/setup.ps1` (PowerShell)
- [ ] 创建 `templates/setup.sh` (Bash)
- [ ] 预计时间: 0.5小时

---

## 工作目录

```
python/continuum_sdk/
├── config/
│   ├── __init__.py       ← T1.1
│   ├── loader.py         ← T1.1 环境变量+文件加载
│   ├── providers.py      ← T1.1 多提供商管理
│   └── templates.py      ← T1.1 配置模板
└── __init__.py           ← T1.2 更新导出

templates/
├── config.toml           ← T1.3
├── setup.ps1             ← T1.3
└── setup.sh              ← T1.3
```

---

## 环境变量设计

```
CONTINUUM_PROVIDER        = anthropic|openai|gemini|custom
CONTINUUM_API_KEY         = xxx
CONTINUUM_BASE_URL        = xxx
CONTINUUM_MODEL           = xxx
CONTINUUM_SMALL_MODEL     = xxx (用于简单任务)
CONTINUUM_EFFORT_LEVEL    = low|medium|high|max
CONTINUUM_DISABLE_TRAFFIC = 0|1
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. Rust config_manager 增强 → Terminal 2
2. CLI config 命令增强 → Terminal 2
```

---

## 完成标准

- [ ] Python Config API 可用
- [ ] 环境变量自动读取
- [ ] TOML 配置文件支持
- [ ] 配置模板文件生成
- [ ] pytest 测试通过
- [ ] 更新本文档状态为完成