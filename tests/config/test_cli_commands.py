"""CLI 配置命令测试

测试所有 CLI config 子命令。
"""

import pytest
import subprocess
import tempfile
from pathlib import Path


class TestCLIConfigInit:
    """config init 命令测试"""

    @pytest.mark.cli
    def test_config_init_creates_file(self, temp_working_dir):
        """测试初始化创建配置文件"""
        # subprocess.run(["sh", "config", "init", "--path", temp_working_dir])
        # Expected: .sh/config.toml 文件已创建
        pass

    @pytest.mark.cli
    def test_config_init_default_content(self, temp_working_dir):
        """测试默认配置内容"""
        # Expected: 包含默认提供商配置
        pass

    @pytest.mark.cli
    def test_config_init_overwrite(self, temp_working_dir):
        """测试覆盖现有配置"""
        # 先创建一个配置
        # subprocess.run(["sh", "config", "init", "--force"])
        # Expected: 旧配置被覆盖
        pass

    @pytest.mark.cli
    def test_config_init_interactive(self):
        """测试交互式初始化"""
        # 模拟用户输入
        # Expected: 根据输入生成配置
        pass


class TestCLIConfigAddProvider:
    """config add-provider 命令测试"""

    @pytest.mark.cli
    def test_add_provider_anthropic(self, temp_working_dir):
        """测试添加 Anthropic 提供商"""
        # subprocess.run([
        #     "sh", "config", "add-provider", "anthropic",
        #     "--api-key", "test-key"
        # ])
        # Expected: 配置文件包含 anthropic 分区
        pass

    @pytest.mark.cli
    def test_add_provider_custom(self, temp_working_dir):
        """测试添加自定义提供商"""
        # subprocess.run([
        #     "sh", "config", "add-provider", "custom",
        #     "--api-key", "key",
        #     "--base-url", "https://custom.api"
        # ])
        # Expected: 自定义提供商配置添加成功
        pass

    @pytest.mark.cli
    def test_add_provider_missing_key(self):
        """测试缺少 API key"""
        # subprocess.run(["sh", "config", "add-provider", "test"])
        # Expected: 报错提示缺少必要参数
        pass

    @pytest.mark.cli
    def test_add_provider_update_existing(self, temp_working_dir):
        """测试更新现有提供商"""
        # 先添加，再添加同名提供商
        # Expected: 配置被更新
        pass


class TestCLIConfigUse:
    """config use 命令测试"""

    @pytest.mark.cli
    def test_use_provider(self, temp_working_dir):
        """测试切换提供商"""
        # subprocess.run(["sh", "config", "use", "openai"])
        # Expected: default_provider 变为 openai
        pass

    @pytest.mark.cli
    def test_use_nonexistent_provider(self):
        """测试使用不存在提供商"""
        # subprocess.run(["sh", "config", "use", "nonexistent"])
        # Expected: 报错
        pass

    @pytest.mark.cli
    def test_use_shows_current(self):
        """测试显示当前提供商"""
        # subprocess.run(["sh", "config", "use"])
        # Expected: 显示当前提供商
        pass


class TestCLIConfigShow:
    """config show 命令测试"""

    @pytest.mark.cli
    def test_show_all_config(self, temp_working_dir):
        """测试显示所有配置"""
        # subprocess.run(["sh", "config", "show"])
        # Expected: 显示完整配置
        pass

    @pytest.mark.cli
    def test_show_specific_key(self):
        """测试显示特定键"""
        # subprocess.run(["sh", "config", "show", "model"])
        # Expected: 只显示 model 值
        pass

    @pytest.mark.cli
    def test_show_with_source(self):
        """测试显示配置来源"""
        # subprocess.run(["sh", "config", "show", "--source"])
        # Expected: 显示每个值的来源
        pass


class TestCLIConfigList:
    """config list 命令测试"""

    @pytest.mark.cli
    def test_list_all_providers(self, temp_working_dir):
        """测试列出所有提供商"""
        # subprocess.run(["sh", "config", "list"])
        # Expected: 显示所有配置的提供商
        pass

    @pytest.mark.cli
    def test_list_empty_config(self):
        """测试空配置列表"""
        # Expected: 显示提示信息
        pass

    @pytest.mark.cli
    def test_list_with_status(self):
        """测试显示状态"""
        # subprocess.run(["sh", "config", "list", "--status"])
        # Expected: 显示每个提供商的可用状态
        pass


class TestCLIConfigValidate:
    """config validate 命令测试"""

    @pytest.mark.cli
    def test_validate_valid_config(self, temp_working_dir):
        """测试验证有效配置"""
        # subprocess.run(["sh", "config", "validate"])
        # Expected: 通过验证
        pass

    @pytest.mark.cli
    def test_validate_invalid_config(self, temp_working_dir):
        """测试验证无效配置"""
        # 写入无效配置
        # subprocess.run(["sh", "config", "validate"])
        # Expected: 显示错误
        pass

    @pytest.mark.cli
    def test_validate_missing_api_key(self):
        """测试验证缺少 API key"""
        # Expected: 警告
        pass


pytestmark = pytest.mark.cli