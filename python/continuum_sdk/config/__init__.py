"""
Config Module

Configuration management for Continuum SDK.

Provides flexible configuration options:
    - Environment variables (highest priority)
    - TOML/JSON configuration files
    - Multi-provider support (Anthropic, OpenAI, Google)
    - Automatic environment variable expansion

Configuration Priority (highest to lowest):
    1. Environment variables (CONTINUUM_* > CONTINUUM_* > ANTHROPIC_*)
    2. Project-level config (.continuum/config.toml)
    3. User-level config (~/.continuum/config.toml)
    4. Default values

Environment Variables:
    - CONTINUUM_API_KEY / CONTINUUM_API_KEY: API key for current provider
    - CONTINUUM_PROVIDER / CONTINUUM_PROVIDER: Active provider name
    - CONTINUUM_MODEL / CONTINUUM_MODEL: Model name
    - CONTINUUM_BASE_URL / CONTINUUM_BASE_URL: API base URL (optional)

Quick Usage:
    >>> from continuum import Config
    >>> config = Config.from_default()  # Auto-load
    >>> config.use("openai")  # Switch provider

Config File Format (TOML):
    [providers.anthropic]
    api_key = "${ANTHROPIC_API_KEY}"
    base_url = "https://api.anthropic.com/v1"
    model = "claude-sonnet-4-6"

    [settings]
    session_auto_save = true
    checkpoint_enabled = true
"""

from .loader import (
    Config,
    ConfigLoader,
    Provider,
    ProviderConfig,
    get_user_config_dir,
    load_config,
)
from .providers import (
    ProviderInfo,
    ProviderType,
    get_default_model,
    get_default_small_model,
    get_env_key_name,
    get_provider_info,
    list_models,
    list_providers,
)

__all__ = [
    # Core
    "Config",
    "ConfigLoader",
    "load_config",
    "get_user_config_dir",
    # Provider types
    "Provider",
    "ProviderConfig",
    "ProviderType",
    "ProviderInfo",
    # Provider helpers
    "get_provider_info",
    "list_providers",
    "get_default_model",
    "get_default_small_model",
    "get_env_key_name",
    "list_models",
]
