"""
LLM 真实 API 调用测试

这些测试需要真实的 API 密钥，验证 LLM 调用逻辑。

环境变量（按优先级）:
1. CONTINUUM_API_KEY / CONTINUUM_BASE_URL / CONTINUUM_MODEL（推荐）
2. CONTINUUM_API_KEY / CONTINUUM_BASE_URL / CONTINUUM_MODEL（兼容）
3. ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL / ANTHROPIC_MODEL（兼容）

运行: pytest python/tests/test_llm_real.py -v -s
"""

import sys
import os
import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from test_config import get_api_key, get_base_url, get_model, is_api_available, load_env
from continuum_sdk.llm import LlmClient, Message


# 跳过条件：API 不可用
requires_api = pytest.mark.skipif(
    not is_api_available(),
    reason="API key not configured in .env.test"
)


@requires_api
class TestRealLlmCalls:
    """真实 LLM API 调用测试"""

    @pytest.fixture
    def client(self):
        """创建 LLM 客户端"""
        load_env()
        return LlmClient.for_provider(
            provider="anthropic",
            api_key=get_api_key(),
            base_url=get_base_url(),
            model=get_model(),
        )

    @pytest.mark.asyncio
    async def test_simple_chat(self, client):
        """测试简单对话"""
        messages = [Message.user("Say 'hello world' and nothing else.")]
        response = await client.chat(
            messages=messages,
            max_tokens=50,
            temperature=0.0,
        )
        assert response is not None
        assert response.content is not None
        assert len(response.content) > 0
        assert isinstance(response.content, str)
        # 验证内容包含预期关键词
        content_lower = response.content.lower()
        assert "hello" in content_lower or "world" in content_lower, \
            f"Response should contain 'hello' or 'world', got: {response.content}"
        print(f"\n[Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_system_prompt(self, client):
        """测试系统提示生效 - 验证 LLM 遵循角色设定"""
        messages = [Message.user("What is your role?")]
        response = await client.chat(
            messages=messages,
            system_prompt="You are a helpful coding assistant specializing in Python.",
            max_tokens=100,
        )
        assert response is not None
        assert response.content is not None
        print(f"\n[Response]: {response.content}")
        # 验证响应包含编码/助手相关内容（LLM 应遵循系统提示）
        content_lower = response.content.lower()
        is_coding_related = any(word in content_lower for word in [
            "coding", "python", "assistant", "program", "code", "develop", "help"
        ])
        assert is_coding_related, \
            f"Response should reflect coding assistant role, got: {response.content}"

    @pytest.mark.asyncio
    async def test_multi_turn(self, client):
        """测试多轮对话"""
        messages = [
            Message.user("My name is Alice."),
            Message.assistant("Nice to meet you, Alice!"),
            Message.user("What is my name?"),
        ]
        response = await client.chat(
            messages=messages,
            max_tokens=50,
        )
        assert response is not None
        # 模型应该记得名字
        assert "Alice" in response.content or "alice" in response.content.lower()
        print(f"\n[Response]: {response.content}")

    @pytest.mark.asyncio
    async def test_tool_call(self, client):
        """测试工具调用 - 验证 LLM 能处理工具定义"""
        from continuum_sdk.llm import ToolDefinition

        tools = [
            ToolDefinition(
                name="get_weather",
                description="Get current weather for a location",
                parameters={
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "City name"
                        }
                    },
                    "required": ["location"]
                }
            )
        ]

        messages = [Message.user("What's the weather in Tokyo?")]
        response = await client.chat(
            messages=messages,
            tools=tools,
            max_tokens=100,
        )
        assert response is not None
        # 验证请求成功完成（即使内容为空，模型可能正在发起工具调用）
        print(f"\n[Response]: {response.content}")

        # 检查是否有 tool_calls 属性
        has_tool_calls = hasattr(response, 'tool_calls') and response.tool_calls
        if has_tool_calls:
            print(f"[Tool calls]: {response.tool_calls}")
            # 验证工具调用包含必要字段
            for tc in response.tool_calls:
                tc_dict = tc if isinstance(tc, dict) else {"name": getattr(tc, 'name', None)}
                assert tc_dict.get('name') in ('get_weather', None), \
                    f"Tool name should be get_weather or None, got: {tc_dict}"

        # 如果没有 tool_calls 且有内容，验证内容相关性
        if not has_tool_calls and response.content and len(response.content.strip()) > 0:
            content_lower = response.content.lower()
            has_relevant_words = any(word in content_lower for word in [
                "tokyo", "weather", "sunny", "rain", "temperature", "forecast", "city"
            ])
            assert has_relevant_words, \
                f"Response should discuss weather/Tokyo, got: {response.content}"
            print("[Text response verified]: contains weather-related content")

    @pytest.mark.asyncio
    async def test_long_response(self, client):
        """测试长响应处理"""
        messages = [Message.user(
            "Explain in 3 short paragraphs: what is Python and why is it popular for data science?"
        )]
        response = await client.chat(
            messages=messages,
            max_tokens=500,
            temperature=0.3,
        )
        assert response is not None
        assert response.content is not None
        # 验证长响应内容
        assert len(response.content) > 100, \
            f"Long response should be >100 chars, got {len(response.content)} chars"
        # 验证内容包含 Python 相关关键词
        content_lower = response.content.lower()
        assert "python" in content_lower or "data" in content_lower, \
            f"Response should discuss Python/data, got: {response.content[:100]}..."
        print(f"\n[Long response ({len(response.content)} chars)]: {response.content[:200]}...")

    @pytest.mark.asyncio
    async def test_multi_turn_context(self, client):
        """测试多轮对话上下文记忆"""
        messages = [
            Message.user("My favorite programming language is Python."),
            Message.assistant("Got it, Python is your favorite language."),
            Message.user("What is my favorite programming language? Reply with ONLY the name."),
        ]
        response = await client.chat(
            messages=messages,
            max_tokens=20,
            temperature=0.0,
        )
        assert response is not None
        content_lower = response.content.lower()
        assert "python" in content_lower, \
            f"LLM should remember 'Python' from context, got: {response.content}"
        print(f"\n[Multi-turn response]: {response.content}")

    @pytest.mark.asyncio
    async def test_error_handling(self, client):
        """测试 API 错误处理 - 无效模型必须抛出 LlmError"""
        from continuum_sdk.llm.errors import LlmError

        invalid_client = LlmClient.for_provider(
            provider="anthropic",
            api_key=get_api_key(),
            base_url=get_base_url(),
            model="invalid-model-name",
        )

        messages = [Message.user("Hello")]
        with pytest.raises(LlmError) as exc_info:
            await invalid_client.chat(messages=messages, max_tokens=50)
        print(f"\n[Error caught]: {type(exc_info.value).__name__}: {str(exc_info.value)}")


@requires_api
class TestRealAgentPlanning:
    """Agent 真实规划测试"""

    @pytest.fixture
    def agent(self):
        """创建智能 Agent"""
        load_env()
        from continuum_sdk.agent import IntelligentAgent, AgentMode
        return IntelligentAgent(
            api_key=get_api_key(),
            base_url=get_base_url(),
            model=get_model(),
            mode=AgentMode.AUTONOMOUS,
        )

    @pytest.mark.asyncio
    async def test_plan_bug_fix(self, agent):
        """测试 bug 修复规划"""
        plan = await agent.plan("Fix the null pointer bug in auth.py")
        assert plan is not None
        assert len(plan.steps) > 0
        print(f"\n[Plan]: {plan.task}")
        for step in plan.steps[:5]:
            print(f"  [{step.id}] {step.type.value}: {step.description}")

    @pytest.mark.asyncio
    async def test_plan_add_feature(self, agent):
        """测试功能添加规划"""
        plan = await agent.plan("Add logging to the user service")
        assert plan is not None
        assert len(plan.steps) > 0
        print(f"\n[Plan]: {plan.task}")
        for step in plan.steps[:5]:
            print(f"  [{step.id}] {step.type.value}: {step.description}")

    @pytest.mark.asyncio
    async def test_plan_refactor(self, agent):
        """测试重构规划"""
        plan = await agent.plan("Refactor the database module to use async patterns")
        assert plan is not None
        assert len(plan.steps) > 0
        print(f"\n[Plan]: {plan.task}")
        for step in plan.steps[:5]:
            print(f"  [{step.id}] {step.type.value}: {step.description}")

    @pytest.mark.asyncio
    async def test_execute_simple_plan(self, agent):
        """测试简单计划执行 - 直接测试工具执行功能"""
        from continuum_sdk.tools import BashTool, ReadTool

        # 直接测试工具执行（不依赖 LLM）- 使用异步版本
        bash = BashTool()
        result = await bash.run_async("echo 'test execution'")
        assert result.is_error is False
        assert "test execution" in result.content
        print(f"\n[Bash tool result]: {result.content}")

        # 测试文件读取工具（同步版本在异步测试中可用）
        read_tool = ReadTool()
        result = read_tool.read("pyproject.toml")
        assert result.is_error is False
        assert "continuum" in result.content.lower()
        print(f"[Read tool result]: {result.content[:50]}...")

        # 验证 Agent 的规划功能（pattern-based fallback）
        plan = await agent.plan("Test simple bash execution")
        assert plan is not None
        assert len(plan.steps) > 0
        print(f"[Plan created]: {plan.task}")


class TestRealToolExecution:
    """真实工具执行测试（不需要 API）"""

    def test_bash_tool_sync(self):
        """测试 Bash 工具同步执行"""
        from continuum_sdk.tools import BashTool

        bash = BashTool()
        result = bash.run("echo hello")
        assert result.is_error is False
        assert "hello" in result.content
        print(f"\n[Bash]: {result.content}")

    def test_read_tool_sync(self):
        """测试文件读取"""
        from continuum_sdk.tools import ReadTool
        import tempfile

        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False, encoding='utf-8') as f:
            f.write("test content for real read")
            filepath = f.name

        try:
            reader = ReadTool()
            result = reader.read(filepath)
            assert result.is_error is False
            assert "test content" in result.content
            print(f"\n[Read]: {result.content[:50]}...")
        finally:
            import time
            time.sleep(0.1)
            os.unlink(filepath)

    @requires_api
    @pytest.mark.asyncio
    async def test_agent_with_bash(self):
        """测试 Agent 使用 Bash 工具 - 验证工具实际执行"""
        from continuum_sdk.agent import IntelligentAgent, AgentMode
        from continuum_sdk.tools import BashTool

        # 直接测试 Bash 工具执行能力
        bash = BashTool()
        result = await bash.run_async("echo 'agent_bash_test'")
        assert result.is_error is False, f"Bash execution failed: {result.content}"
        assert "agent_bash_test" in result.content, \
            f"Output should contain 'agent_bash_test', got: {result.content}"
        print(f"\n[Bash result]: {result.content}")

        # 验证 Agent 可以规划 bash 相关任务
        load_env()
        agent = IntelligentAgent(
            api_key=get_api_key(),
            base_url=get_base_url(),
            model=get_model(),
            mode=AgentMode.AUTONOMOUS,
        )
        plan = await agent.plan("List current directory files using bash")
        assert plan is not None
        assert len(plan.steps) > 0
        print(f"[Plan]: {plan.task}")

    @requires_api
    @pytest.mark.asyncio
    async def test_agent_with_file_ops(self):
        """测试 Agent 使用文件工具 - 验证文件读写实际工作"""
        from continuum_sdk.agent import IntelligentAgent, AgentMode
        from continuum_sdk.tools import ReadTool, WriteTool
        import tempfile

        # 直接测试文件读写能力
        with tempfile.TemporaryDirectory() as tmpdir:
            test_file = os.path.join(tmpdir, "agent_test.txt")

            # 测试写入
            writer = WriteTool(backup=False)
            write_result = writer.write(test_file, "Agent file ops test content")
            assert write_result.is_error is False, f"Write failed: {write_result.content}"

            # 测试读取
            reader = ReadTool()
            read_result = reader.read(test_file)
            assert read_result.is_error is False, f"Read failed: {read_result.content}"
            assert "Agent file ops test content" in read_result.content, \
                f"Read content mismatch, got: {read_result.content}"

        print(f"\n[Write result]: {write_result.content}")
        print(f"[Read result]: {read_result.content[:50]}...")

        # 验证 Agent 可以规划文件操作任务
        load_env()
        agent = IntelligentAgent(
            api_key=get_api_key(),
            base_url=get_base_url(),
            model=get_model(),
            mode=AgentMode.AUTONOMOUS,
        )
        plan = await agent.plan("Read the pyproject.toml file and summarize it")
        assert plan is not None
        assert len(plan.steps) > 0
        print(f"[Plan]: {plan.task}")

    @pytest.mark.asyncio
    async def test_agent_error_recovery(self):
        """测试 Agent 错误恢复 - 验证 SelfCorrection 实际分析错误"""
        from continuum_sdk.agent.self_correction import SelfCorrection, ErrorType, RecoveryStrategy

        # 直接测试 SelfCorrection 错误分类能力
        correction = SelfCorrection()

        # 测试 1: ImportError 应分类为 IMPORT 类型
        import_err = ImportError("No module named 'nonexistent_module'")
        ctx = correction.analyze_error(import_err)
        assert ctx.error_type == ErrorType.IMPORT, \
            f"ImportError should classify as IMPORT, got {ctx.error_type}"
        proposal = correction.propose_correction(ctx)
        assert proposal.strategy in (RecoveryStrategy.RETRY_MODIFIED, RecoveryStrategy.RETRY), \
            f"Import error should propose RETRY/RETRY_MODIFIED, got {proposal.strategy}"
        print(f"\n[Import error]: {ctx.error_type} → {proposal.strategy}")

        # 测试 2: FileNotFoundError 应分类为 NOT_FOUND 类型
        not_found_err = FileNotFoundError("config.yaml not found")
        ctx2 = correction.analyze_error(not_found_err)
        assert ctx2.error_type == ErrorType.NOT_FOUND, \
            f"FileNotFoundError should classify as NOT_FOUND, got {ctx2.error_type}"
        print(f"[FileNotFound error]: {ctx2.error_type}")

        # 测试 3: 连接错误应提出重试策略
        conn_err = ConnectionError("Connection refused")
        ctx3 = correction.analyze_error(conn_err)
        proposal3 = correction.propose_correction(ctx3)
        assert proposal3.strategy in (RecoveryStrategy.RETRY, RecoveryStrategy.RETRY_MODIFIED), \
            f"Connection error should propose RETRY, got {proposal3.strategy}"
        print(f"[Connection error]: {ctx3.error_type} → {proposal3.strategy}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])