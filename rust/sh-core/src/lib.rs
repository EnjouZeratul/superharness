//! # SuperHarness Core
//!
//! 统一重导出所有 layer 模块。

// 重导出各层
pub use sh_layer0 as layer0;
pub use sh_layer1 as layer1;
pub use sh_layer2 as layer2;
pub use sh_layer3 as layer3;
pub use sh_layer4 as layer4;

// 常用类型快捷导出
pub use sh_layer0::SecurityGateway;
pub use sh_layer1::{CostTracker, LlmClient, StorageEngine};
pub use sh_layer2::{AgentRuntime, CheckpointWriter, ConcurrentSessionManager as SessionManager};
pub use sh_layer3::{
    DefaultToolExecutor as ToolExecutor, QueryEngine, UnifiedMemorySystem as MemorySystem,
};
pub use sh_layer4::{AuditLogger, McpBridge, PluginLoader};
