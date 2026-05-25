"""
Provider Configuration

Multi-provider management for LLM services with support for:
- Anthropic API format
- OpenAI-compatible API format (most providers)
- Custom providers with configurable format
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional


class ProviderType(Enum):
    """提供商类型"""
    ANTHROPIC = "anthropic"
    OPENAI = "openai"
    GOOGLE = "google"
    GEMINI = "gemini"
    CUSTOM = "custom"


class ApiFormat(Enum):
    """API 请求格式"""
    ANTHROPIC = "anthropic"  # Anthropic 原生格式
    OPENAI = "openai"        # OpenAI 兼容格式（大多数提供商）
    GOOGLE = "google"        # Google AI 格式


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
    api_format: ApiFormat = ApiFormat.OPENAI  # 默认使用 OpenAI 兼容格式


# 内置提供商配置
BUILTIN_PROVIDERS: Dict[str, ProviderInfo] = {
    # Anthropic 原生格式
    "anthropic": ProviderInfo(
        name="anthropic",
        display_name="Anthropic (Claude)",
        default_model="claude-sonnet-4-6",
        default_small_model="claude-haiku-4-5",
        default_base_url="https://api.anthropic.com",
        env_key_name="ANTHROPIC_API_KEY",
        models=[],
        api_format=ApiFormat.ANTHROPIC,
    ),
    # OpenAI 格式
    "openai": ProviderInfo(
        name="openai",
        display_name="OpenAI (GPT)",
        default_model="gpt-4.1",
        default_small_model="gpt-4.1-mini",
        default_base_url="https://api.openai.com/v1",
        env_key_name="OPENAI_API_KEY",
        models=[],
        api_format=ApiFormat.OPENAI,
    ),
    # Google AI 格式
    "google": ProviderInfo(
        name="google",
        display_name="Google (Gemini)",
        default_model="gemini-2.5-pro",
        default_small_model="gemini-2.5-flash",
        default_base_url="https://generativelanguage.googleapis.com/v1beta",
        env_key_name="GOOGLE_API_KEY",
        models=[],
        api_format=ApiFormat.GOOGLE,
    ),
    "gemini": ProviderInfo(
        name="gemini",
        display_name="Google Gemini",
        default_model="gemini-2.5-pro",
        default_small_model="gemini-2.5-flash",
        default_base_url="https://generativelanguage.googleapis.com/v1beta",
        env_key_name="GOOGLE_API_KEY",
        models=[],
        api_format=ApiFormat.GOOGLE,
    ),
    # 常用 OpenAI 兼容提供商
    "together": ProviderInfo(
        name="together",
        display_name="Together AI",
        default_model="meta-llama/Llama-3-70b-chat-hf",
        default_base_url="https://api.together.xyz/v1",
        env_key_name="TOGETHER_API_KEY",
        models=[],
        api_format=ApiFormat.OPENAI,
    ),
    "groq": ProviderInfo(
        name="groq",
        display_name="Groq",
        default_model="llama-3.3-70b-versatile",
        default_base_url="https://api.groq.com/openai/v1",
        env_key_name="GROQ_API_KEY",
        models=[],
        api_format=ApiFormat.OPENAI,
    ),
    "deepseek": ProviderInfo(
        name="deepseek",
        display_name="DeepSeek",
        default_model="deepseek-chat",
        default_base_url="https://api.deepseek.com/v1",
        env_key_name="DEEPSEEK_API_KEY",
        models=[],
        api_format=ApiFormat.OPENAI,
    ),
    "moonshot": ProviderInfo(
        name="moonshot",
        display_name="Moonshot (Kimi)",
        default_model="moonshot-v1-8k",
        default_base_url="https://api.moonshot.cn/v1",
        env_key_name="MOONSHOT_API_KEY",
        models=[],
        api_format=ApiFormat.OPENAI,
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


def get_default_base_url(provider: str) -> Optional[str]:
    """获取提供商的默认 API URL"""
    info = BUILTIN_PROVIDERS.get(provider)
    if info:
        return info.default_base_url
    return None


def get_api_format(provider: str) -> ApiFormat:
    """获取提供商的 API 请求格式"""
    info = BUILTIN_PROVIDERS.get(provider)
    if info:
        return info.api_format
    return ApiFormat.OPENAI


def list_models(provider: str) -> List[str]:
    """列出提供商支持的模型"""
    info = BUILTIN_PROVIDERS.get(provider)
    if info:
        return info.models.copy()
    return []
