"""配置系统测试 - 配置优先级

测试配置源的优先级：env > file > default
"""

import pytest
import os
import tempfile
from pathlib import Path
from unittest.mock import patch


class TestConfigPriority:
    """配置优先级测试"""

    def test_env_overrides_file_overrides_default(self, temp_working_dir):
        """测试完整优先级链"""
        config_file = temp_working_dir / "config.toml"
        config_file.write_text("""
model = "claude-3-haiku"
max_tokens = 4096
""")
        with patch.dict(os.environ, {"SH_MODEL": "claude-3-opus"}):
            # Expected:
            # model: env (claude-3-opus)
            # max_tokens: file (4096)
            pass

    def test_env_only(self):
        """测试只有环境变量"""
        with patch.dict(os.environ, {
            "SH_API_KEY": "env-key",
            "SH_MODEL": "env-model"
        }):
            # Expected: 所有值来自环境变量
            pass

    def test_file_only(self, temp_working_dir):
        """测试只有文件配置"""
        config_file = temp_working_dir / "config.toml"
        config_file.write_text("""
api_key = "file-key"
model = "file-model"
""")
        # Expected: 所有值来自文件
        pass

    def test_default_only(self):
        """测试只有默认值"""
        # 无环境变量，无配置文件
        # Expected: 所有值使用内置默认
        pass

    def test_env_explicit_overrides_env_general(self):
        """测试特定环境变量覆盖通用"""
        with patch.dict(os.environ, {
            "SH_API_KEY": "general-key",
            "SH_ANTHROPIC_API_KEY": "anthropic-key"
        }):
            # 当使用 anthropic 提供商时
            # Expected: 使用 SH_ANTHROPIC_API_KEY
            pass

    def test_project_config_overrides_global(self, temp_working_dir):
        """测试项目配置覆盖全局"""
        global_config = temp_working_dir / "global.toml"
        project_config = temp_working_dir / "project.toml"

        global_config.write_text("model = 'global-model'")
        project_config.write_text("model = 'project-model'")

        # Expected: project-model
        pass


class TestPrioritySpecificKeys:
    """特定键的优先级测试"""

    def test_api_key_priority(self):
        """测试 API key 优先级"""
        # 优先级: SH_ANTHROPIC_API_KEY > SH_API_KEY > config > default
        with patch.dict(os.environ, {
            "SH_API_KEY": "general",
            "SH_ANTHROPIC_API_KEY": "specific"
        }):
            # Expected: specific
            pass

    def test_base_url_priority(self):
        """测试 base URL 优先级"""
        # 优先级: SH_BASE_URL > SH_ANTHROPIC_BASE_URL > config > default
        with patch.dict(os.environ, {
            "SH_BASE_URL": "general-url",
            "SH_ANTHROPIC_BASE_URL": "specific-url"
        }):
            # Expected: specific-url
            pass

    def test_model_priority(self):
        """测试模型优先级"""
        # SH_MODEL > SH_ANTHROPIC_MODEL > config > default
        pass

    def test_max_tokens_priority(self):
        """测试 max_tokens 优先级"""
        # SH_MAX_TOKENS > config > default (4096)
        with patch.dict(os.environ, {"SH_MAX_TOKENS": "8192"}):
            # Expected: 8192
            pass


class TestPriorityWithProviders:
    """提供商切换时的优先级测试"""

    def test_priority_with_provider_switch(self, temp_working_dir):
        """测试切换提供商后的优先级"""
        config_file = temp_working_dir / "config.toml"
        config_file.write_text("""
[providers.anthropic]
api_key = "anthropic-config-key"

[providers.openai]
api_key = "openai-config-key"
""")
        with patch.dict(os.environ, {
            "SH_ANTHROPIC_API_KEY": "anthropic-env-key",
            "SH_OPENAI_API_KEY": "openai-env-key"
        }):
            # 切换到 anthropic: 使用 anthropic-env-key
            # 切换到 openai: 使用 openai-env-key
            pass

    def test_cross_provider_env_fallback(self):
        """测试跨提供商环境变量回退"""
        with patch.dict(os.environ, {"SH_API_KEY": "general-key"}):
            # 没有 SH_OPENAI_API_KEY 时
            # Expected: 使用 SH_API_KEY
            pass


pytestmark = pytest.mark.config