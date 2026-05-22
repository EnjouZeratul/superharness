# T3 任务：client.py 测试

## 基本信息

| 项目 | 内容 |
|------|------|
| 终端 | T3 |
| 模块 | `python/continuum_sdk/llm/client.py` |
| 当前覆盖率 | 21% |
| 目标覆盖率 | 70% |
| 优先级 | P1 |
| 参考规范 | `docs/TESTING_STANDARDS.md` |

---

## 任务目标

为 LLM 客户端模块编写 Mock 测试，不依赖真实 API，验证请求构建和响应处理逻辑。

---

## 重要提示

⚠️ **使用 Mock 测试，禁止调用真实 API**

```python
from unittest.mock import AsyncMock, MagicMock, patch
```

---

## 需测试的功能

### 1. 客户端创建
- [ ] `LlmClient.for_provider()` 工厂方法
- [ ] Anthropic 客户端创建
- [ ] OpenAI 客户端创建
- [ ] Gemini 客户端创建
- [ ] 自定义端点配置

### 2. 请求构建
- [ ] Message 构建正确
- [ ] System prompt 处理
- [ ] Tools 定义格式化
- [ ] 参数传递（max_tokens, temperature）

### 3. 响应处理
- [ ] ChatResponse 解析
- [ ] TokenUsage 统计
- [ ] StreamChunk 流式处理
- [ ] Tool call 解析

### 4. 错误处理
- [ ] 认证失败 (AuthenticationError)
- [ ] 速率限制 (RateLimitError)
- [ ] 网络超时 (TimeoutError)
- [ ] 无效响应 (InvalidResponseError)

### 5. 配置优先级
- [ ] CONTINUUM_* > CONTINUUM_* > ANTHROPIC_*
- [ ] 参数覆盖环境变量

---

## 测试要求

### 禁止行为
```python
# ❌ 不要调用真实 API
async def test_chat(self):
    client = LlmClient.for_provider("anthropic", api_key="real-key")
    response = await client.chat(...)  # 真实调用！
```

### 必须这样写
```python
# ✅ 正确示例 - 使用 Mock
import pytest
from unittest.mock import AsyncMock, patch

@pytest.mark.asyncio
async def test_chat_success(self):
    """测试成功的聊天请求"""
    mock_response = ChatResponse(
        content="Hello!",
        model="claude-sonnet-4-6",
        usage=TokenUsage(input_tokens=10, output_tokens=5)
    )
    
    client = LlmClient.for_provider("anthropic", api_key="test-key")
    
    with patch.object(client, '_make_request', new_callable=AsyncMock) as mock:
        mock.return_value = mock_response
        
        messages = [Message.user("Hi")]
        result = await client.chat(messages)
        
        assert result.content == "Hello!"
        assert result.usage.input_tokens == 10
        mock.assert_called_once()

@pytest.mark.asyncio
async def test_auth_failure_raises_error(self):
    """测试认证失败"""
    client = LlmClient.for_provider("anthropic", api_key="invalid-key")
    
    with patch.object(client, '_make_request', new_callable=AsyncMock) as mock:
        mock.side_effect = AuthenticationError("Invalid API key")
        
        with pytest.raises(AuthenticationError):
            await client.chat([Message.user("Hi")])
```

---

## 测试文件位置

```
python/tests/test_llm_client.py
```

---

## 额外任务

完成 `config/loader.py` 测试（工作量小）：
- 配置加载逻辑
- 环境变量解析
- 优先级验证

---

## 验收标准

1. 覆盖率 ≥ 70%
2. 所有 provider 类型均有测试
3. 错误处理完整
4. 无真实 API 调用
5. 无占位测试
6. CI 通过

---

## 提交方式

```bash
git checkout -b test/llm-client-coverage
git add python/tests/test_llm_client.py
git commit -m "test: add mock tests for LLM client"
git push origin test/llm-client-coverage
# 创建 PR
```
