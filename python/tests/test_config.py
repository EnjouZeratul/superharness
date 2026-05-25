"""
测试配置加载器

支持的环境变量（按优先级）:
1. CONTINUUM_API_KEY / CONTINUUM_BASE_URL（推荐）
2. CONTINUUM_API_KEY / CONTINUUM_BASE_URL（兼容）
3. ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL（兼容）
4. OPENAI_API_KEY（多提供商支持）
"""

import os
from pathlib import Path

# 配置文件路径
PROJECT_ROOT = Path(__file__).parent.parent.parent
ENV_FILE = PROJECT_ROOT / ".env"
TEST_ENV_FILE = PROJECT_ROOT / ".env.test"


def _load_env_file(filepath: Path):
    """从 env 文件加载配置（不覆盖已存在的环境变量）"""
    if not filepath.exists():
        return
    with open(filepath, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, value = line.split("=", 1)
                key = key.strip()
                value = value.strip()
                if key not in os.environ:
                    os.environ[key] = value


def load_env():
    """加载环境配置（按优先级，不覆盖已存在的环境变量）"""
    _load_env_file(ENV_FILE)
    _load_env_file(TEST_ENV_FILE)


def get_api_key() -> str | None:
    """
    获取 API 密钥（按优先级）

    优先级:
    1. CONTINUUM_API_KEY
    2. CONTINUUM_API_KEY
    3. ANTHROPIC_API_KEY
    """
    load_env()
    return (
        os.environ.get("CONTINUUM_API_KEY")
        or os.environ.get("CONTINUUM_API_KEY")
        or os.environ.get("ANTHROPIC_API_KEY")
    )


def get_base_url() -> str:
    """
    获取 API 基础 URL（按优先级）

    优先级:
    1. CONTINUUM_BASE_URL
    2. CONTINUUM_BASE_URL
    3. ANTHROPIC_BASE_URL
    4. 默认: https://api.anthropic.com/v1
    """
    load_env()
    return (
        os.environ.get("CONTINUUM_BASE_URL")
        or os.environ.get("CONTINUUM_BASE_URL")
        or os.environ.get("ANTHROPIC_BASE_URL")
        or "https://api.anthropic.com/v1"
    )


def get_model() -> str:
    """获取模型名称"""
    load_env()
    return (
        os.environ.get("CONTINUUM_MODEL")
        or os.environ.get("CONTINUUM_MODEL")
        or os.environ.get("ANTHROPIC_MODEL")
        or "claude-sonnet-4-6"
    )


def is_api_available() -> bool:
    """检查 API 是否可用"""
    key = get_api_key()
    if key is None:
        return False
    placeholders = ["your-api-key", "sk-test", "placeholder", "xxx"]
    return not any(p in key.lower() for p in placeholders)
