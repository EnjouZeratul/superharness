"""配置系统测试 - 配置优先级

测试配置源的优先级：env > file > default
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


class TestConfigPriority:
    """配置优先级测试"""

    def test_env_overrides_file_overrides_default(self, tmp_path):
        """测试完整优先级链"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({
            "model": "file-model",
            "max_tokens": 2048,
        }))

        # 加载文件配置
        config = Config.from_file(str(config_file))
        assert config.model == "file-model"
        assert config.max_tokens == 2048

        # 环境变量覆盖
        with patch.dict(os.environ, {"CONTINUUM_MODEL": "env-model"}, clear=False):
            env_config = Config.from_env()
            assert env_config.model == "env-model"

    def test_env_only(self):
        """测试只有环境变量"""
        with patch.dict(os.environ, {
            "CONTINUUM_API_KEY": "env-key",
            "CONTINUUM_MODEL": "env-model"
        }, clear=False):
            config = Config.from_env()
            assert config.api_key == "env-key"
            assert config.model == "env-model"

    def test_file_only(self, tmp_path):
        """测试只有文件配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({
            "api_key": "file-key",
            "model": "file-model"
        }))
        config = Config.from_file(str(config_file))
        assert config.api_key == "file-key"
        assert config.model == "file-model"

    def test_default_only(self):
        """测试只有默认值"""
        # 清除所有环境变量
        env_keys = [k for k in os.environ if k.startswith(("CONTINUUM_", "CONTINUUM_", "ANTHROPIC_"))]
        with patch.dict(os.environ, {}, clear=False):
            for k in env_keys:
                os.environ.pop(k, None)
            config = Config()
            assert config.provider == "anthropic"
            assert config.max_tokens == 4096
            assert config.temperature == 0.7

    def test_specific_env_overrides_general(self):
        """测试 CONTINUUM 覆盖 CONTINUUM"""
        with patch.dict(os.environ, {
            "CONTINUUM_API_KEY": "general-key",
            "CONTINUUM_API_KEY": "continuum-key"
        }, clear=False):
            config = Config.from_env()
            assert config.api_key == "continuum-key"


class TestPrioritySpecificKeys:
    """特定键的优先级测试"""

    def test_api_key_priority(self):
        """测试 API key 优先级"""
        with patch.dict(os.environ, {
            "ANTHROPIC_API_KEY": "anthropic-key",
            "CONTINUUM_API_KEY": "sh-key",
            "CONTINUUM_API_KEY": "continuum-key"
        }, clear=False):
            config = Config.from_env()
            assert config.api_key == "continuum-key"

    def test_base_url_priority(self):
        """测试 base URL 优先级"""
        with patch.dict(os.environ, {
            "ANTHROPIC_BASE_URL": "https://anthropic.api",
            "CONTINUUM_BASE_URL": "https://sh.api",
            "CONTINUUM_BASE_URL": "https://cc.api"
        }, clear=False):
            config = Config.from_env()
            assert config.base_url == "https://cc.api"

    def test_model_priority(self):
        """测试 model 优先级"""
        with patch.dict(os.environ, {
            "ANTHROPIC_MODEL": "anthropic-model",
            "CONTINUUM_MODEL": "sh-model",
            "CONTINUUM_MODEL": "continuum-model"
        }, clear=False):
            config = Config.from_env()
            assert config.model == "continuum-model"

    def test_max_tokens_from_env(self):
        """测试 max_tokens 从环境变量"""
        with patch.dict(os.environ, {"CONTINUUM_MAX_TOKENS": "8192"}, clear=False):
            config = Config.from_env()
            assert config.max_tokens == 8192


class TestPriorityWithProviders:
    """提供商切换时的优先级测试"""

    def test_provider_switch_changes_defaults(self):
        """测试切换提供商后默认模型改变"""
        config = Config(provider="anthropic")
        assert config.model == "claude-sonnet-4-6"

        config.use("openai")
        assert config.provider == "openai"

    def test_provider_with_preset_config(self):
        """测试预配置提供商的信息加载"""
        config = Config(provider="anthropic")
        config.add_provider("openai", api_key="openai-key", model="gpt-4.1")
        config.use("openai")
        assert config.api_key == "openai-key"


pytestmark = pytest.mark.config
