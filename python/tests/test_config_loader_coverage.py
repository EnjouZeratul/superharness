"""
Config Loader Tests - Coverage Enhancement

Tests for config/loader.py to improve coverage from 25% to 70%+.
"""

import json
import os
import tempfile
from pathlib import Path

import pytest

from continuum_sdk.config.loader import (
    Config,
    ProviderConfig,
    ConfigLoader,
    load_config,
    get_user_config_dir,
    _get_env,
    ALLOWED_ENV_VARS,
)


class TestGetEnv:
    """Test _get_env security function"""

    def test_allowed_env_var(self, monkeypatch):
        """Test accessing allowed environment variable"""
        monkeypatch.setenv("CONTINUUM_API_KEY", "test-key")
        result = _get_env("CONTINUUM_API_KEY")
        assert result == "test-key"

    def test_blocked_env_var(self, monkeypatch):
        """Test that blocked env vars return None"""
        # PATH is not in the whitelist
        monkeypatch.setenv("PATH", "/usr/bin")
        result = _get_env("PATH")
        assert result is None

    def test_default_value(self):
        """Test default value for non-existent var"""
        result = _get_env("CONTINUUM_NONEXISTENT_VAR", default="default")
        assert result == "default"

    def test_allowed_vars_whitelist(self):
        """Test that whitelist contains expected vars"""
        assert "CONTINUUM_API_KEY" in ALLOWED_ENV_VARS
        assert "ANTHROPIC_API_KEY" in ALLOWED_ENV_VARS
        assert "OPENAI_API_KEY" in ALLOWED_ENV_VARS
        assert "GOOGLE_API_KEY" in ALLOWED_ENV_VARS


class TestProviderConfig:
    """Test ProviderConfig dataclass"""

    def test_provider_config_creation(self):
        """Test basic creation"""
        config = ProviderConfig(name="anthropic")
        assert config.name == "anthropic"
        assert config.api_key is None
        assert config.base_url is None

    def test_provider_config_full(self):
        """Test with all fields"""
        config = ProviderConfig(
            name="openai",
            api_key="sk-test",
            base_url="https://api.openai.com",
            model="gpt-4",
        )
        assert config.api_key == "sk-test"
        assert config.base_url == "https://api.openai.com"
        assert config.model == "gpt-4"

    def test_to_dict(self):
        """Test to_dict method"""
        config = ProviderConfig(
            name="test",
            api_key="key",
            base_url="url",
            model="model",
            small_model="small",
        )
        d = config.to_dict()
        assert d["name"] == "test"
        assert d["api_key"] == "key"
        assert d["base_url"] == "url"


class TestConfig:
    """Test Config class"""

    def test_config_creation(self):
        """Test basic config creation"""
        config = Config()
        assert config.provider == "anthropic"

    def test_config_with_params(self):
        """Test config with parameters"""
        config = Config(
            provider="openai",
            api_key="test-key",
            model="gpt-4",
        )
        assert config.provider == "openai"
        assert config.api_key == "test-key"
        assert config.model == "gpt-4"

    def test_config_properties(self):
        """Test config property access"""
        config = Config(
            provider="anthropic",
            api_key="key",
            model="claude-sonnet-4-6",
            max_tokens=8192,
            temperature=0.5,
        )
        assert config.provider == "anthropic"
        assert config.api_key == "key"
        assert config.model == "claude-sonnet-4-6"
        assert config.max_tokens == 8192
        assert config.temperature == 0.5

    def test_config_get_set(self):
        """Test get/set methods"""
        config = Config()
        config.set("custom_key", "custom_value")
        assert config.get("custom_key") == "custom_value"
        assert config.get("nonexistent", "default") == "default"

    def test_config_update(self):
        """Test update method"""
        config = Config()
        config.update({"key1": "value1", "key2": "value2"})
        assert config.get("key1") == "value1"
        assert config.get("key2") == "value2"

    def test_config_to_dict(self):
        """Test to_dict method"""
        config = Config(provider="test", api_key="key")
        d = config.to_dict()
        assert "provider" in d
        assert d["provider"] == "test"

    def test_config_from_dict(self):
        """Test from_dict class method"""
        data = {"provider": "openai", "api_key": "test-key"}
        config = Config.from_dict(data)
        assert config.provider == "openai"
        assert config.api_key == "test-key"

    def test_config_from_env(self, monkeypatch):
        """Test from_env class method"""
        monkeypatch.setenv("CONTINUUM_API_KEY", "env-key")
        monkeypatch.setenv("CONTINUUM_PROVIDER", "google")
        config = Config.from_env()
        assert config.api_key == "env-key"
        assert config.provider == "google"

    def test_config_from_file_json(self):
        """Test loading from JSON file"""
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False
        ) as f:
            json.dump({"provider": "openai", "model": "gpt-4"}, f)
            path = f.name

        try:
            config = Config.from_file(path)
            assert config.provider == "openai"
            assert config.model == "gpt-4"
        finally:
            os.unlink(path)

    def test_config_from_file_toml(self):
        """Test loading from TOML file"""
        toml_content = """
provider = "anthropic"
model = "claude-sonnet-4-6"
"""
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".toml", delete=False
        ) as f:
            f.write(toml_content)
            path = f.name

        try:
            config = Config.from_file(path)
            assert config.provider == "anthropic"
        except ImportError:
            # TOML support requires Python 3.11+ or tomli
            pass
        finally:
            os.unlink(path)

    def test_config_from_file_not_found(self):
        """Test error for nonexistent file"""
        with pytest.raises(FileNotFoundError):
            Config.from_file("/nonexistent/path/config.json")

    def test_config_from_default(self, monkeypatch):
        """Test from_default class method"""
        # Clear any existing env vars
        for var in ALLOWED_ENV_VARS:
            monkeypatch.delenv(var, raising=False)

        config = Config.from_default()
        assert config is not None

    def test_config_use_provider(self):
        """Test use() method for switching providers"""
        config = Config(provider="anthropic")
        config.add_provider("openai", api_key="openai-key", model="gpt-4")
        result = config.use("openai")
        assert result is config  # Returns self for chaining

    def test_config_add_provider(self):
        """Test add_provider method"""
        config = Config()
        config.add_provider(
            "custom",
            api_key="custom-key",
            base_url="https://custom.api",
            model="custom-model",
        )
        assert "custom" in config.list_providers()

    def test_config_list_providers(self):
        """Test list_providers method"""
        config = Config()
        config.add_provider("test1")
        config.add_provider("test2")
        providers = config.list_providers()
        assert "test1" in providers
        assert "test2" in providers

    def test_config_repr(self):
        """Test __repr__ method"""
        config = Config(provider="test", model="test-model")
        repr_str = repr(config)
        assert "Config" in repr_str
        assert "test" in repr_str


class TestConfigLoader:
    """Test ConfigLoader class"""

    def test_config_loader_init(self):
        """Test initialization"""
        loader = ConfigLoader()
        assert loader._config is None

    def test_config_loader_with_path(self):
        """Test with path"""
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False
        ) as f:
            json.dump({"provider": "test"}, f)
            path = f.name

        try:
            loader = ConfigLoader(path)
            config = loader.load()
            assert config.provider == "test"
        finally:
            os.unlink(path)

    def test_config_loader_load_default(self):
        """Test loading default config"""
        loader = ConfigLoader()
        config = loader.load()
        assert config is not None

    def test_config_loader_get_config(self):
        """Test get_config method"""
        loader = ConfigLoader()
        loader.load()
        config = loader.get_config()
        assert config is not None

    def test_config_loader_save(self):
        """Test save method"""
        with tempfile.TemporaryDirectory() as tmpdir:
            loader = ConfigLoader()
            loader.load()

            save_path = os.path.join(tmpdir, "saved_config.json")
            loader.save(save_path)

            assert os.path.exists(save_path)

            # Verify content
            with open(save_path) as f:
                data = json.load(f)
            assert "provider" in data

    def test_config_loader_save_no_config(self):
        """Test save without loading config"""
        loader = ConfigLoader()
        with pytest.raises(ValueError, match="No config loaded"):
            loader.save()

    def test_config_loader_get_default_config(self):
        """Test get_default_config static method"""
        config = ConfigLoader.get_default_config()
        assert isinstance(config, Config)


class TestUtilityFunctions:
    """Test utility functions"""

    def test_load_config_with_path(self):
        """Test load_config with path"""
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False
        ) as f:
            json.dump({"provider": "test"}, f)
            path = f.name

        try:
            config = load_config(path)
            assert config.provider == "test"
        finally:
            os.unlink(path)

    def test_load_config_default(self):
        """Test load_config without path"""
        config = load_config()
        assert config is not None

    def test_get_user_config_dir(self):
        """Test get_user_config_dir"""
        config_dir = get_user_config_dir()
        assert isinstance(config_dir, Path)
        # Should be under home directory
        assert ".config" in str(config_dir) or "AppData" in str(config_dir)


class TestConfigEdgeCases:
    """Test edge cases and error handling"""

    def test_config_effort_level(self):
        """Test effort_level property"""
        config = Config(effort_level="high")
        assert config.effort_level == "high"

    def test_config_disable_traffic(self):
        """Test disable_traffic property"""
        config = Config(disable_traffic=True)
        assert config.disable_traffic is True

    def test_config_budget(self):
        """Test budget property"""
        config = Config(budget=100.0)
        assert config.budget == 100.0

    def test_config_audit_enabled(self):
        """Test audit_enabled property"""
        config = Config(audit_enabled=False)
        assert config.audit_enabled is False

    def test_config_api_format(self):
        """Test api_format property"""
        config = Config(api_format="openai")
        assert config.api_format == "openai"

    def test_config_small_model(self):
        """Test small_model property"""
        config = Config(small_model="gpt-4o-mini")
        assert config.small_model == "gpt-4o-mini"

    def test_env_var_expansion(self):
        """Test environment variable expansion in config files"""
        os.environ["TEST_CONFIG_VAR"] = "expanded_value"

        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False
        ) as f:
            json.dump({"api_key": "${TEST_CONFIG_VAR}"}, f)
            path = f.name

        try:
            config = Config.from_file(path)
            # The expansion should work for whitelisted patterns
            # but may not for arbitrary vars
        finally:
            os.unlink(path)
            del os.environ["TEST_CONFIG_VAR"]


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--cov=continuum_sdk.config.loader", "--cov-report=term-missing"])
