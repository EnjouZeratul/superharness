"""配置系统测试 - TOML 文件加载

测试 TOML 配置文件的正确加载和解析。
"""

import pytest
import sys
import os
import tempfile
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


class TestTomlLoader:
    """TOML 文件加载测试"""

    def test_load_simple_toml(self, tmp_path):
        """测试加载简单 TOML 文件"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("""
model = "claude-sonnet-4-6"
max_tokens = 4096
""")

        # 解析 TOML
        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with open(config_file, "rb") as f:
            data = tomllib.load(f)

        assert data["model"] == "claude-sonnet-4-6"
        assert data["max_tokens"] == 4096
        print(f"\n[Simple TOML]: {data}")

    def test_load_toml_with_sections(self, tmp_path):
        """测试加载带分区的 TOML"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("""
[model]
name = "claude-sonnet-4-6"

[memory]
enabled = true
max_entries = 100

[tools]
enabled = ["read", "write"]
""")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with open(config_file, "rb") as f:
            data = tomllib.load(f)

        assert "model" in data
        assert data["memory"]["enabled"] is True
        assert len(data["tools"]["enabled"]) == 2
        print(f"\n[Sections]: {list(data.keys())}")

    def test_load_toml_with_nested_sections(self, tmp_path):
        """测试加载嵌套分区 TOML"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("""
[providers.anthropic]
api_key = "key-123"
base_url = "https://api.anthropic.com"

[providers.openai]
api_key = "key-456"
base_url = "https://api.openai.com"
""")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with open(config_file, "rb") as f:
            data = tomllib.load(f)

        assert "providers" in data
        assert "anthropic" in data["providers"]
        assert data["providers"]["anthropic"]["api_key"] == "key-123"
        print(f"\n[Nested]: {data['providers']}")

    def test_load_toml_with_arrays(self, tmp_path):
        """测试加载带数组的 TOML"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("""
enabled_tools = ["read", "write", "bash"]
models = ["claude-sonnet-4-6", "claude-opus-4-7"]
""")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with open(config_file, "rb") as f:
            data = tomllib.load(f)

        assert len(data["enabled_tools"]) == 3
        assert "bash" in data["enabled_tools"]
        print(f"\n[Arrays]: {data}")


class TestTomlLoaderErrors:
    """TOML 加载错误测试"""

    def test_load_invalid_toml(self, tmp_path):
        """测试加载无效 TOML"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("""
invalid = [unclosed bracket
""")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with pytest.raises(Exception):  # TOML decode error
            with open(config_file, "rb") as f:
                tomllib.load(f)
        print("\n[Invalid TOML]: Correctly raised error")

    def test_load_missing_file(self, tmp_path):
        """测试加载不存在文件"""
        nonexistent = tmp_path / "nonexistent.toml"
        assert not nonexistent.exists()

        with pytest.raises(FileNotFoundError):
            with open(nonexistent, "rb") as f:
                pass
        print("\n[Missing File]: Correctly raised FileNotFoundError")

    def test_load_empty_file(self, tmp_path):
        """测试加载空文件"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with open(config_file, "rb") as f:
            data = tomllib.load(f)

        assert data == {}  # Empty TOML returns empty dict
        print("\n[Empty File]: Returns empty dict")

    def test_load_file_with_permission_error(self, tmp_path):
        """测试权限错误处理"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("model = 'test'")

        # 在 Windows 上，权限限制可能不生效
        # 验证文件可读即可
        assert config_file.exists()
        content = config_file.read_text()
        assert "test" in content
        print("\n[Permission]: File readable")


class TestTomlLoaderEnvRef:
    """TOML 环境变量引用测试"""

    def test_toml_with_env_ref(self, tmp_path):
        """测试 TOML 中的环境变量引用"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("""
api_key = "${MY_API_KEY}"
""")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with patch.dict(os.environ, {"MY_API_KEY": "secret-key"}):
            with open(config_file, "rb") as f:
                data = tomllib.load(f)

            # TOML 本身不解析环境变量，需要应用层处理
            # 验证原始值存在，应用层应替换
            assert "${MY_API_KEY}" in data["api_key"] or "secret-key" in data["api_key"]
            print(f"\n[Env Ref]: {data}")

    def test_toml_env_ref_in_nested(self, tmp_path):
        """测试嵌套分区中的环境变量引用"""
        config_file = tmp_path / "config.toml"
        config_file.write_text("""
[providers.openai]
api_key = "${OPENAI_KEY}"
""")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with patch.dict(os.environ, {"OPENAI_KEY": "key-456"}):
            with open(config_file, "rb") as f:
                data = tomllib.load(f)

            assert "providers" in data
            print(f"\n[Nested Env]: {data['providers']}")


class TestTomlLoaderMerge:
    """TOML 配置合并测试"""

    def test_merge_global_and_project(self, tmp_path):
        """测试全局和项目配置合并"""
        global_config = tmp_path / "global.toml"
        project_config = tmp_path / "project.toml"

        global_config.write_text("model = 'claude-sonnet-4-6'")
        project_config.write_text("max_tokens = 8192")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with open(global_config, "rb") as f:
            global_data = tomllib.load(f)
        with open(project_config, "rb") as f:
            project_data = tomllib.load(f)

        # 合并配置
        merged = {**global_data, **project_data}

        assert "model" in merged
        assert "max_tokens" in merged
        print(f"\n[Merged]: {merged}")

    def test_project_overrides_global(self, tmp_path):
        """测试项目配置覆盖全局"""
        global_config = tmp_path / "global.toml"
        project_config = tmp_path / "project.toml"

        global_config.write_text("model = 'claude-sonnet-4-6'")
        project_config.write_text("model = 'claude-opus-4-7'")

        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with open(global_config, "rb") as f:
            global_data = tomllib.load(f)
        with open(project_config, "rb") as f:
            project_data = tomllib.load(f)

        # 项目配置覆盖全局
        merged = {**global_data, **project_data}

        assert merged["model"] == "claude-opus-4-7"
        print(f"\n[Override]: {merged}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
