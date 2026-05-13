//! # ReadWriteLock
//!
//! 读写分离锁实现，支持并发读取和互斥写入。
//!
//! 特性：
//! - 读操作可并发执行（共享锁）
//! - 写操作需互斥执行（排他锁）
//! - 写优先：当有写者等待时，新的读者会被阻塞

use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::time::Duration;

/// 读写锁状态
#[derive(Debug, Default)]
struct LockState {
    readers: i32,
    writers: i32,
    waiting_writers: i32,
    write_preferred: bool,
}

/// 读写分离锁
///
/// 使用 parking_lot 实现的读写分离锁，比标准库 RwLock 提供更细粒度的控制。
pub struct ReadWriteLock {
    state: Mutex<LockState>,
    read_cond: Condvar,
    write_cond: Condvar,
}

impl ReadWriteLock {
    /// 创建新的读写锁
    pub fn new() -> Self {
        Self {
            state: Mutex::new(LockState::default()),
            read_cond: Condvar::new(),
            write_cond: Condvar::new(),
        }
    }

    /// 获取读锁
    ///
    /// 多个读者可以同时持有读锁。
    /// 当有写者活跃或等待时，读者会被阻塞。
    pub fn read<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let mut state = self.state.lock();

        // 等待条件：没有活跃写者，且没有写者在等待（写优先）
        while state.writers > 0 || (state.write_preferred && state.waiting_writers > 0) {
            self.read_cond.wait(&mut state);
        }

        state.readers += 1;
        // 释放锁后执行读操作
        drop(state);

        let result = f();

        let mut state = self.state.lock();
        state.readers -= 1;

        // 如果没有读者了，通知等待的写者
        if state.readers == 0 {
            self.write_cond.notify_all();
            self.read_cond.notify_all();
        }

        result
    }

    /// 获取写锁
    ///
    /// 写锁是排他的，同一时间只能有一个写者。
    /// 当有读者或写者活跃时，新的写者会被阻塞。
    pub fn write<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let mut state = self.state.lock();
        state.waiting_writers += 1;
        state.write_preferred = true;

        // 等待条件：没有活跃读者和写者
        while state.readers > 0 || state.writers > 0 {
            self.write_cond.wait(&mut state);
        }

        state.waiting_writers -= 1;
        state.writers += 1;

        // 释放锁后执行写操作
        drop(state);

        let result = f();

        let mut state = self.state.lock();
        state.writers -= 1;

        // 如果没有等待的写者了，清除写优先标志
        if state.waiting_writers == 0 {
            state.write_preferred = false;
        }

        // 通知所有等待的线程
        self.write_cond.notify_all();
        self.read_cond.notify_all();

        result
    }

    /// 尝试获取读锁（带超时）
    ///
    /// # Returns
    /// 成功返回 Some(result)，超时返回 None
    pub fn try_read_timeout<F, T>(&self, f: F, timeout: Duration) -> Option<T>
    where
        F: FnOnce() -> T,
    {
        let mut state = self.state.lock();

        let deadline = std::time::Instant::now() + timeout;
        while state.writers > 0 || (state.write_preferred && state.waiting_writers > 0) {
            if self.read_cond.wait_until(&mut state, deadline).timed_out() {
                return None;
            }
        }

        state.readers += 1;
        drop(state);

        let result = f();

        let mut state = self.state.lock();
        state.readers -= 1;

        if state.readers == 0 {
            self.write_cond.notify_all();
            self.read_cond.notify_all();
        }

        Some(result)
    }

    /// 尝试获取写锁（带超时）
    ///
    /// # Returns
    /// 成功返回 Some(result)，超时返回 None
    pub fn try_write_timeout<F, T>(&self, f: F, timeout: Duration) -> Option<T>
    where
        F: FnOnce() -> T,
    {
        let mut state = self.state.lock();
        state.waiting_writers += 1;
        state.write_preferred = true;

        let deadline = std::time::Instant::now() + timeout;
        while state.readers > 0 || state.writers > 0 {
            if self.write_cond.wait_until(&mut state, deadline).timed_out() {
                state.waiting_writers -= 1;
                if state.waiting_writers == 0 {
                    state.write_preferred = false;
                }
                return None;
            }
        }

        state.waiting_writers -= 1;
        state.writers += 1;
        drop(state);

        let result = f();

        let mut state = self.state.lock();
        state.writers -= 1;

        if state.waiting_writers == 0 {
            state.write_preferred = false;
        }

        self.write_cond.notify_all();
        self.read_cond.notify_all();

        Some(result)
    }

    /// 获取锁状态（用于调试）
    pub fn state(&self) -> LockStateInfo {
        let state = self.state.lock();
        LockStateInfo {
            readers: state.readers,
            writers: state.writers,
            waiting_writers: state.waiting_writers,
            write_preferred: state.write_preferred,
        }
    }
}

impl Default for ReadWriteLock {
    fn default() -> Self {
        Self::new()
    }
}

/// 锁状态信息（用于调试）
#[derive(Debug, Clone)]
pub struct LockStateInfo {
    pub readers: i32,
    pub writers: i32,
    pub waiting_writers: i32,
    pub write_preferred: bool,
}

/// RAII 读锁守卫
pub struct ReadGuard<'a> {
    lock: &'a ReadWriteLock,
}

impl<'a> ReadGuard<'a> {
    pub fn new(lock: &'a ReadWriteLock) -> Self {
        let mut state = lock.state.lock();
        while state.writers > 0 || (state.write_preferred && state.waiting_writers > 0) {
            lock.read_cond.wait(&mut state);
        }
        state.readers += 1;
        Self { lock }
    }
}

impl<'a> Drop for ReadGuard<'a> {
    fn drop(&mut self) {
        let mut state = self.lock.state.lock();
        state.readers -= 1;
        if state.readers == 0 {
            self.lock.write_cond.notify_all();
            self.lock.read_cond.notify_all();
        }
    }
}

/// RAII 写锁守卫
pub struct WriteGuard<'a> {
    lock: &'a ReadWriteLock,
}

impl<'a> WriteGuard<'a> {
    pub fn new(lock: &'a ReadWriteLock) -> Self {
        let mut state = lock.state.lock();
        state.waiting_writers += 1;
        state.write_preferred = true;

        while state.readers > 0 || state.writers > 0 {
            lock.write_cond.wait(&mut state);
        }

        state.waiting_writers -= 1;
        state.writers += 1;
        Self { lock }
    }
}

impl<'a> Drop for WriteGuard<'a> {
    fn drop(&mut self) {
        let mut state = self.lock.state.lock();
        state.writers -= 1;

        if state.waiting_writers == 0 {
            state.write_preferred = false;
        }

        self.lock.write_cond.notify_all();
        self.lock.read_cond.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    #[test]
    fn test_read_write_lock_basic() {
        let lock = ReadWriteLock::new();

        // 测试读操作
        let result = lock.read(|| 42);
        assert_eq!(result, 42);

        // 测试写操作
        let result = lock.write(|| 100);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_concurrent_reads() {
        let lock = Arc::new(ReadWriteLock::new());
        let counter = Arc::new(AtomicU32::new(0));

        let mut handles = vec![];

        for _ in 0..10 {
            let lock = Arc::clone(&lock);
            let counter = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                lock.read(|| {
                    counter.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(Duration::from_millis(10));
                });
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 所有读操作应该并发执行
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_state_info() {
        let lock = ReadWriteLock::new();
        let state = lock.state();
        assert_eq!(state.readers, 0);
        assert_eq!(state.writers, 0);
    }
}
