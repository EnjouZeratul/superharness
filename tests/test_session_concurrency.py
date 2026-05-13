"""
SessionManager 并发测试用例

测试覆盖：
1. 读写锁基本功能
2. 并发读操作
3. 写操作互斥性
4. 写优先机制
5. 超时处理
6. 会话管理器并发安全
7. 性能压力测试

运行方式：
    pytest test_session_concurrency.py -v
    # 或直接运行
    python test_session_concurrency.py
"""

import threading
import time
import pytest
from typing import List, Dict
import random
import sys
from pathlib import Path

# 添加源代码路径
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from superharness.session_concurrency import (
    ReadWriteLock,
    SimpleStateLock,
    ConcurrentSessionManager,
    ExecutionContext,
    AgentState,
    CheckpointManager
)


# ============================================================================
# 读写锁基本测试
# ============================================================================

class TestReadWriteLock:
    """读写锁测试"""

    def test_read_lock_basic(self):
        """测试读锁基本功能"""
        lock = ReadWriteLock()

        # 应该可以获取读锁
        with lock.read_lock():
            state = lock.get_state()
            assert state["readers"] == 1
            assert state["writers"] == 0

        # 释放后计数归零
        state = lock.get_state()
        assert state["readers"] == 0

    def test_write_lock_basic(self):
        """测试写锁基本功能"""
        lock = ReadWriteLock()

        # 应该可以获取写锁
        with lock.write_lock():
            state = lock.get_state()
            assert state["writers"] == 1
            assert state["readers"] == 0

        # 释放后计数归零
        state = lock.get_state()
        assert state["writers"] == 0

    def test_multiple_readers(self):
        """测试多个读者可以并发"""
        lock = ReadWriteLock()
        readers_count = 5
        results = []
        barrier = threading.Barrier(readers_count)

        def reader():
            with lock.read_lock():
                barrier.wait()  # 等待所有读者进入
                state = lock.get_state()
                results.append(state["readers"])
                time.sleep(0.1)  # 模拟读操作

        threads = [threading.Thread(target=reader) for _ in range(readers_count)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 验证：所有读者都看到了其他读者
        assert max(results) == readers_count
        # 最终读者数应该归零
        assert lock.get_state()["readers"] == 0

    def test_writer_exclusive(self):
        """测试写者互斥"""
        lock = ReadWriteLock()
        write_order = []
        num_writers = 3

        def writer(writer_id: int):
            with lock.write_lock():
                write_order.append(writer_id)
                # 验证只有一个写者
                state = lock.get_state()
                assert state["writers"] == 1
                time.sleep(0.05)

        threads = [threading.Thread(target=writer, args=(i,)) for i in range(num_writers)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 所有写者都执行了
        assert len(write_order) == num_writers
        # 最终写者数应该归零
        assert lock.get_state()["writers"] == 0

    def test_read_write_mutual_exclusion(self):
        """测试读写互斥"""
        lock = ReadWriteLock()
        events = []
        writer_started = threading.Event()
        reader_can_start = threading.Event()

        def writer():
            with lock.write_lock():
                events.append("writer_start")
                writer_started.set()
                time.sleep(0.2)  # 模拟写操作
                events.append("writer_end")

        def reader():
            # 等待写者获取锁后再尝试
            writer_started.wait(timeout=1)
            # 读者应该被阻塞，不能立即获取
            with lock.read_lock():
                events.append("reader")

        # 启动写者
        writer_thread = threading.Thread(target=writer)
        writer_thread.start()
        time.sleep(0.05)  # 确保写者先获取锁

        # 启动读者
        reader_thread = threading.Thread(target=reader)
        reader_thread.start()

        writer_thread.join()
        reader_thread.join()

        # 写者应该先完成，然后读者才能执行
        writer_end_idx = events.index("writer_end")
        reader_idx = events.index("reader")
        assert writer_end_idx < reader_idx

    def test_write_priority(self):
        """测试写优先机制"""
        lock = ReadWriteLock()
        events = []
        reader_can_proceed = threading.Event()

        def reader():
            with lock.read_lock():
                events.append("reader_start")
                reader_can_proceed.wait(timeout=2)  # 等待信号
                events.append("reader_end")

        def writer():
            # 让写者在读者持有锁后尝试获取
            time.sleep(0.05)
            events.append("writer_attempt")
            with lock.write_lock():
                events.append("writer_executed")

        def late_reader():
            # 这个读者在写者等待后尝试获取读锁
            time.sleep(0.1)  # 确保写者先开始等待
            events.append("late_reader_attempt")
            with lock.read_lock():
                events.append("late_reader_executed")

        # 启动第一个读者（持有读锁）
        reader_thread = threading.Thread(target=reader)
        reader_thread.start()

        # 启动写者（应该等待）
        writer_thread = threading.Thread(target=writer)
        writer_thread.start()

        # 启动后续读者（应该被写者优先阻塞）
        late_reader_thread = threading.Thread(target=late_reader)
        late_reader_thread.start()

        # 等待所有线程开始等待
        time.sleep(0.2)
        # 让第一个读者释放锁
        reader_can_proceed.set()

        reader_thread.join()
        writer_thread.join()
        late_reader_thread.join()

        # 验证写优先：写者应该在后续读者之前执行
        writer_idx = events.index("writer_executed")
        late_reader_idx = events.index("late_reader_executed")
        assert writer_idx < late_reader_idx, f"写者应该先于后续读者执行: {events}"

    def test_timeout_read_lock(self):
        """测试读锁超时"""
        lock = ReadWriteLock()

        # 先获取写锁
        with lock.write_lock():
            # 尝试获取读锁，应该超时
            start_time = time.time()
            try:
                with lock.read_lock(timeout=0.1):
                    pass
                assert False, "应该超时"
            except TimeoutError:
                elapsed = time.time() - start_time
                assert elapsed >= 0.1, f"超时时间不正确: {elapsed}"

    def test_timeout_write_lock(self):
        """测试写锁超时"""
        lock = ReadWriteLock()

        # 先获取读锁
        with lock.read_lock():
            # 尝试获取写锁，应该超时
            start_time = time.time()
            try:
                with lock.write_lock(timeout=0.1):
                    pass
                assert False, "应该超时"
            except TimeoutError:
                elapsed = time.time() - start_time
                assert elapsed >= 0.1, f"超时时间不正确: {elapsed}"


# ============================================================================
# 会话管理器并发测试
# ============================================================================

class TestConcurrentSessionManager:
    """会话管理器并发测试"""

    def test_create_session_thread_safe(self):
        """测试会话创建线程安全"""
        manager = ConcurrentSessionManager()
        session_ids = []
        num_threads = 10

        def create_session():
            context = manager.create_session()
            session_ids.append(context.session_id)

        threads = [threading.Thread(target=create_session) for _ in range(num_threads)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 验证所有会话都被创建
        assert len(session_ids) == num_threads
        # 验证没有重复ID
        assert len(set(session_ids)) == num_threads

    def test_concurrent_read_operations(self):
        """测试并发读操作"""
        manager = ConcurrentSessionManager()
        context = manager.create_session()

        # 添加一些消息
        for i in range(10):
            manager.add_message(context.session_id, "user", f"message_{i}")

        read_count = 20
        results = []

        def read_messages():
            messages = manager.get_messages(context.session_id)
            results.append(len(messages) if messages else 0)

        threads = [threading.Thread(target=read_messages) for _ in range(read_count)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 所有读操作都应该成功
        assert len(results) == read_count
        # 所有读操作应该看到相同的消息数量
        assert all(r == 10 for r in results)

    def test_concurrent_write_operations(self):
        """测试并发写操作"""
        manager = ConcurrentSessionManager()
        context = manager.create_session()

        write_count = 50

        def add_message(i: int):
            manager.add_message(context.session_id, "user", f"message_{i}")

        threads = [threading.Thread(target=add_message, args=(i,)) for i in range(write_count)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 验证所有消息都被添加
        messages = manager.get_messages(context.session_id)
        assert len(messages) == write_count

    def test_mixed_read_write_operations(self):
        """测试混合读写操作"""
        manager = ConcurrentSessionManager()
        context = manager.create_session()

        operations = []
        num_readers = 10
        num_writers = 5

        def reader():
            for _ in range(5):
                state = manager.get_session_state(context.session_id)
                operations.append(("read", state))
                time.sleep(0.01)

        def writer():
            for i in range(5):
                manager.add_message(context.session_id, "user", f"msg_{i}")
                operations.append(("write", None))
                time.sleep(0.02)

        threads = []
        threads.extend([threading.Thread(target=reader) for _ in range(num_readers)])
        threads.extend([threading.Thread(target=writer) for _ in range(num_writers)])

        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 验证操作都执行了
        assert len(operations) == num_readers * 5 + num_writers * 5

    def test_session_deletion_thread_safe(self):
        """测试会话删除线程安全"""
        manager = ConcurrentSessionManager()

        # 创建会话
        session_ids = []
        for _ in range(5):
            context = manager.create_session()
            session_ids.append(context.session_id)

        errors = []

        def delete_session(session_id: str):
            try:
                # 先读再删
                manager.get_messages(session_id)
                manager.delete_session(session_id)
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=delete_session, args=(sid,)) for sid in session_ids]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 不应该有错误
        assert len(errors) == 0, f"删除出错: {errors}"
        # 所有会话应该被删除
        assert len(manager.list_sessions()) == 0

    def test_max_sessions_limit(self):
        """测试最大会话数限制"""
        manager = ConcurrentSessionManager(max_sessions=5)
        errors = []

        def create_and_handle():
            try:
                manager.create_session()
            except RuntimeError as e:
                errors.append(str(e))

        threads = [threading.Thread(target=create_and_handle) for _ in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 应该有一些超限错误
        assert len(errors) > 0
        # 最终会话数不应超过限制
        assert len(manager.list_sessions()) <= 5


# ============================================================================
# 性能压力测试
# ============================================================================

class TestPerformance:
    """性能压力测试"""

    def test_read_performance(self):
        """测试读操作性能"""
        manager = ConcurrentSessionManager()
        context = manager.create_session()

        # 添加初始数据
        for i in range(100):
            manager.add_message(context.session_id, "user", f"message_{i}")

        num_reads = 1000
        start_time = time.time()

        def read_task():
            for _ in range(10):
                manager.get_messages(context.session_id)

        threads = [threading.Thread(target=read_task) for _ in range(num_reads // 10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        elapsed = time.time() - start_time
        reads_per_second = num_reads / elapsed

        print(f"\n读性能: {reads_per_second:.0f} 读/秒 ({num_reads}次读操作耗时 {elapsed:.2f}秒)")
        assert reads_per_second > 100, "读性能过低"

    def test_write_performance(self):
        """测试写操作性能"""
        manager = ConcurrentSessionManager()
        context = manager.create_session()

        num_writes = 100
        start_time = time.time()

        def write_task(i: int):
            manager.add_message(context.session_id, "user", f"message_{i}")

        threads = [threading.Thread(target=write_task, args=(i,)) for i in range(num_writes)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        elapsed = time.time() - start_time
        writes_per_second = num_writes / elapsed

        print(f"\n写性能: {writes_per_second:.0f} 写/秒 ({num_writes}次写操作耗时 {elapsed:.2f}秒)")

    def test_high_concurrency(self):
        """高并发压力测试"""
        manager = ConcurrentSessionManager()

        num_threads = 50
        operations_per_thread = 100
        total_operations = num_threads * operations_per_thread

        start_time = time.time()

        def worker(thread_id: int):
            # 创建会话
            context = manager.get_or_create_session(f"session_{thread_id}")
            for i in range(operations_per_thread):
                if i % 3 == 0:
                    # 写操作
                    manager.add_message(context.session_id, "user", f"msg_{i}")
                else:
                    # 读操作
                    manager.get_messages(context.session_id)

        threads = [threading.Thread(target=worker, args=(i,)) for i in range(num_threads)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        elapsed = time.time() - start_time
        ops_per_second = total_operations / elapsed

        print(f"\n高并发测试: {ops_per_second:.0f} 操作/秒 ({total_operations}次操作耗时 {elapsed:.2f}秒)")
        print(f"  线程数: {num_threads}")
        print(f"  每线程操作数: {operations_per_thread}")


# ============================================================================
# 正确性验证测试
# ============================================================================

class TestCorrectness:
    """正确性验证测试"""

    def test_data_consistency(self):
        """测试数据一致性"""
        manager = ConcurrentSessionManager()
        context = manager.create_session()

        expected_count = 100
        counter = {"value": 0}

        def increment():
            def update(ctx: ExecutionContext):
                ctx.iteration += 1
                counter["value"] += 1
            manager.update_session(context.session_id, update)

        threads = [threading.Thread(target=increment) for _ in range(expected_count)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 验证计数器
        def get_iteration(ctx: ExecutionContext):
            return ctx.iteration

        actual_iteration = manager.read_session(context.session_id, get_iteration)
        assert actual_iteration == expected_count, f"迭代计数不一致: 期望{expected_count}, 实际{actual_iteration}"
        assert counter["value"] == expected_count

    def test_no_deadlock(self):
        """测试无死锁"""
        lock = ReadWriteLock()

        def mixed_operations():
            for _ in range(100):
                if random.random() < 0.3:
                    with lock.write_lock(timeout=1):
                        time.sleep(0.001)
                else:
                    with lock.read_lock(timeout=1):
                        time.sleep(0.001)

        threads = [threading.Thread(target=mixed_operations) for _ in range(20)]
        for t in threads:
            t.start()

        # 设置超时，如果死锁会超时
        for t in threads:
            t.join(timeout=10)
            assert not t.is_alive(), "检测到可能的死锁"

    def test_lock_state_integrity(self):
        """测试锁状态完整性"""
        lock = ReadWriteLock()

        def check_state():
            state = lock.get_state()
            # 读者和写者不能同时存在
            if state["readers"] > 0:
                assert state["writers"] == 0, "读者存在时不能有写者"
            if state["writers"] > 0:
                assert state["readers"] == 0, "写者存在时不能有读者"
                assert state["writers"] == 1, "最多只能有一个写者"

        def worker():
            for _ in range(50):
                if random.random() < 0.2:
                    with lock.write_lock():
                        check_state()
                        time.sleep(0.001)
                else:
                    with lock.read_lock():
                        check_state()
                        time.sleep(0.001)

        threads = [threading.Thread(target=worker) for _ in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        # 最终状态应该归零
        state = lock.get_state()
        assert state["readers"] == 0
        assert state["writers"] == 0


# ============================================================================
# Checkpoint 测试
# ============================================================================

class TestCheckpoint:
    """Checkpoint测试"""

    def test_checkpoint_save_load(self):
        """测试Checkpoint保存和加载"""
        import tempfile
        import os

        with tempfile.TemporaryDirectory() as tmpdir:
            manager = ConcurrentSessionManager()
            cp_manager = CheckpointManager(tmpdir)

            # 创建会话并添加数据
            context = manager.create_session()
            manager.add_message(context.session_id, "user", "Hello")
            manager.add_message(context.session_id, "assistant", "Hi there!")

            # 保存checkpoint
            cp_id = cp_manager.save_checkpoint(context, "test_save")

            # 加载checkpoint
            loaded = cp_manager.load_checkpoint(context.session_id, cp_id)

            assert loaded is not None
            assert loaded.session_id == context.session_id
            assert len(loaded.messages) == 2

    def test_checkpoint_concurrent_save(self):
        """测试并发保存Checkpoint"""
        import tempfile

        with tempfile.TemporaryDirectory() as tmpdir:
            manager = ConcurrentSessionManager()
            cp_manager = CheckpointManager(tmpdir)

            context = manager.create_session()
            saved_ids = []

            def save_checkpoint(i: int):
                manager.add_message(context.session_id, "user", f"msg_{i}")
                cp_id = cp_manager.save_checkpoint(context, f"concurrent_{i}")
                saved_ids.append(cp_id)

            threads = [threading.Thread(target=save_checkpoint, args=(i,)) for i in range(10)]
            for t in threads:
                t.start()
            for t in threads:
                t.join()

            # 所有保存应该成功
            assert len(saved_ids) == 10


# ============================================================================
# 运行测试
# ============================================================================

def run_all_tests():
    """运行所有测试"""
    print("=" * 70)
    print("SessionManager 并发测试")
    print("=" * 70)

    # 使用pytest运行
    exit_code = pytest.main([__file__, "-v", "-s", "--tb=short"])

    if exit_code == 0:
        print("\n" + "=" * 70)
        print("所有测试通过!")
        print("=" * 70)

    return exit_code


if __name__ == "__main__":
    run_all_tests()
