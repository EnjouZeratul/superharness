"""配置系统测试 - 多提供商

测试多提供商的配置和切换。
"""

import pytest
import os
import sys
import json
import tempfile
from pathlib import Path
from unittest.mock import patch

# Add python directory to path for continuum_sdk
_python_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 'python')
sys.path.insert(0, _python_dir)

from continuum_sdk.config.loader import Config
from continuum_sdk.config.providers import (
    BUILTIN_PROVIDERS,
    get_provider_info,
    list_providers,
    get_default_model,
    get_env_key_name,
    list_models,
)


class TestProviderConfig:
    """提供商配置测试"""

    def test_anthropic_provider_config(self, tmp_path):
        """测试 Anthropic 提供商配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({
            "provider": "anthropic",
            "api_key": "sk-ant-xxx",
            "model": "claude-3-haiku"
        }))
        config = Config.from_file(str(config_file))
        assert config.provider == "anthropic"
        assert config.api_key == "sk-ant-xxx"

    def test_openai_provider_config(self, tmp_path):
        """测试 OpenAI 提供商配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({
            "provider": "openai",
            "api_key": "sk-xxx",
            "model": "gpt-4"
        }))
        config = Config.from_file(str(config_file))
        assert config.provider == "openai"
        assert config.api_key == "sk-xxx"

    def test_gemini_provider_config(self, tmp_path):
        """测试 Gemini 提供商配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({
            "provider": "gemini",
            "api_key": "gemini-key",
            "model": "gemini-pro"
        }))
        config = Config.from_file(str(config_file))
        assert config.provider == "gemini"
        assert config.api_key == "gemini-key"

    def test_custom_provider_config(self, tmp_path):
        """测试自定义提供商配置"""
        config = Config(
            provider="custom",
            api_key="custom-key",
            base_url="https://custom.api.com/v1",
            model="custom-model"
        )
        assert config.provider == "custom"
        assert config.base_url == "https://custom.api.com/v1"

    def test_multiple_providers_registered(self):
        """测试多个提供商同时配置"""
        config = Config(provider="anthropic")
        config.add_provider("openai", api_key="openai-key", model="gpt-4")
        config.add_provider("gemini", api_key="gemini-key", model="gemini-pro")

        providers = config.list_providers()
        assert "openai" in providers
        assert "gemini" in providers


class TestProviderSwitching:
    """提供商切换测试"""

    def test_switch_provider(self):
        """测试切换提供商"""
        config = Config(provider="anthropic")
        config.add_provider("openai", api_key="openai-key", model="gpt-4.1")

        config.use("openai")
        assert config.provider == "openai"
        assert config.api_key == "openai-key"

    def test_switch_to_unregistered_provider(self):
        """测试切换到未注册的提供商"""
        config = Config(provider="anthropic")
        config.use("unregistered")
        # 未注册提供商仍然可以切换，但不会加载特定配置
        assert config.provider == "unregistered"

    def test_get_current_provider(self):
        """测试获取当前提供商"""
        config = Config(provider="anthropic")
        assert config.provider == "anthropic"

    def test_provider_specific_config_loads(self):
        """测试提供商特定配置加载"""
        config = Config(provider="anthropic")
        config.add_provider("openai", api_key="openai-key", model="gpt-4.1", small_model="gpt-4.1-mini")

        config.use("openai")
        assert config.model == "gpt-4.1"


class TestProviderFallback:
    """提供商回退测试"""

    def test_default_provider_is_anthropic(self):
        """测试默认提供商是 Anthropic"""
        config = Config()
        assert config.provider == "anthropic"

    def test_provider_default_models(self):
        """测试提供商默认模型"""
        assert get_default_model("anthropic") == "claude-sonnet-4-6"
        assert get_default_model("openai") == "gpt-4.1"
        assert get_default_model("gemini") == "gemini-2.5-pro"

    def test_env_key_names(self):
        """测试提供商环境变量名"""
        assert get_env_key_name("anthropic") == "ANTHROPIC_API_KEY"
        assert get_env_key_name("openai") == "OPENAI_API_KEY"
        assert get_env_key_name("gemini") == "GOOGLE_API_KEY"


class TestProviderInfo:
    """提供商信息测试"""

    def test_list_all_providers(self):
        """测试列出所有提供商"""
        providers = list_providers()
        assert "anthropic" in providers
        assert "openai" in providers
        assert "google" in providers
        assert "gemini" in providers

    def test_get_provider_info(self):
        """测试获取提供商信息"""
        info = get_provider_info("anthropic")
        assert info is not None
        assert info.name == "anthropic"
        assert info.display_name == "Anthropic (Claude)"
        assert info.default_model == "claude-sonnet-4-6"

    def test_get_unknown_provider_info(self):
        """测试获取未知提供商信息"""
        info = get_provider_info("unknown")
        assert info is None

    def test_list_provider_models(self):
        """测试列出提供商支持的模型"""
        models = list_models("anthropic")
        assert len(models) >= 1
        assert "claude-sonnet-4-6" in models


pytestmark = pytest.mark.config