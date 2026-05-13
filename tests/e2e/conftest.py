"""E2E 测试配置"""

import pytest
import asyncio


@pytest.fixture(scope="session")
def event_loop():
    """Session 级别 event loop"""
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def e2e_timeout():
    """E2E 测试超时设置"""
    return 30.0  # 30 秒