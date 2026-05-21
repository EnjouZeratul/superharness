#!/bin/bash
# Continuum 配置流程演示
# 从零配置到运行 Agent 的完整流程

set -e

echo "=========================================="
echo "Continuum 配置流程演示"
echo "=========================================="
echo ""

# 步骤 1: 初始化配置
echo "步骤 1: 初始化配置"
echo "----------------------------------------"
echo "$ sh config init"
echo ""

# 模拟输出
cat << 'EOF'
✓ 配置文件已创建: ~/.sh/config.toml

默认配置:
  - 提供商: anthropic
  - 模型: claude-3-haiku
  - max_tokens: 4096

请设置 API key:
  export ANTHROPIC_API_KEY=your-key-here
EOF

echo ""
echo ""

# 步骤 2: 添加自定义提供商
echo "步骤 2: 添加自定义提供商 (腾讯云示例)"
echo "----------------------------------------"
echo "$ sh config add-provider tencent --api-key \$TENCENT_KEY --base-url https://hunyuan.tencentcloudapi.com"
echo ""

cat << 'EOF'
✓ 提供商已添加: tencent

配置内容:
  [providers.tencent]
  api_key = "${TENCENT_API_KEY}"
  base_url = "https://hunyuan.tencentcloudapi.com"
  model = "hunyuan-lite"
EOF

echo ""
echo ""

# 步骤 3: 切换提供商
echo "步骤 3: 切换到自定义提供商"
echo "----------------------------------------"
echo "$ sh config use tencent"
echo ""

cat << 'EOF'
✓ 当前提供商: tencent

使用配置:
  提供商: tencent
  模型: hunyuan-lite
  API Key: 从环境变量加载
EOF

echo ""
echo ""

# 步骤 4: 验证配置
echo "步骤 4: 验证配置"
echo "----------------------------------------"
echo "$ sh config validate"
echo ""

cat << 'EOF'
✓ 配置验证通过

检查项:
  ✓ 配置文件存在
  ✓ TOML 语法正确
  ✓ API key 已设置 (环境变量)
  ✓ Base URL 可访问
EOF

echo ""
echo ""

# 步骤 5: 显示当前配置
echo "步骤 5: 显示当前配置"
echo "----------------------------------------"
echo "$ sh config show"
echo ""

cat << 'EOF'
当前配置:
  提供商: tencent
  模型: hunyuan-lite
  max_tokens: 4096

提供商配置:
  api_key: ${TENCENT_API_KEY}
  base_url: https://hunyuan.tencentcloudapi.com
EOF

echo ""
echo ""

# 步骤 6: 运行 Agent
echo "步骤 6: 运行 Agent"
echo "----------------------------------------"
echo '$ sh run "你好，请介绍一下你自己"'
echo ""

cat << 'EOF'
Agent: 你好！我是 Continuum Agent，基于腾讯云混元大模型。
我可以帮助你：
- 读取和编辑文件
- 执行命令
- 搜索代码
- 管理会话

有什么可以帮你的吗？
EOF

echo ""
echo ""

# 完成
echo "=========================================="
echo "演示完成！"
echo "=========================================="
echo ""
echo "后续步骤:"
echo "  1. 查看配置: sh config show"
echo "  2. 添加更多提供商: sh config add-provider --help"
echo "  3. 开始对话: sh run"
echo ""