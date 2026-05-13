//! # Hook System
//!
//! 生命周期钩子系统，用于在关键事件点注入自定义逻辑。

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{AgentState, HookEvent, Layer2Result, SessionId};

/// Hook 回调函数类型
pub type HookCallback = Arc<dyn Fn(&HookContext) -> Layer2Result<()> + Send + Sync>;

/// Hook 上下文
#[derive(Debug, Clone)]
pub struct HookContext {
    pub session_id: SessionId,
    pub event: HookEvent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

impl HookContext {
    pub fn new(session_id: SessionId, event: HookEvent) -> Self {
        Self {
            session_id,
            event,
            timestamp: chrono::Utc::now(),
            data: serde_json::Value::Null,
            metadata: HashMap::new(),
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Hook 系统接口
#[async_trait]
pub trait HookSystemTrait: Send + Sync {
    /// 注册前置钩子
    fn on_before(&self, event: HookEvent, callback: HookCallback);

    /// 注册后置钩子
    fn on_after(&self, event: HookEvent, callback: HookCallback);

    /// 触发钩子
    async fn trigger(&self, context: &HookContext) -> Layer2Result<()>;

    /// 移除钩子
    fn remove(&self, event: HookEvent, is_before: bool);

    /// 清除所有钩子
    fn clear(&self);

    /// 获取钩子数量
    fn count(&self, event: HookEvent) -> (usize, usize);
}

/// Hook 注册表
type HookRegistry = HashMap<HookEvent, Vec<HookCallback>>;

/// Hook 系统实现
pub struct HookSystem {
    before_hooks: RwLock<HookRegistry>,
    after_hooks: RwLock<HookRegistry>,
}

impl HookSystem {
    pub fn new() -> Self {
        Self {
            before_hooks: RwLock::new(HashMap::new()),
            after_hooks: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for HookSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HookSystemTrait for HookSystem {
    fn on_before(&self, event: HookEvent, callback: HookCallback) {
        let mut hooks = self.before_hooks.write();
        hooks.entry(event).or_insert_with(Vec::new).push(callback);
    }

    fn on_after(&self, event: HookEvent, callback: HookCallback) {
        let mut hooks = self.after_hooks.write();
        hooks.entry(event).or_insert_with(Vec::new).push(callback);
    }

    async fn trigger(&self, context: &HookContext) -> Layer2Result<()> {
        // 执行前置钩子
        {
            let hooks = self.before_hooks.read();
            if let Some(callbacks) = hooks.get(&context.event) {
                for callback in callbacks {
                    callback(context)?;
                }
            }
        }

        // 执行后置钩子
        {
            let hooks = self.after_hooks.read();
            if let Some(callbacks) = hooks.get(&context.event) {
                for callback in callbacks {
                    callback(context)?;
                }
            }
        }

        Ok(())
    }

    fn remove(&self, event: HookEvent, is_before: bool) {
        let hooks = if is_before {
            &self.before_hooks
        } else {
            &self.after_hooks
        };

        let mut hooks = hooks.write();
        hooks.remove(&event);
    }

    fn clear(&self) {
        self.before_hooks.write().clear();
        self.after_hooks.write().clear();
    }

    fn count(&self, event: HookEvent) -> (usize, usize) {
        let before = self.before_hooks.read().get(&event).map(|v| v.len()).unwrap_or(0);
        let after = self.after_hooks.read().get(&event).map(|v| v.len()).unwrap_or(0);
        (before, after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_system_creation() {
        let hooks = HookSystem::new();
        let (before, after) = hooks.count(HookEvent::BeforeAgentStart);
        assert_eq!(before, 0);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_hook_registration() {
        let hooks = HookSystem::new();
        let callback: HookCallback = Arc::new(|_| Ok(()));

        hooks.on_before(HookEvent::BeforeAgentStart, callback);

        let (before, _) = hooks.count(HookEvent::BeforeAgentStart);
        assert_eq!(before, 1);
    }
}
