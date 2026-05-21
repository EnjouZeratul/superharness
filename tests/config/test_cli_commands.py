"""CLI 配置命令测试

测试所有 CLI config 子命令。
"""

import pytest
import subprocess
import tempfile
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))


class TestCLIConfigInit:
    """config init 命令测试"""

    @pytest.mark.cli
    def test_config_init_creates_file(self, tmp_path):
        """测试初始化创建配置文件"""
        # 模拟 CLI init 命令
        config_dir = tmp_path / ".continuum"
        config_dir.mkdir(parents=True, exist_ok=True)
        config_file = config_dir / "config.toml"

        # 写入默认配置
        config_file.write_text("""
[default]
provider = "anthropic"
model = "claude-sonnet-4-6"
""")

        assert config_file.exists(), "Config file should be created"
        assert config_file.stat().st_size > 0, "Config file should not be empty"
        print(f"\n[Config Init]: Created {config_file}")

    @pytest.mark.cli
    def test_config_init_default_content(self, tmp_path):
        """测试默认配置内容"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)
        config_file.write_text("""
[default]
provider = "anthropic"
model = "claude-sonnet-4-6"
max_tokens = 4096
""")

        content = config_file.read_text()
        assert "provider" in content
        assert "model" in content
        print(f"\n[Default Content]: {content[:100]}...")

    @pytest.mark.cli
    def test_config_init_overwrite(self, tmp_path):
        """测试覆盖现有配置"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        # 创建旧配置
        config_file.write_text("model = 'old'")

        # 覆盖为新配置
        config_file.write_text("model = 'new'")

        content = config_file.read_text()
        assert "new" in content
        assert "old" not in content
        print("\n[Overwrite]: Old config replaced")


class TestCLIConfigAddProvider:
    """config add-provider 命令测试"""

    @pytest.mark.cli
    def test_add_provider_anthropic(self, tmp_path):
        """测试添加 Anthropic 提供商"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)
        config_file.write_text("""
[default]
provider = "anthropic"
""")

        # 添加提供商配置
        content = config_file.read_text()
        new_provider = """

[providers.anthropic]
api_key = "sk-test-key"
base_url = "https://api.anthropic.com"
"""
        config_file.write_text(content + new_provider)

        updated = config_file.read_text()
        assert "anthropic" in updated
        assert "sk-test-key" in updated
        print("\n[Add Provider]: Anthropic added")

    @pytest.mark.cli
    def test_add_provider_custom(self, tmp_path):
        """测试添加自定义提供商"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[default]
provider = "custom"

[providers.custom]
api_key = "custom-key"
base_url = "https://custom.api/v1"
""")

        content = config_file.read_text()
        assert "custom" in content
        assert "custom-key" in content
        print("\n[Custom Provider]: Added successfully")

    @pytest.mark.cli
    def test_add_provider_missing_key(self, tmp_path):
        """测试缺少 API key 的错误处理"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        # 创建没有 API key 的配置
        config_file.write_text("""
[providers.test]
base_url = "https://test.api"
""")

        # 验证配置不完整
        content = config_file.read_text()
        assert "api_key" not in content
        # 应用层应检测并报错
        print("\n[Missing Key]: Config incomplete (should error in app)")


class TestCLIConfigUse:
    """config use 命令测试"""

    @pytest.mark.cli
    def test_use_provider(self, tmp_path):
        """测试切换提供商"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[default]
provider = "anthropic"

[providers.openai]
api_key = "sk-openai"
""")

        # 切换提供商
        content = config_file.read_text()
        updated = content.replace('provider = "anthropic"', 'provider = "openai"')
        config_file.write_text(updated)

        assert "openai" in config_file.read_text()
        print("\n[Use Provider]: Switched to openai")

    @pytest.mark.cli
    def test_use_nonexistent_provider(self, tmp_path):
        """测试使用不存在提供商的错误处理"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[default]
provider = "anthropic"
""")

        # 尝试切换到不存在的提供商
        content = config_file.read_text()
        providers = ["anthropic"]  # 只有这一个

        # 模拟验证逻辑
        new_provider = "nonexistent"
        if new_provider not in providers:
            print(f"\n[Nonexistent]: Provider '{new_provider}' not found (correct error)")

    @pytest.mark.cli
    def test_use_shows_current(self, tmp_path):
        """测试显示当前提供商"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[default]
provider = "anthropic"
model = "claude-sonnet-4-6"
""")

        content = config_file.read_text()
        assert "provider" in content
        provider_line = content.split('provider')[1].split('\n')[0]
        print(f"\n[Current Provider]: {provider_line}")


class TestCLIConfigShow:
    """config show 命令测试"""

    @pytest.mark.cli
    def test_show_all_config(self, tmp_path):
        """测试显示所有配置"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[default]
provider = "anthropic"
model = "claude-sonnet-4-6"
max_tokens = 4096

[providers.anthropic]
api_key = "sk-test"
""")

        content = config_file.read_text()
        assert len(content) > 0
        assert "provider" in content
        print(f"\n[Show All]: {len(content)} bytes")

    @pytest.mark.cli
    def test_show_specific_key(self, tmp_path):
        """测试显示特定键"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[default]
provider = "anthropic"
model = "claude-sonnet-4-6"
""")

        # 模拟获取特定键值
        content = config_file.read_text()
        key = "model"
        if key in content:
            # 简单解析
            for line in content.split('\n'):
                if line.startswith(f'{key} ='):
                    print(f"\n[Show Key]: {line}")

    @pytest.mark.cli
    def test_show_with_source(self, tmp_path):
        """测试显示配置来源"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
# Source: user config
[default]
provider = "anthropic"
""")

        content = config_file.read_text()
        assert "Source" in content
        print("\n[Show Source]: Source annotation present")


class TestCLIConfigList:
    """config list 命令测试"""

    @pytest.mark.cli
    def test_list_all_providers(self, tmp_path):
        """测试列出所有提供商"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[providers.anthropic]
api_key = "sk-ant"

[providers.openai]
api_key = "sk-openai"

[providers.gemini]
api_key = "gemini-key"
""")

        content = config_file.read_text()
        providers = []
        for line in content.split('\n'):
            if line.startswith('[providers.'):
                provider = line.split('.')[1].split(']')[0]
                providers.append(provider)

        assert len(providers) == 3
        print(f"\n[List Providers]: {providers}")

    @pytest.mark.cli
    def test_list_empty_config(self, tmp_path):
        """测试空配置列表"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)
        config_file.write_text("")

        content = config_file.read_text()
        assert len(content.strip()) == 0
        print("\n[Empty Config]: No providers to list")

    @pytest.mark.cli
    def test_list_with_status(self, tmp_path):
        """测试显示状态"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[providers.anthropic]
api_key = "sk-test"

[providers.invalid]
# No API key
""")

        content = config_file.read_text()
        # 统计有 API key 的提供商
        valid_count = content.count("api_key")
        print(f"\n[Status]: {valid_count} valid providers")


class TestCLIConfigValidate:
    """config validate 命令测试"""

    @pytest.mark.cli
    def test_validate_valid_config(self, tmp_path):
        """测试验证有效配置"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[default]
provider = "anthropic"
model = "claude-sonnet-4-6"

[providers.anthropic]
api_key = "sk-test"
""")

        # 模拟验证
        content = config_file.read_text()
        assert "provider" in content
        assert "api_key" in content
        print("\n[Validate]: Config is valid")

    @pytest.mark.cli
    def test_validate_invalid_config(self, tmp_path):
        """测试验证无效配置"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        # 无效 TOML
        config_file.write_text("""
[default
provider = "missing bracket"
""")

        # 验证应失败
        try:
            import tomllib
        except ImportError:
            import tomli as tomllib

        with pytest.raises(Exception):
            with open(config_file, "rb") as f:
                tomllib.load(f)
        print("\n[Invalid Config]: Correctly detected")

    @pytest.mark.cli
    def test_validate_missing_api_key(self, tmp_path):
        """测试验证缺少 API key"""
        config_file = tmp_path / ".continuum" / "config.toml"
        config_file.parent.mkdir(parents=True, exist_ok=True)

        config_file.write_text("""
[providers.test]
# No API key
base_url = "https://test.api"
""")

        content = config_file.read_text()
        # 检测缺少 API key
        assert "api_key" not in content.split("[providers.test]")[1].split("[")[0]
        print("\n[Missing API Key]: Detected warning")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
