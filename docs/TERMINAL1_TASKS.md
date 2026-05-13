# Terminal 1 任务清单

> 分配时间: 2026-05-10
> 负责层级: Layer 0 + Layer 1
> 角色: 安全基础层开发者

---

## 当前状态

**已完成模块**: 11/11 ✅

### Layer 0 (4/4) ✅
- [x] input_validator
- [x] pii_scrubber
- [x] access_controller
- [x] rate_limiter

### Layer 1 (7/10) ✅
- [x] llm_client
- [x] embeddings
- [x] storage_engine
- [x] streaming
- [x] config_manager
- [x] event_bus
- [x] observability
- [x] cost_tracker
- [x] cache_manager
- [x] error_handler

---

## 当前任务

### 任务 1.1: 完善测试覆盖

**优先级**: P0
**预计时间**: 2-3小时

```
目标: 为已完成模块补充边界测试

需要添加测试:
├── input_validator
│   ├── test_max_length_boundary
│   ├── test_unicode_handling
│   └── test_concurrent_validation
│
├── pii_scrubber
│   ├── test_chinese_pii (中文姓名、地址)
│   ├── test_multiple_pii_types
│   └── test_performance_large_text
│
├── rate_limiter
│   ├── test_concurrent_requests
│   ├── test_burst_handling
│   └── test_token_refill_accuracy
│
└── cost_tracker
    ├── test_multiple_models
    ├── test_budget_reset
    └── test_concurrent_recording
```

**输出文件**:
```
rust/layer0/src/input_validator.rs  # 添加 #[cfg(test)]
rust/layer0/src/pii_scrubber.rs
rust/layer0/src/rate_limiter.rs
rust/layer1/src/cost_tracker.rs
```

---

### 任务 1.2: 新增安全模块

**优先级**: P1
**预计时间**: 3-4小时

```
目标: 实现 secrets-manager 模块

文件: rust/layer0/src/secrets_manager.rs

功能:
├── 从环境变量安全读取密钥
├── 支持密钥轮换
├── 内存中加密存储
└── 审计日志记录

接口设计:
pub struct SecretsManager {
    // 内部实现
}

impl SecretsManager {
    pub fn new() -> Self;
    pub fn get(&self, key: &str) -> Result<Option<String>>;
    pub fn set(&self, key: &str, value: &str) -> Result<()>;
    pub fn rotate(&self, key: &str, new_value: &str) -> Result<()>;
}
```

---

### 任务 1.3: 性能优化

**优先级**: P2
**预计时间**: 2小时

```
目标: 优化热点路径性能

优化点:
├── pii_scrubber: 使用更高效的正则引擎
├── rate_limiter: 减少锁竞争
└── cache_manager: 调整 LRU 参数

验证:
├── cargo bench
└── 确保无性能回退
```

---

## 工作目录

```
rust/layer0/src/
rust/layer1/src/
tests/integration/test_layer01.rs
```

## 自检清单

提交前必须确认:
```
□ cargo clippy --all-targets 无警告
□ cargo fmt --check 通过
□ cargo test --all 通过
□ 文档注释完整
□ 无 TODO!/unimplemented!
```

## 完成标准

- 所有测试通过
- 代码覆盖率 > 80%
- clippy 无警告
- 文档注释完整

---

## 注意事项

1. **不修改其他层级的文件**
2. **遇到阻塞立即通知 Terminal 0**
3. **每完成一个任务提交一次**
4. **保持与主分支同步**
