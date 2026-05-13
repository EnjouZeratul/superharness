//! Session 序列化性能基准测试
//!
//! 测试 Session 的创建、消息添加、序列化性能。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;

/// 简化的 Session 结构用于基准测试
#[derive(Debug, Clone)]
struct Session {
    id: String,
    messages: Vec<Message>,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct Message {
    role: String,
    content: String,
}

impl Session {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            messages: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
    }

    fn to_json(&self) -> String {
        // 简化的 JSON 序列化
        let messages: Vec<String> = self
            .messages
            .iter()
            .map(|m| format!(r#"{{"role":"{}","content":"{}"}}"#, m.role, m.content))
            .collect();
        format!(
            r#"{{"id":"{}","messages":[{}]}}"#,
            self.id,
            messages.join(",")
        )
    }
}

fn bench_session_creation(c: &mut Criterion) {
    c.bench_function("session_new", |b| {
        b.iter(|| black_box(Session::new("test-session")))
    });
}

fn bench_session_add_message(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_message");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut session = Session::new("test");
                for i in 0..*size {
                    session.add_message("user", &format!("Message {}", i));
                }
                black_box(session)
            })
        });
    }
    group.finish();
}

fn bench_session_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize");

    for size in [10, 100, 500].iter() {
        let mut session = Session::new("test");
        for i in 0..*size {
            session.add_message(
                "user",
                &format!("Message content {} with some additional text", i),
            );
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| black_box(session.to_json()))
        });
    }
    group.finish();
}

fn bench_session_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone");

    for size in [10, 100, 500].iter() {
        let mut session = Session::new("test");
        for i in 0..*size {
            session.add_message("user", &format!("Message {}", i));
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| black_box(session.clone()))
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_session_creation,
    bench_session_add_message,
    bench_session_serialize,
    bench_session_clone,
);

criterion_main!(benches);
