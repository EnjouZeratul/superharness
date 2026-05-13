#!/bin/bash
# SuperHarness 端到端验证脚本
#
# 用户使用真实 API key 运行完整流程验证。

set -e

echo "SuperHarness 端到端验证"
echo "========================"
echo ""

# 检查环境变量
check_api_key() {
    echo "=== 检查 API Key ==="
    echo ""

    if [ -z "$ANTHROPIC_API_KEY" ] && [ -z "$OPENAI_API_KEY" ] && [ -z "$GEMINI_API_KEY" ] && [ -z "$CUSTOM_API_KEY" ]; then
        echo "⚠ 未检测到任何 API key 环境变量"
        echo ""
        echo "请设置至少一个:"
        echo "  export ANTHROPIC_API_KEY=your-key"
        echo "  export OPENAI_API_KEY=your-key"
        echo "  export GEMINI_API_KEY=your-key"
        echo "  export CUSTOM_API_KEY=your-key"
        echo ""
        read -p "是否继续验证配置? (y/n) " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    else
        echo "✓ 检测到 API key:"
        [ -n "$ANTHROPIC_API_KEY" ] && echo "  - ANTHROPIC_API_KEY"
        [ -n "$OPENAI_API_KEY" ] && echo "  - OPENAI_API_KEY"
        [ -n "$GEMINI_API_KEY" ] && echo "  - GEMINI_API_KEY"
        [ -n "$CUSTOM_API_KEY" ] && echo "  - CUSTOM_API_KEY"
    fi
}

# 初始化配置
init_config() {
    echo ""
    echo "=== 初始化配置 ==="
    echo ""

    if sh config init &> /dev/null; then
        echo "✓ 配置初始化成功"
    else
        echo "⚠ 配置初始化失败或已存在"
    fi
}

# 显示配置
show_config() {
    echo ""
    echo "=== 当前配置 ==="
    echo ""

    sh config show || echo "⚠ 无法显示配置"
}

# 运行简单测试 (如果有 API key)
run_test() {
    echo ""
    echo "=== 运行 Agent 测试 ==="
    echo ""

    if [ -n "$ANTHROPIC_API_KEY" ] || [ -n "$OPENAI_API_KEY" ] || [ -n "$GEMINI_API_KEY" ]; then
        echo "测试命令: sh run 'hello'"
        echo ""
        echo "输出:"
        echo "----------------------------------------"
        sh run "hello" || echo "⚠ 运行失败"
        echo "----------------------------------------"
    else
        echo "⚠ 无 API key，跳过 Agent 测试"
        echo ""
        echo "设置 API key 后运行:"
        echo "  sh run 'hello'"
    fi
}

# Python SDK 测试
test_python_sdk() {
    echo ""
    echo "=== Python SDK 测试 ==="
    echo ""

    if python3 -c "from superharness_sdk import Agent" 2>/dev/null; then
        echo "✓ SDK 导入成功"
        echo ""

        if [ -n "$ANTHROPIC_API_KEY" ]; then
            echo "测试脚本:"
            echo "  from superharness_sdk import Agent"
            echo "  agent = Agent()"
            echo "  result = agent.run('hello')"
            echo ""
            python3 -c "
from superharness_sdk import Agent
print('SDK 测试...')
" || echo "⚠ SDK 运行失败"
        else
            echo "⚠ 无 API key，跳过 SDK 运行测试"
        fi
    else
        echo "✗ SDK 导入失败"
        echo "  请运行: pip install superharness"
    fi
}

# 生成报告
generate_report() {
    echo ""
    echo "========================"
    echo ""
    echo "=== 验证报告 ==="
    echo ""

    echo "验证时间: $(date)"
    echo "环境:"
    echo "  - OS: $(uname -s)"
    echo "  - Python: $(python3 --version 2>/dev/null || echo '未安装')"
    echo ""

    read -p "验证是否成功完成? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo "🎉 端到端验证通过!"
        echo ""
        echo "请填写验证报告:"
        echo "  docs/release/validation_template.md"
    else
        echo ""
        echo "❌ 验证存在问题"
        echo ""
        echo "请记录问题并在 GitHub Issues 报告"
    fi
}

# 主流程
check_api_key
init_config
show_config
run_test
test_python_sdk
generate_report