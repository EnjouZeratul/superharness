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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_subscribe_and_publish() {
        let bus = EventBus::new();
        let received = Arc::new(Mutex::new(String::new()));
        let received_clone = Arc::clone(&received);

        bus.subscribe("test_event", move |event| {
            *received_clone.lock().unwrap() = event.payload.clone();
        });

        let event = Event {
            id: "1".to_string(),
            event_type: "test_event".to_string(),
            payload: "hello world".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        bus.publish(&event);

        assert_eq!(*received.lock().unwrap(), "hello world");
    }

    #[test]
    fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let counter = Arc::new(Mutex::new(0));
        let counter1 = Arc::clone(&counter);
        let counter2 = Arc::clone(&counter);

        bus.subscribe("increment", move |_| {
            *counter1.lock().unwrap() += 1;
        });

        bus.subscribe("increment", move |_| {
            *counter2.lock().unwrap() += 10;
        });

        let event = Event {
            id: "1".to_string(),
            event_type: "increment".to_string(),
            payload: String::new(),
            timestamp: String::new(),
        };

        bus.publish(&event);

        assert_eq!(*counter.lock().unwrap(), 11);
    }

    #[test]
    fn test_no_subscribers() {
        let bus = EventBus::new();

        let event = Event {
            id: "1".to_string(),
            event_type: "unknown_event".to_string(),
            payload: String::new(),
            timestamp: String::new(),
        };

        // 应该不崩溃
        bus.publish(&event);
    }

    #[test]
    fn test_different_event_types() {
        let bus = EventBus::new();
        let results = Arc::new(Mutex::new(Vec::new()));
        let r1 = Arc::clone(&results);
        let r2 = Arc::clone(&results);

        bus.subscribe("event_a", move |_| {
            r1.lock().unwrap().push("A");
        });

        bus.subscribe("event_b", move |_| {
            r2.lock().unwrap().push("B");
        });

        let event_a = Event {
            id: "1".to_string(),
            event_type: "event_a".to_string(),
            payload: String::new(),
            timestamp: String::new(),
        };

        let event_b = Event {
            id: "2".to_string(),
            event_type: "event_b".to_string(),
            payload: String::new(),
            timestamp: String::new(),
        };

        bus.publish(&event_a);
        bus.publish(&event_b);

        let res = results.lock().unwrap();
        assert_eq!(*res, vec!["A", "B"]);
    }

    #[test]
    fn test_event_serialization() {
        let event = Event {
            id: "123".to_string(),
            event_type: "test".to_string(),
            payload: "data".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("123"));
        assert!(json.contains("test"));
        assert!(json.contains("data"));
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{
            "id": "456",
            "event_type": "my_event",
            "payload": "my_payload",
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "456");
        assert_eq!(event.event_type, "my_event");
        assert_eq!(event.payload, "my_payload");
    }

    #[test]
    fn test_default_event_bus() {
        let bus = EventBus::default();
        let event = Event {
            id: "1".to_string(),
            event_type: "test".to_string(),
            payload: String::new(),
            timestamp: String::new(),
        };

        bus.publish(&event); // 应该不崩溃
    }

    #[test]
    fn test_concurrent_publish() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;

        let bus = Arc::new(EventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));

        let c1 = Arc::clone(&counter);
        bus.subscribe("count", move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let mut handles = vec![];
        for _ in 0..10 {
            let b = Arc::clone(&bus);
            handles.push(thread::spawn(move || {
                let event = Event {
                    id: "1".to_string(),
                    event_type: "count".to_string(),
                    payload: String::new(),
                    timestamp: String::new(),
                };
                b.publish(&event);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_event_with_empty_payload() {
        let bus = EventBus::new();
        let received = Arc::new(Mutex::new(false));
        let r = Arc::clone(&received);

        bus.subscribe("empty", move |_| {
            *r.lock().unwrap() = true;
        });

        let event = Event {
            id: "1".to_string(),
            event_type: "empty".to_string(),
            payload: String::new(),
            timestamp: String::new(),
        };

        bus.publish(&event);

        assert!(*received.lock().unwrap());
    }
}
