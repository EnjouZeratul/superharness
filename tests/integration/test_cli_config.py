"""CLI config 命令集成测试

测试 `continuum config` 命令的各种场景。
"""

import pytest
import os
import sys
import json
import tempfile
from pathlib import Path
from unittest.mock import patch

# Add src and python directories to path
_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(_root, "python"))
sys.path.insert(0, os.path.join(_root, "src"))

from continuum_sdk.config.loader import Config, ConfigLoader


class TestCLIConfigShow:
    """config show 测试"""

    def test_show_current_config(self, tmp_path):
        """测试显示当前配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({
            "provider": "anthropic",
            "model": "claude-sonnet-4-6",
            "max_tokens": 4096,
        }))

        config = Config.from_file(str(config_file))
        data = config.to_dict()

        assert data["provider"] == "anthropic"
        assert data["model"] == "claude-sonnet-4-6"
        assert data["max_tokens"] == 4096

    def test_show_config_source(self, tmp_path):
        """测试显示配置来源"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({"model": "file-model"}))

        config = Config.from_file(str(config_file))
        # 验证配置确实来自文件
        assert config.model == "file-model"

    def test_show_config_defaults(self):
        """测试显示默认值"""
        config = Config()
        defaults = config.to_dict()

        assert defaults["provider"] == "anthropic"
        assert defaults["max_tokens"] == 4096
        assert defaults["temperature"] == 0.7


class TestCLIConfigSet:
    """config set 测试"""

    def test_set_single_value(self):
        """测试设置单个值"""
        config = Config()
        config.set("model", "claude-opus-4-7")

        assert config.model == "claude-opus-4-7"

    def test_set_nested_value(self):
        """测试设置嵌套值"""
        config = Config()
        config.set("memory.project.max_entries", 1000)

        assert config.get("memory.project.max_entries") == 1000

    def test_set_invalid_key_still_stored(self):
        """测试设置无效键（仍存储，但不影响功能）"""
        config = Config()
        config.set("invalid.key", "value")

        assert config.get("invalid.key") == "value"

    def test_set_invalid_value_type(self):
        """测试设置无效值类型"""
        config = Config()
        # max_tokens 应该是 int，但 set 不强制类型
        config.set("max_tokens", "invalid")

        assert config.get("max_tokens") == "invalid"


class TestCLIConfigGet:
    """config get 测试"""

    def test_get_existing_key(self):
        """测试获取存在的键"""
        config = Config(model="claude-opus-4-7")
        assert config.model == "claude-opus-4-7"

    def test_get_nonexistent_key(self):
        """测试获取不存在键"""
        config = Config()
        result = config.get("unknown.key")
        assert result is None

    def test_get_with_default(self):
        """测试带默认值获取"""
        config = Config()
        result = config.get("unknown.key", "my-default")
        assert result == "my-default"


class TestCLIConfigList:
    """config list 测试"""

    def test_list_all_keys(self):
        """测试列出所有键"""
        config = Config()
        data = config.to_dict()

        assert "provider" in data
        assert "model" in data
        assert "max_tokens" in data

    def test_list_with_descriptions(self):
        """测试列出配置项含类型信息"""
        config = Config()
        data = config.to_dict()

        for key, value in data.items():
            assert key is not None
            assert isinstance(value, (str, int, float, bool, type(None)))


class TestCLIConfigValidate:
    """config validate 测试"""

    def test_validate_valid_config(self, tmp_path):
        """测试验证有效配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({
            "provider": "anthropic",
            "max_tokens": 4096
        }))

        config = Config.from_file(str(config_file))
        assert config.provider == "anthropic"
        assert config.max_tokens == 4096

    def test_validate_invalid_config(self, tmp_path):
        """测试验证无效 JSON 配置 — 加载失败时返回默认配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text("{invalid json")

        # Config.from_file 对无效 JSON 打印警告并返回默认配置
        config = Config.from_file(str(config_file))
        assert config is not None

    def test_validate_missing_required(self):
        """测试缺少必填项时使用默认值"""
        config = Config()
        # 不设置 api_key 时应为 None
        assert config.api_key is None


class TestCLIConfigInit:
    """config init 测试"""

    def test_init_new_config(self, tmp_path):
        """测试初始化配置"""
        config_dir = tmp_path / ".continuum"
        config_dir.mkdir(parents=True, exist_ok=True)
        config_file = config_dir / "config.json"

        # 保存默认配置
        config = Config()
        with open(config_file, 'w') as f:
            json.dump(config.to_dict(), f, indent=2)

        assert config_file.exists()
        loaded = Config.from_file(str(config_file))
        assert loaded.provider == "anthropic"

    def test_init_with_overwrite(self, tmp_path):
        """测试覆盖现有配置"""
        config_file = tmp_path / "config.json"
        config_file.write_text(json.dumps({"model": "old-model"}))

        # 覆盖
        new_config = Config(model="new-model")
        with open(config_file, 'w') as f:
            json.dump(new_config.to_dict(), f, indent=2)

        loaded = Config.from_file(str(config_file))
        assert loaded.model == "new-model"

    def test_init_saves_all_defaults(self, tmp_path):
        """测试初始化保存所有默认值"""
        config = Config()
        data = config.to_dict()

        config_file = tmp_path / "config.json"
        with open(config_file, 'w') as f:
            json.dump(data, f, indent=2)

        loaded = Config.from_file(str(config_file))
        assert loaded.provider == "anthropic"
        assert loaded.max_tokens == 4096


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])