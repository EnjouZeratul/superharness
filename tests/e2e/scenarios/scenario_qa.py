"""E2E 场景: 简单问答流程

测试最基本的使用场景：用户提问，Agent 回答。

场景描述:
1. 用户启动 Agent
2. 用户发送问题
3. Agent 返回答案
4. 用户结束会话

预期行为:
- 无工具调用
- 单次对话
- 响应时间 < 5s
"""

import pytest
from unittest.mock import Mock, patch
import asyncio


class ScenarioQA:
    """简单问答场景"""

    # 场景配置
    PROMPTS = [
        "你好，介绍一下你自己",
        "什么是 Python？",
        "今天天气怎么样？",
    ]

    EXPECTED_RESPONSES = {
        "你好，介绍一下你自己": "包含自我介绍的响应",
        "什么是 Python？": "包含 Python 解释的响应",
        "今天天气怎么样？": "无法获取天气或说明需要工具",
    }

    async def run(self, agent):
        """运行场景"""
        results = []
        for prompt in self.PROMPTS:
            response = await agent.chat(prompt)
            results.append({
                "prompt": prompt,
                "response": response,
                "expected": self.EXPECTED_RESPONSES.get(prompt),
            })
        return results

    def validate(self, results):
        """验证结果"""
        for result in results:
            # 检查响应非空
            assert result["response"], "响应不应为空"
            # 检查响应相关（人工验证）
            # TODO: 实现自动相关性检查
        return True


class TestScenarioQA:
    """简单问答场景测试"""

    @pytest.mark.e2e
    async def test_basic_qa(self):
        """测试基本问答"""
        scenario = ScenarioQA()
        # Mock agent 或使用真实 agent
        # agent = Agent(...)
        # results = await scenario.run(agent)
        # assert scenario.validate(results)
        pass

    @pytest.mark.e2e
    async def test_qa_with_chinese(self):
        """测试中文问答"""
        prompt = "用中文解释什么是 API"
        # Expected: 中文响应
        pass

    @pytest.mark.e2e
    async def test_qa_with_english(self):
        """测试英文问答"""
        prompt = "Explain what is REST API"
        # Expected: 英文响应
        pass

    @pytest.mark.e2e
    async def test_qa_response_time(self):
        """测试响应时间"""
        # start = time.time()
        # response = await agent.chat("hello")
        # elapsed = time.time() - start
        # assert elapsed < 5.0  # 5秒内
        pass


# ==================== 运行标记 ====================

pytestmark = pytest.mark.e2e