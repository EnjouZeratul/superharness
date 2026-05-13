//! 并发性能基准测试
//!
//! 测试并发场景下的性能。

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::Arc;
use std::thread;
use parking_lot::Mutex;
use dashmap::DashMap;
use std::collections::HashMap;

// 使用 Mutex 的 HashMap
fn bench_mutex_hashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutex_hashmap");

    for threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::new("threads", threads), threads, |b, _| {
            b.iter(|| {
                let map = Arc::new(Mutex::new(HashMap::new()));
                let mut handles = vec![];

                for i in 0..*threads {
                    let map_clone = Arc::clone(&map);
                    handles.push(thread::spawn(move || {
                        for j in 0..100 {
                            let mut m = map_clone.lock();
                            m.insert(format!("key_{}_{}", i, j), format!("value_{}_{}", i, j));
                        }
                    }));
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                black_box(map)
            })
        });
    }
    group.finish();
}

// 使用 DashMap
fn bench_dashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("dashmap");

    for threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::new("threads", threads), threads, |b, _| {
            b.iter(|| {
                let map = Arc::new(DashMap::new());
                let mut handles = vec![];

                for i in 0..*threads {
                    let map_clone = Arc::clone(&map);
                    handles.push(thread::spawn(move || {
                        for j in 0..100 {
                            map_clone.insert(format!("key_{}_{}", i, j), format!("value_{}_{}", i, j));
                        }
                    }));
                }

                for handle in handles {
                    handle.join().unwrap();
                }

                black_box(map)
            })
        });
    }
    group.finish();
}

// 读操作基准测试
fn bench_read_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_ops");

    // 预填充数据
    let mutex_map = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut m = mutex_map.lock();
        for i in 0..1000 {
            m.insert(format!("key_{}", i), format!("value_{}", i));
        }
    }

    let dash_map: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
    for i in 0..1000 {
        dash_map.insert(format!("key_{}", i), format!("value_{}", i));
    }

    group.bench_function("mutex_read", |b| {
        let mut idx = 0u32;
        b.iter(|| {
            idx = (idx + 1) % 1000;
            let m = mutex_map.lock();
            let key = format!("key_{}", idx);
            let result = m.get(&key).cloned();
            black_box(result)
        })
    });

    group.bench_function("dashmap_read", |b| {
        let mut idx = 0u32;
        b.iter(|| {
            idx = (idx + 1) % 1000;
            let key = format!("key_{}", idx);
            let result = dash_map.get(&key).map(|v| v.clone());
            black_box(result)
        })
    });

    group.finish();
}

// 配置管理器并发访问模拟
fn bench_config_concurrent_access(c: &mut Criterion) {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let counter = Arc::new(AtomicUsize::new(0));

    c.bench_function("atomic_counter", |b| {
        b.iter(|| {
            counter.fetch_add(1, Ordering::SeqCst);
            black_box(counter.load(Ordering::SeqCst))
        })
    });
}

criterion_group!(
    benches,
    bench_mutex_hashmap,
    bench_dashmap,
    bench_read_operations,
    bench_config_concurrent_access,
);

criterion_main!(benches);
