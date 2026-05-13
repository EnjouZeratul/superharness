# 配置流程演示

本目录展示 SuperHarness 从零配置到运行 Agent 的完整流程。

## 演示步骤

### 1. 初始化配置
```bash
sh config init
```
创建默认配置文件 `~/.sh/config.toml`

### 2. 添加自定义提供商
```bash
sh config add-provider tencent \
  --api-key $TENCENT_API_KEY \
  --base-url https://hunyuan.tencentcloudapi.com
```

### 3. 切换提供商
```bash
sh config use tencent
```

### 4. 验证配置
```bash
sh config validate
```

### 5. 显示当前配置
```bash
sh config show
```

### 6. 运行 Agent
```bash
sh run "你好"
```

## 运行演示

```bash
./demo.sh
```

## 配置文件示例

```toml
# ~/.sh/config.toml

model = "claude-3-haiku"
max_tokens = 4096
default_provider = "anthropic"

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com"

[providers.tencent]
api_key = "${TENCENT_API_KEY}"
base_url = "https://hunyuan.tencentcloudapi.com"
model = "hunyuan-lite"
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GEMINI_API_KEY` | Google Gemini API key |
| `SH_API_KEY` | 通用 API key (低优先级) |
| `SH_MODEL` | 默认模型 |
| `SH_BASE_URL` | 默认 base URL |

## 优先级

配置源优先级（从高到低）：
1. 环境变量 (`SH_*`)
2. 项目配置 (`.sh/config.toml`)
3. 全局配置 (`~/.sh/config.toml`)
4. 内置默认值