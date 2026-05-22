"""
Provider Configuration

Multi-provider management for LLM services.
"""

from typing import Optional, Dict, List
from dataclasses import dataclass, field
from enum import Enum


class ProviderType(Enum):
    """提供商类型"""
    ANTHROPIC = "anthropic"
    OPENAI = "openai"
    GOOGLE = "google"
    GEMINI = "gemini"
    CUSTOM = "custom"


@dataclass
class ProviderInfo:
    """提供商信息"""
    name: str
    display_name: str
    default_model: str
    default_small_model: Optional[str] = None
    default_base_url: Optional[str] = None
    env_key_name: Optional[str] = None
    models: List[str] = field(default_factory=list)


# 内置提供商配置
BUILTIN_PROVIDERS: Dict[str, ProviderInfo] = {
    "anthropic": ProviderInfo(
        name="anthropic",
        display_name="Anthropic (Claude)",
        default_model="claude-sonnet-4-6",
        default_small_model="claude-haiku-4-5",
        env_key_name="ANTHROPIC_API_KEY",
        models=[
            "claude-opus-4-6",
            "claude-sonnet-4-6",
            "claude-haiku-4-5",
        ],
    ),
    "openai": ProviderInfo(
        name="openai",
        display_name="OpenAI (GPT)",
        default_model="gpt-4.1",
        default_small_model="gpt-4.1-mini",
        env_key_name="OPENAI_API_KEY",
        models=[
            "gpt-4.1",
            "gpt-4.1-mini",
            "gpt-4.1-nano",
            "o3",
            "o4-mini",
        ],
    ),
    "google": ProviderInfo(
        name="google",
        display_name="Google (Gemini)",
        default_model="gemini-2.5-pro",
        default_small_model="gemini-2.5-flash",
        env_key_name="GOOGLE_API_KEY",
        models=[
            "gemini-2.5-pro",
            "gemini-2.5-flash",
        ],
    ),
    "gemini": ProviderInfo(
        name="gemini",
        display_name="Google Gemini",
        default_model="gemini-2.5-pro",
        default_small_model="gemini-2.5-flash",
        env_key_name="GOOGLE_API_KEY",
        models=[
            "gemini-2.5-pro",
            "gemini-2.5-flash",
        ],
    ),
}


def get_provider_info(name: str) -> Optional[ProviderInfo]:
    """获取提供商信息"""
    return BUILTIN_PROVIDERS.get(name)


def list_providers() -> List[str]:
    """列出所有内置提供商"""
    return list(BUILTIN_PROVIDERS.keys())


def get_default_model(provider: str) -> str:
    """获取提供商的默认模型"""
    info = BUILTIN_PROVIDERS.get(provider)
    if info:
        return info.default_model
    return "claude-sonnet-4-6"


def get_default_small_model(provider: str) -> Optional[str]:
    """获取提供商的默认小模型"""
    info = BUILTIN_PROVIDERS.get(provider)
    if info:
        return info.default_small_model
    return None


def get_env_key_name(provider: str) -> Optional[str]:
    """获取提供商的环境变量密钥名"""
    info = BUILTIN_PROVIDERS.get(provider)
    if info:
        return info.env_key_name
    return None


def list_models(provider: str) -> List[str]:
    """列出提供商支持的模型"""
    info = BUILTIN_PROVIDERS.get(provider)
    if info:
        return info.models.copy()
    return []
