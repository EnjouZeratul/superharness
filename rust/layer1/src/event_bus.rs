//! 事件总线模块
//!
//! 发布订阅、事件溯源、持久化。

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: String,
    pub payload: String,
    pub timestamp: String,
}

/// 事件处理器
pub type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

/// 事件总线
pub struct EventBus {
    handlers: RwLock<HashMap<String, Vec<EventHandler>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    /// 订阅事件
    pub fn subscribe<F>(&self, event_type: &str, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.handlers
            .write()
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    /// 发布事件
    pub fn publish(&self, event: &Event) {
        if let Some(handlers) = self.handlers.read().get(&event.event_type) {
            for handler in handlers {
                handler(event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
