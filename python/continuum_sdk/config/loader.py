"""
Config Loader - Enhanced Configuration Management

Configuration loading and management for Continuum SDK with:
- Environment variable support (CONTINUUM_* as primary, CONTINUUM_* as fallback)
- TOML configuration file support
- Environment variable expansion (${VAR_NAME})
- Multi-provider management
- Security: Whitelist-based environment variable access
"""

import json
import os
import re
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Any

# Security: Whitelist of allowed environment variables
ALLOWED_ENV_VARS = {
    # Continuum namespace
    "CONTINUUM_API_KEY",
    "CONTINUUM_BASE_URL",
    "CONTINUUM_PROVIDER",
    "CONTINUUM_MODEL",
    "CONTINUUM_SMALL_MODEL",
    "CONTINUUM_DEFAULT_MODEL",
    "CONTINUUM_LOG_LEVEL",
    "CONTINUUM_MAX_TOKENS",
    "CONTINUUM_TIMEOUT",
    "CONTINUUM_MAX_ITERATIONS",
    "CONTINUUM_AUDIT_ENABLED",
    "CONTINUUM_AUDIT_LOG_PATH",
    "CONTINUUM_AUDIT_RETENTION",
    # Provider-specific (standard)
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_MODEL",
    "OPENAI_API_KEY",
    "OPENAI_BASE_URL",
    "OPENAI_MODEL",
    "GOOGLE_API_KEY",
    "GOOGLE_BASE_URL",
    "GEMINI_API_KEY",
    "GEMINI_BASE_URL",
    # Legacy support (deprecated)
    "SUPERHARNESS_API_KEY",
    "SUPERHARNESS_BASE_URL",
}


def _get_env(name: str, default: str | None = None) -> str | None:
    """
    Securely get environment variable with whitelist validation.

    Only allows predefined environment variables to prevent
    environment variable injection attacks.
    """
    if name not in ALLOWED_ENV_VARS:
        # Log warning but don't expose the variable name
        import warnings
        warnings.warn(
            f"Attempted to access non-whitelisted environment variable",
            UserWarning,
            stacklevel=3,
        )
        return default
    return os.environ.get(name, default)

# TOML support (Python 3.11+ has built-in, otherwise use tomllib)
try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        tomllib = None


class Provider(Enum):
    """LLM 提供商"""

    ANTHROPIC = "anthropic"
    OPENAI = "openai"
    GOOGLE = "google"
    GEMINI = "gemini"
    CUSTOM = "custom"


@dataclass
class ProviderConfig:
    """提供商配置"""

    name: str
    api_key: str | None = None
    base_url: str | None = None
    model: str | None = None
    small_model: str | None = None
    default_model: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "api_key": self.api_key,
            "base_url": self.base_url,
            "model": self.model,
            "small_model": self.small_model,
            "default_model": self.default_model,
        }


class Config:
    """
    Continuum 配置类

    支持多种配置来源，优先级：环境变量 > 配置文件 > 默认值

    Usage:
        # 方式1: 环境变量自动读取
        config = Config.from_env()

        # 方式2: 从配置文件加载
        config = Config.from_file("~/.continuum/config.toml")

        # 方式3: 显式配置
        config = Config(
            provider="anthropic",
            api_key="xxx",
            model="claude-sonnet-4-6"
        )

        # 切换提供商
        config.use("openai")
    """

    # 环境变量前缀 (CONTINUUM_* 优先，CONTINUUM_* 兼容)
    ENV_PREFIX = "CONTINUUM_"
    ENV_PREFIX_FALLBACK = "CONTINUUM_"

    # 默认配置目录
    DEFAULT_CONFIG_DIRS = [
        ".",
        ".claude",
        "~/.config/continuum",
        "~/.continuum",
    ]

    # 默认配置文件名
    DEFAULT_CONFIG_FILES = ["config.toml", "continuum.toml", "config.json"]

    def __init__(
        self,
        provider: str = "anthropic",
        api_key: str | None = None,
        base_url: str | None = None,
        api_format: str | None = None,
        model: str | None = None,
        small_model: str | None = None,
        effort_level: str = "medium",
        disable_traffic: bool = False,
        budget: float | None = None,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        worktrees_dir: str | None = None,
        plugins_dir: str | None = None,
        log_level: str = "info",
        audit_enabled: bool = True,
        audit_retention_days: int = 90,
        **kwargs,
    ):
        """
        创建配置

        Args:
            provider: LLM 提供商 (anthropic|openai|google|custom|together|groq|...)
            api_key: API 密钥
            base_url: API 基础 URL（用于自定义端点或代理）
            api_format: API 格式 (anthropic|openai|google)。不设置则根据 provider 自动推断
            model: 主模型名称
            small_model: 小模型名称（用于简单任务）
            effort_level: 努力级别 (low|medium|high|max)
            disable_traffic: 是否禁用流量统计
            budget: 预算上限
            max_tokens: 最大 token 数
            temperature: 温度参数
            worktrees_dir: worktrees 目录
            plugins_dir: 插件目录
            log_level: 日志级别
            audit_enabled: 是否启用审计
            audit_retention_days: 审计日志保留天数
            **kwargs: 其他配置项
        """
        self._data: dict[str, Any] = {
            "provider": provider,
            "api_key": api_key,
            "base_url": base_url,
            "api_format": api_format,
            "model": model,
            "small_model": small_model,
            "effort_level": effort_level,
            "disable_traffic": disable_traffic,
            "budget": budget,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "worktrees_dir": worktrees_dir,
            "plugins_dir": plugins_dir,
            "log_level": log_level,
            "audit_enabled": audit_enabled,
            "audit_retention_days": audit_retention_days,
        }
        self._data.update(kwargs)

        # 提供商配置存储
        self._providers: dict[str, ProviderConfig] = {}
        self._current_provider: str | None = None

    # ==================== 属性访问 ====================

    @property
    def provider(self) -> str:
        """当前提供商"""
        return self._data.get("provider", "anthropic")

    @property
    def api_key(self) -> str | None:
        """API 密钥"""
        return self._data.get("api_key")

    @property
    def model(self) -> str:
        """模型名称"""
        return self._data.get("model") or self._get_default_model()

    @property
    def small_model(self) -> str | None:
        """小模型名称"""
        return self._data.get("small_model")

    @property
    def base_url(self) -> str | None:
        """API 基础 URL"""
        return self._data.get("base_url")

    @property
    def api_format(self) -> str | None:
        """API 请求格式 (anthropic|openai|google)"""
        return self._data.get("api_format")

    @property
    def effort_level(self) -> str:
        """努力级别"""
        return self._data.get("effort_level", "medium")

    @property
    def disable_traffic(self) -> bool:
        """是否禁用流量统计"""
        return self._data.get("disable_traffic", False)

    @property
    def budget(self) -> float | None:
        """预算上限"""
        return self._data.get("budget")

    @property
    def max_tokens(self) -> int:
        """最大 token 数"""
        return self._data.get("max_tokens", 4096)

    @property
    def temperature(self) -> float:
        """温度参数"""
        return self._data.get("temperature", 0.7)

    @property
    def audit_enabled(self) -> bool:
        """审计是否启用"""
        return self._data.get("audit_enabled", True)

    def get(self, key: str, default: Any = None) -> Any:
        """获取配置项"""
        return self._data.get(key, default)

    def set(self, key: str, value: Any) -> None:
        """设置配置项"""
        self._data[key] = value

    def update(self, data: dict[str, Any]) -> None:
        """批量更新配置"""
        self._data.update(data)

    def to_dict(self) -> dict[str, Any]:
        """转换为字典"""
        return self._data.copy()

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "Config":
        """从字典创建配置"""
        return cls(**data)

    # ==================== 便捷加载方法 ====================

    @classmethod
    def from_env(cls) -> "Config":
        """
        从环境变量加载配置

        环境变量优先级：CONTINUUM_* > {PROVIDER}_* > ANTHROPIC_*
        例：CONTINUUM_PROVIDER=openai, CONTINUUM_API_KEY=xxx
        """
        env_mapping = {
            "PROVIDER": "provider",
            "API_KEY": "api_key",
            "BASE_URL": "base_url",
            "API_FORMAT": "api_format",  # 新增：anthropic, openai, google
            "MODEL": "model",
            "SMALL_MODEL": "small_model",
            "EFFORT_LEVEL": "effort_level",
            "DISABLE_TRAFFIC": (
                "disable_traffic",
                lambda x: x.lower() in ("1", "true", "yes"),
            ),
            "BUDGET": ("budget", float),
            "MAX_TOKENS": ("max_tokens", int),
            "TEMPERATURE": ("temperature", float),
            "WORKTREES_DIR": "worktrees_dir",
            "PLUGINS_DIR": "plugins_dir",
            "LOG_LEVEL": "log_level",
            "AUDIT_ENABLED": (
                "audit_enabled",
                lambda x: x.lower() in ("1", "true", "yes"),
            ),
            "AUDIT_RETENTION": ("audit_retention_days", int),
        }

        # Provider-specific env var names
        provider_env_keys = {
            "anthropic": "ANTHROPIC_API_KEY",
            "openai": "OPENAI_API_KEY",
            "google": "GOOGLE_API_KEY",
            "gemini": "GOOGLE_API_KEY",
        }

        config_data = {}

        # First pass: get provider to know which fallback to use
        provider = (
            _get_env(f"{cls.ENV_PREFIX}PROVIDER")
            or _get_env(f"{cls.ENV_PREFIX_FALLBACK}PROVIDER")
            or "anthropic"
        )
        config_data["provider"] = provider

        # Get provider-specific fallback env var
        provider_fallback_key = provider_env_keys.get(provider, "ANTHROPIC_API_KEY")

        for env_suffix, config_key in env_mapping.items():
            # 检查多个环境变量前缀（按优先级）
            if env_suffix == "API_KEY":
                # For API_KEY, use provider-specific fallback
                value = (
                    _get_env(f"{cls.ENV_PREFIX}{env_suffix}")
                    or _get_env(f"{cls.ENV_PREFIX_FALLBACK}{env_suffix}")
                    or _get_env(provider_fallback_key)
                )
            elif env_suffix == "BASE_URL":
                # For BASE_URL, check provider-specific var too
                value = (
                    _get_env(f"{cls.ENV_PREFIX}{env_suffix}")
                    or _get_env(f"{cls.ENV_PREFIX_FALLBACK}{env_suffix}")
                    or _get_env(f"{provider.upper()}_BASE_URL")
                )
            else:
                value = (
                    _get_env(f"{cls.ENV_PREFIX}{env_suffix}")
                    or _get_env(f"{cls.ENV_PREFIX_FALLBACK}{env_suffix}")
                    or _get_env(f"ANTHROPIC_{env_suffix}")
                )

            if value:
                if isinstance(config_key, tuple):
                    key, converter = config_key
                    try:
                        config_data[key] = converter(value)
                    except (ValueError, TypeError):
                        pass
                else:
                    config_data[config_key] = value

        return cls(**config_data)

    @classmethod
    def from_file(cls, path: str) -> "Config":
        """
        从配置文件加载

        支持 TOML 和 JSON 格式
        """
        file_path = Path(path).expanduser()
        if not file_path.exists():
            raise FileNotFoundError(f"Config file not found: {path}")

        data = cls._load_file(file_path)

        # 展开环境变量引用
        data = cls._expand_env_vars(data)

        return cls(**data)

    @classmethod
    def from_default(cls) -> "Config":
        """
        从默认位置加载配置

        按优先级：环境变量 > 配置文件 > 默认值
        """
        # 1. 从环境变量
        config = cls.from_env()

        # 2. 查找并加载配置文件
        config_file = cls._find_config_file()
        if config_file:
            file_data = cls._load_file(config_file)
            file_data = cls._expand_env_vars(file_data)
            config._data.update(file_data)

        return config

    # ==================== 提供商管理 ====================

    def use(self, provider: str) -> "Config":
        """
        切换提供商

        Args:
            provider: 提供商名称 (anthropic|openai|google|custom)

        Returns:
            self（支持链式调用）
        """
        self._data["provider"] = provider

        # 如果有预配置的提供商信息，加载它
        if provider in self._providers:
            prov_config = self._providers[provider]
            if prov_config.api_key:
                self._data["api_key"] = prov_config.api_key
            if prov_config.base_url:
                self._data["base_url"] = prov_config.base_url
            if prov_config.model:
                self._data["model"] = prov_config.model

        return self

    def add_provider(
        self,
        name: str,
        api_key: str | None = None,
        base_url: str | None = None,
        model: str | None = None,
        small_model: str | None = None,
    ) -> None:
        """
        添加提供商配置

        Args:
            name: 提供商名称
            api_key: API 密钥
            base_url: 基础 URL
            model: 默认模型
            small_model: 小模型
        """
        self._providers[name] = ProviderConfig(
            name=name,
            api_key=api_key,
            base_url=base_url,
            model=model,
            small_model=small_model,
        )

    def list_providers(self) -> list[str]:
        """列出所有配置的提供商"""
        return list(self._providers.keys())

    # ==================== 内部方法 ====================

    def _get_default_model(self) -> str:
        """获取提供商的默认模型"""
        provider = self.provider
        defaults = {
            "anthropic": "claude-sonnet-4-6",
            "openai": "gpt-4.1",
            "google": "gemini-2.5-pro",
            "gemini": "gemini-2.5-pro",
        }
        return defaults.get(provider, "claude-sonnet-4-6")

    @classmethod
    def _find_config_file(cls) -> Path | None:
        """查找配置文件"""
        for dir_path in cls.DEFAULT_CONFIG_DIRS:
            dir_expanded = Path(dir_path).expanduser()
            for config_name in cls.DEFAULT_CONFIG_FILES:
                path = dir_expanded / config_name
                if path.exists():
                    return path
        return None

    @classmethod
    def _load_file(cls, path: Path) -> dict[str, Any]:
        """从文件加载配置"""
        suffix = path.suffix.lower()

        try:
            if suffix == ".toml":
                if tomllib is None:
                    print(
                        "Warning: TOML support requires Python 3.11+ or tomli package"
                    )
                    return {}
                with open(path, "rb") as f:
                    return tomllib.load(f)
            elif suffix == ".json":
                with open(path, encoding="utf-8") as f:
                    return json.load(f)
            else:
                # 尝试自动检测
                content = path.read_text(encoding="utf-8")
                if content.strip().startswith("{"):
                    return json.loads(content)
                elif tomllib:
                    with open(path, "rb") as f:
                        return tomllib.load(f)
        except Exception as e:
            print(f"Warning: Failed to load config from {path}: {e}")

        return {}

    @classmethod
    def _expand_env_vars(cls, data: dict[str, Any]) -> dict[str, Any]:
        """
        展开配置中的环境变量引用

        支持 ${VAR_NAME} 和 $VAR_NAME 格式
        """
        pattern = re.compile(r"\$\{([^}]+)\}|\$([A-Za-z_][A-Za-z0-9_]*)")

        def expand_value(value: Any) -> Any:
            if isinstance(value, str):

                def replacer(match):
                    var_name = match.group(1) or match.group(2)
                    # Security: use whitelisted env access
                    return _get_env(var_name) or match.group(0)

                return pattern.sub(replacer, value)
            elif isinstance(value, dict):
                return {k: expand_value(v) for k, v in value.items()}
            elif isinstance(value, list):
                return [expand_value(item) for item in value]
            return value

        return expand_value(data)

    def __repr__(self) -> str:
        return f"Config(provider={self.provider}, model={self.model})"


# 便捷函数
def load_config(path: str | None = None) -> Config:
    """
    加载配置

    Args:
        path: 配置文件路径（可选，默认自动查找）

    Returns:
        Config 实例
    """
    if path:
        return Config.from_file(path)
    return Config.from_default()


def get_user_config_dir() -> Path:
    """获取用户配置目录"""
    config_dir = Path.home() / ".config" / "continuum"
    config_dir.mkdir(parents=True, exist_ok=True)
    return config_dir


# Backward compatibility wrapper
class ConfigLoader:
    """
    配置加载器（向后兼容）

    推荐直接使用 Config 类方法：
        Config.from_env()
        Config.from_file(path)
        Config.from_default()
    """

    def __init__(self, config_path: str | None = None):
        self._config_path = config_path
        self._config: Config | None = None

    def load(self) -> Config:
        """加载配置"""
        if self._config_path:
            self._config = Config.from_file(self._config_path)
        else:
            self._config = Config.from_default()
        return self._config

    def get_config(self) -> Config | None:
        """获取已加载的配置"""
        return self._config

    def save(self, path: str | None = None) -> None:
        """保存配置到文件"""
        if not self._config:
            raise ValueError("No config loaded")
        save_path = Path(path or self._config_path or "config.json")
        save_path.parent.mkdir(parents=True, exist_ok=True)
        with open(save_path, "w") as f:
            json.dump(self._config.to_dict(), f, indent=2)

    @staticmethod
    def get_default_config() -> Config:
        """获取默认配置"""
        return Config()
