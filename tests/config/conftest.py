"""配置测试 fixtures"""

import pytest
import tempfile
from pathlib import Path


@pytest.fixture
def temp_working_dir():
    """临时工作目录"""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def sample_toml_config(temp_working_dir):
    """示例 TOML 配置"""
    config_file = temp_working_dir / ".sh" / "config.toml"
    config_file.parent.mkdir(parents=True, exist_ok=True)
    config_file.write_text("""
# SuperHarness 配置示例

model = "claude-3-haiku"
max_tokens = 4096

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"

[providers.gemini]
api_key = "${GEMINI_API_KEY}"
base_url = "https://generativelanguage.googleapis.com/v1"

default_provider = "anthropic"
""")
    return config_file


@pytest.fixture
def sample_env_vars():
    """示例环境变量"""
    return {
        "ANTHROPIC_API_KEY": "sk-ant-test",
        "OPENAI_API_KEY": "sk-test",
        "GEMINI_API_KEY": "gemini-test",
        "SH_MODEL": "claude-3-opus",
    }


pytestmark = pytest.mark.config