"""真实 API 调用验证 - 自定义端点

测试自定义 API 端点（如腾讯云、阿里云等）。
"""

import pytest
import os


class TestCustomAPI:
    """自定义 API 端点测试"""

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("CUSTOM_API_KEY"),
        reason="CUSTOM_API_KEY not set"
    )
    async def test_custom_endpoint_call(self):
        """测试自定义端点调用"""
        # agent = Agent(
        #     provider="custom",
        #     base_url=os.environ.get("CUSTOM_BASE_URL"),
        #     api_key=os.environ.get("CUSTOM_API_KEY")
        # )
        # response = await agent.chat("Hello")
        # Expected: 正确调用自定义端点
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("TENCENT_API_KEY"),
        reason="TENCENT_API_KEY not set"
    )
    async def test_tencent_cloud_api(self):
        """测试腾讯云 API"""
        # agent = Agent(
        #     provider="custom",
        #     base_url="https://api.tencentcloud.com/...",
        #     api_key=os.environ.get("TENCENT_API_KEY")
        # )
        pass

    @pytest.mark.api
    @pytest.mark.skipif(
        not os.environ.get("ALIBABA_API_KEY"),
        reason="ALIBABA_API_KEY not set"
    )
    async def test_alibaba_cloud_api(self):
        """测试阿里云 API"""
        # agent = Agent(
        #     provider="custom",
        #     base_url="https://dashscope.aliyuncs.com/...",
        #     api_key=os.environ.get("ALIBABA_API_KEY")
        # )
        pass

    @pytest.mark.api
    async def test_custom_endpoint_with_auth_header(self):
        """测试自定义认证头"""
        # 部分自定义端点使用不同的认证方式
        # headers = {"X-Custom-Auth": "token"}
        pass

    @pytest.mark.api
    async def test_custom_endpoint_response_format(self):
        """测试自定义响应格式"""
        # 不同端点可能有不同的响应结构
        # Expected: 正确解析响应
        pass

    @pytest.mark.api
    def test_custom_endpoint_connection_error(self):
        """测试连接错误"""
        # base_url = "https://nonexistent.endpoint"
        # Expected: 报错
        pass


pytestmark = pytest.mark.api