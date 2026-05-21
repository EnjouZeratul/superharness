"""Session 性能基准测试"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import time
import statistics


class BenchResult:
    def __init__(self, name: str):
        self.name = name
        self.times = []
    
    def add(self, elapsed: float):
        self.times.append(elapsed)
    
    @property
    def mean(self) -> float:
        return statistics.mean(self.times) if self.times else 0
    
    @property
    def median(self) -> float:
        return statistics.median(self.times) if self.times else 0
    
    @property
    def stdev(self) -> float:
        return statistics.stdev(self.times) if len(self.times) > 1 else 0
    
    @property
    def min(self) -> float:
        return min(self.times) if self.times else 0
    
    @property
    def max(self) -> float:
        return max(self.times) if self.times else 0


def bench(func, iterations=100):
    """运行基准测试"""
    result = BenchResult(func.__name__)
    
    # 预热
    func()
    
    # 正式测试
    for _ in range(iterations):
        start = time.perf_counter()
        func()
        elapsed = time.perf_counter() - start
        result.add(elapsed)
    
    return result


def format_result(result):
    return f"""
{result.name}:
  Iterations: {len(result.times)}
  Mean:   {result.mean*1000:.4f} ms
  Median: {result.median*1000:.4f} ms
  StdDev: {result.stdev*1000:.4f} ms
  Min:    {result.min*1000:.4f} ms
  Max:    {result.max*1000:.4f} ms
"""


# ========================================
# Session 基准测试
# ========================================

def bench_session_creation():
    """测试 Session 创建时间"""
    from continuum_sdk.agent import Session
    return Session()


def bench_session_with_id():
    """测试带 ID 的 Session 创建"""
    from continuum_sdk.agent import Session
    return Session(id="test-session-id")


def bench_session_add_message():
    """测试添加消息"""
    from continuum_sdk.agent import Session, MessageRole
    session = Session()
    session.add_user_message("Hello")
    return session


def bench_session_add_100_messages():
    """测试批量添加100条消息"""
    from continuum_sdk.agent import Session
    session = Session()
    for i in range(100):
        session.add_user_message(f"Message {i}")
    return session


def bench_session_get_messages():
    """测试获取消息列表"""
    from continuum_sdk.agent import Session
    session = Session()
    for i in range(50):
        session.add_user_message(f"Message {i}")
    return session.get_messages()


def bench_session_export():
    """测试导出"""
    from continuum_sdk.agent import Session
    session = Session(id="export-test")
    session.add_user_message("Test message")
    return session.export()


def bench_session_import():
    """测试导入"""
    from continuum_sdk.agent import Session
    session = Session(id="import-test")
    session.add_user_message("Test")
    exported = session.export()
    return Session.from_export(exported)


def bench_session_message_count():
    """测试消息计数"""
    from continuum_sdk.agent import Session
    session = Session()
    for i in range(100):
        session.add_user_message(f"Message {i}")
    return session.message_count


def bench_session_clear():
    """测试清空消息"""
    from continuum_sdk.agent import Session
    session = Session()
    for i in range(100):
        session.add_user_message(f"Message {i}")
    session.clear_messages()
    return session


def bench_session_metadata():
    """测试元数据操作"""
    from continuum_sdk.agent import Session
    session = Session()
    for i in range(10):
        session.set_metadata(f"key_{i}", f"value_{i}")
    return session


def run_session_benchmarks():
    """运行所有 Session 基准测试"""
    print("=" * 50)
    print("Session Performance Benchmarks")
    print("=" * 50)
    
    tests = [
        ("Session Creation", bench_session_creation, 1000),
        ("Session with ID", bench_session_with_id, 1000),
        ("Add 1 message", bench_session_add_message, 1000),
        ("Add 100 messages", bench_session_add_100_messages, 100),
        ("Get messages (50)", bench_session_get_messages, 500),
        ("Session export", bench_session_export, 1000),
        ("Session import", bench_session_import, 1000),
        ("Message count (100)", bench_session_message_count, 500),
        ("Clear (100 msgs)", bench_session_clear, 100),
        ("Metadata (10 keys)", bench_session_metadata, 1000),
    ]
    
    results = []
    for name, func, iterations in tests:
        print(f"\nRunning: {name}...")
        func.__name__ = name
        result = bench(func, iterations)
        results.append(result)
        print(format_result(result))
    
    return results


if __name__ == "__main__":
    run_session_benchmarks()
