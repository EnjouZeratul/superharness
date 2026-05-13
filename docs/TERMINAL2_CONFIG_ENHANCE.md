# Terminal 2 任务清单 - 配置系统增强 (Rust层)

> 分配时间: 2026-05-11
> 擅长方向: Rust底层、CLI开发
> 前置条件: 项目开发完成 ✅

---

## 🎯 擅长匹配

```
Terminal 2 擅长: Rust底层、CLI开发
本次任务: ✅ 完全匹配（Rust配置管理 + CLI命令）
```

---

## 任务清单

### T2.1: Rust ConfigManager 增强 ✅
- [x] 打开 `rust/layer1/src/config_manager.rs`
- [x] 新增环境变量读取
  - [x] `SUPERHARNESS_PROVIDER`
  - [x] `SUPERHARNESS_API_KEY`
  - [x] `SUPERHARNESS_BASE_URL`
  - [x] `SUPERHARNESS_MODEL`
- [x] 新增多提供商管理
  - [x] `ProviderConfig` 结构体
  - [x] `GlobalSettings` 结构体
- [x] 新增环境变量引用解析 `${VAR_NAME}`
- [x] 实现优先级: 环境变量 > 配置文件 > 默认值
- [x] 预计时间: 2小时
- **完成时间**: 2026-05-11

### T2.2: CLI config 命令增强 ✅
- [x] 打开 `cli/src/commands/config.rs`
- [x] 新增子命令
  ```bash
  superharness config init              # 初始化配置文件
  superharness config add-provider <name> --key <key> --url <url> --model <model>
  superharness config use <provider>    # 切换提供商
  superharness config show              # 显示当前配置
  superharness config list              # 列出所有提供商
  superharness config set <key> <value> # 设置配置项
  superharness config get <key>         # 获取配置项
  ```
- [x] 预计时间: 2小时
- **完成时间**: 2026-05-11

### T2.3: 配置文件默认路径 ✅
- [x] 默认路径: `~/.superharness/config.toml`
- [x] 支持项目级覆盖: `.superharness/config.toml`
- [x] 首次运行自动创建默认配置
- [x] 预计时间: 1小时
- **完成时间**: 2026-05-11

---

## 工作目录

```
rust/layer1/src/
└── config_manager.rs    ← T2.1 增强

cli/src/commands/
└── config.rs            ← T2.2 增强
```

---

## Rust 结构体设计

```rust
/// 提供商配置
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub default_max_tokens: u32,
}

/// 配置管理器
pub struct ConfigManager {
    active_provider: String,
    providers: HashMap<String, ProviderConfig>,
    settings: GlobalSettings,
}

impl ConfigManager {
    /// 从环境变量加载
    pub fn from_env() -> Self;

    /// 从文件加载
    pub fn from_file(path: &Path) -> Self;

    /// 合并配置 (优先级: env > file > default)
    pub fn merge(&mut self, other: ConfigManager);

    /// 切换提供商
    pub fn use_provider(&mut self, name: &str);

    /// 获取当前提供商配置
    pub fn current(&self) -> &ProviderConfig;
}
```

---

## 禁止事项

```
❌ 不要做以下任务（属于其他终端）:

1. Python Config API → Terminal 1
2. SDK 配置便捷接口 → Terminal 1
3. 配置模板文件 → Terminal 1
```

---

## 完成标准

- [x] 环境变量读取实现 ✅
- [x] 多提供商管理实现 ✅
- [x] CLI config 命令增强 ✅
- [x] cargo test 通过 (66 tests) ✅
- [x] 更新本文档状态为完成 ✅

---

## 状态更新

**Terminal 2 完成配置系统增强** ✅

### 完成内容:
- ✅ `ConfigManager` 增强（环境变量、多提供商、环境变量引用解析）
- ✅ `ProviderConfig` 和 `GlobalSettings` 结构体
- ✅ CLI config 命令增强（init/show/set/get/list/add-provider/use）
- ✅ 配置文件默认路径 `~/.superharness/config.toml`

### 新增功能:
- 环境变量支持: `SUPERHARNESS_PROVIDER`, `SUPERHARNESS_API_KEY`, etc.
- 环境变量引用: `${VAR_NAME}` 语法
- 多提供商管理: 添加/切换/列出提供商
- 配置优先级: 环境变量 > 配置文件 > 默认值

### 测试统计:
- Layer 1: 37 tests passed
- CLI: 29 tests passed