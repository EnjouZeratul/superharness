"""Config 性能基准测试"""

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
# Config 基准测试
# ========================================

def bench_config_creation():
    """测试 Config 创建时间"""
    from continuum_sdk import Config
    return Config()


def bench_config_from_env():
    """测试从环境变量加载"""
    from continuum_sdk import Config
    return Config.from_env()


def bench_config_from_default():
    """测试默认加载"""
    from continuum_sdk import Config
    return Config.from_default()


def bench_config_with_params():
    """测试带参数创建"""
    from continuum_sdk import Config
    return Config(
        provider="anthropic",
        api_key="test-key",
        model="claude-sonnet-4-6",
        max_tokens=8192,
        temperature=0.7
    )


def bench_config_use_provider():
    """测试提供商切换"""
    from continuum_sdk import Config
    config = Config()
    config.add_provider("test", api_key="test-key")
    config.use("test")
    return config


def bench_config_to_dict():
    """测试序列化"""
    from continuum_sdk import Config
    config = Config(provider="anthropic", model="claude-sonnet-4-6")
    return config.to_dict()


def bench_config_from_dict():
    """测试反序列化"""
    from continuum_sdk import Config
    data = {"provider": "anthropic", "model": "claude-sonnet-4-6"}
    return Config.from_dict(data)


def bench_config_load_toml():
    """测试TOML文件加载"""
    from continuum_sdk import Config
    from pathlib import Path
    template_path = Path(__file__).parent.parent.parent / "templates" / "config.toml"
    if template_path.exists():
        return Config.from_file(str(template_path))
    return Config()


def bench_config_add_provider():
    """测试添加提供商"""
    from continuum_sdk import Config
    config = Config()
    config.add_provider("custom", api_key="key", model="model")
    return config


def bench_config_list_providers():
    """测试列出提供商"""
    from continuum_sdk.config import list_providers
    return list_providers()


def run_config_benchmarks():
    """运行所有 Config 基准测试"""
    print("=" * 50)
    print("Config Performance Benchmarks")
    print("=" * 50)
    
    tests = [
        ("Config Creation", bench_config_creation, 1000),
        ("Config from env", bench_config_from_env, 500),
        ("Config from default", bench_config_from_default, 500),
        ("Config with params", bench_config_with_params, 1000),
        ("Config use provider", bench_config_use_provider, 1000),
        ("Config to_dict", bench_config_to_dict, 5000),
        ("Config from_dict", bench_config_from_dict, 5000),
        ("Config load TOML", bench_config_load_toml, 100),
        ("Config add provider", bench_config_add_provider, 1000),
        ("List providers", bench_config_list_providers, 1000),
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
    run_config_benchmarks()
