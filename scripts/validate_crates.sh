#!/bin/bash
# SuperHarness crates.io 安装验证脚本
#
# 用户在发布后运行此脚本验证 CLI 安装是否成功。

set -e

echo "SuperHarness crates.io 安装验证"
echo "=================================="
echo ""

# 检查命令是否存在
check_command() {
    if command -v sh &> /dev/null; then
        echo "✓ 'sh' 命令已安装"
    else
        echo "✗ 'sh' 命令未找到"
        echo "  请运行: cargo install superharness"
        return 1
    fi
}

# 检查版本
check_version() {
    echo ""
    echo "=== 检查版本 ==="
    echo ""

    if sh --version &> /dev/null; then
        echo "✓ CLI 版本: $(sh --version)"
    else
        echo "⚠ 无法获取版本信息"
    fi
}

# 检查帮助命令
check_help() {
    echo ""
    echo "=== 检查帮助命令 ==="
    echo ""

    if sh --help &> /dev/null; then
        echo "✓ --help 命令可用"
        echo ""
        echo "可用命令:"
        sh --help | grep -E "^  [a-z]+" || true
    else
        echo "✗ --help 命令失败"
        return 1
    fi
}

# 检查子命令
check_subcommands() {
    echo ""
    echo "=== 检查子命令 ==="
    echo ""

    # config 命令
    if sh config --help &> /dev/null; then
        echo "✓ config 子命令可用"
    else
        echo "✗ config 子命令失败"
    fi

    # run 命令
    if sh run --help &> /dev/null; then
        echo "✓ run 子命令可用"
    else
        echo "✗ run 子命令失败"
    fi

    # session 命令
    if sh session --help &> /dev/null; then
        echo "✓ session 子命令可用"
    else
        echo "✗ session 子命令失败"
    fi
}

# 主流程
echo "=== 检查命令安装 ==="
echo ""

check_command || exit 1
check_version
check_help || exit 1
check_subcommands

echo ""
echo "=================================="
echo ""
echo "🎉 crates.io 安装验证通过!"
echo ""
echo "下一步:"
echo "  1. 运行: sh config init"
echo "  2. 设置: export ANTHROPIC_API_KEY=your-key"
echo "  3. 测试: sh run 'hello'"