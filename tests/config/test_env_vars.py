"""配置系统测试 - 环境变量读取

测试环境变量的正确读取和解析。
"""

import pytest
import os
import sys
from unittest.mock import patch

# Add python directory to path for continuum_sdk
_python_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 'python')
sys.path.insert(0, _python_dir)

from continuum_sdk.config.loader import Config


class TestEnvVarReading:
    """环境变量读取测试"""

    def test_read_api_key_from_env(self):
        """测试从环境变量读取 API key"""
        with patch.dict(os.environ, {"CONTINUUM_API_KEY": "test-key-123"}, clear=False):
            # 清除其他可能的 key
            for key in ["CONTINUUM_API_KEY", "ANTHROPIC_API_KEY"]:
                os.environ.pop(key, None)
            config = Config.from_env()
            assert config.api_key == "test-key-123"

    def test_read_model_from_env(self):
        """测试从环境变量读取模型"""
        with patch.dict(os.environ, {"CONTINUUM_MODEL": "claude-3-opus"}, clear=False):
            for key in ["CONTINUUM_MODEL", "ANTHROPIC_MODEL"]:
                os.environ.pop(key, None)
            config = Config.from_env()
            assert config.model == "claude-3-opus"

    def test_read_base_url_from_env(self):
        """测试从环境变量读取 base URL"""
        with patch.dict(os.environ, {"CONTINUUM_BASE_URL": "https://custom.api.com"}, clear=False):
            for key in ["CONTINUUM_BASE_URL", "ANTHROPIC_BASE_URL"]:
                os.environ.pop(key, None)
            config = Config.from_env()
            assert config.base_url == "https://custom.api.com"

    def test_env_var_not_set_returns_none(self):
        """测试环境变量未设置时返回 None"""
        # 清除所有 API key 相关环境变量
        for key in ["CONTINUUM_API_KEY", "CONTINUUM_API_KEY", "ANTHROPIC_API_KEY"]:
            os.environ.pop(key, None)
        config = Config.from_env()
        assert config.api_key is None

    def test_env_var_empty_string_treated_as_not_set(self):
        """测试环境变量为空字符串时视为未设置"""
        # 空字符串在 Python 中是 falsy，Config.from_env() 使用 or 链式判断
        # 因此空字符串会被视为未设置，返回 None
        with patch.dict(os.environ, {"CONTINUUM_API_KEY": ""}, clear=False):
            # 清除其他可能的 fallback
            for key in ["CONTINUUM_API_KEY", "ANTHROPIC_API_KEY"]:
                os.environ.pop(key, None)
            config = Config.from_env()
            # 空字符串被视为未设置，返回 None
            assert config.api_key is None

    def test_env_var_with_special_chars(self):
        """测试环境变量包含特殊字符"""
        special_key = "key-with-$pecial#chars!"
        with patch.dict(os.environ, {"CONTINUUM_API_KEY": special_key}, clear=False):
            config = Config.from_env()
            assert config.api_key == special_key


class TestEnvVarReference:
    """环境变量引用解析测试"""

    def test_env_expansion_simple(self):
        """测试简单环境变量展开"""
        with patch.dict(os.environ, {"MY_VAR": "test_value"}, clear=False):
            config = Config()
            config.set("test_key", "${MY_VAR}")
            # Config 不自动展开，这是配置文件加载时的行为
            assert config.get("test_key") == "${MY_VAR}"

    def test_config_from_env_priority(self):
        """测试环境变量优先级"""
        # CONTINUUM_* 优先于 CONTINUUM_* 优先于 ANTHROPIC_*
        with patch.dict(os.environ, {
            "ANTHROPIC_MODEL": "anthropic-model",
            "CONTINUUM_MODEL": "sh-model",
            "CONTINUUM_MODEL": "continuum-model"
        }, clear=False):
            config = Config.from_env()
            assert config.model == "continuum-model"


class TestEnvVarPriority:
    """环境变量优先级测试"""

    def test_continuum_over_continuum_over_anthropic(self):
        """测试 CONTINUUM > CONTINUUM > ANTHROPIC 优先级"""
        with patch.dict(os.environ, {
            "ANTHROPIC_API_KEY": "anthropic-key",
            "CONTINUUM_API_KEY": "sh-key",
            "CONTINUUM_API_KEY": "continuum-key"
        }, clear=False):
            config = Config.from_env()
            assert config.api_key == "continuum-key"

    def test_fallback_to_continuum(self):
        """测试回退到 CONTINUUM_*"""
        for key in ["CONTINUUM_API_KEY", "ANTHROPIC_API_KEY"]:
            os.environ.pop(key, None)
        with patch.dict(os.environ, {"CONTINUUM_API_KEY": "sh-only-key"}, clear=False):
            config = Config.from_env()
            assert config.api_key == "sh-only-key"

    def test_fallback_to_anthropic(self):
        """测试回退到 ANTHROPIC_*"""
        for key in ["CONTINUUM_API_KEY", "CONTINUUM_API_KEY"]:
            os.environ.pop(key, None)
        with patch.dict(os.environ, {"ANTHROPIC_API_KEY": "anthropic-only-key"}, clear=False):
            config = Config.from_env()
            assert config.api_key == "anthropic-only-key"


pytestmark = pytest.mark.config
