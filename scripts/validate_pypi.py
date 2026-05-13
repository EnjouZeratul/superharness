#!/usr/bin/env python3
"""SuperHarness PyPI 安装验证脚本

用户在发布后运行此脚本验证安装是否成功。
"""

import sys

def check_imports():
    """检查 SDK 导入"""
    print("=== 检查 SDK 导入 ===\n")

    try:
        from superharness_sdk import Agent
        print("✓ Agent 导入成功")
    except ImportError as e:
        print(f"✗ Agent 导入失败: {e}")
        return False

    try:
        from superharness_sdk import Session
        print("✓ Session 导入成功")
    except ImportError as e:
        print(f"✗ Session 导入失败: {e}")
        return False

    try:
        from superharness_sdk import ConfigLoader
        print("✓ ConfigLoader 导入成功")
    except ImportError as e:
        print(f"✗ ConfigLoader 导入失败: {e}")
        return False

    try:
        from superharness_sdk.tools import ToolRegistry
        print("✓ ToolRegistry 导入成功")
    except ImportError as e:
        print(f"✗ ToolRegistry 导入失败: {e}")
        return False

    try:
        from superharness_sdk.memory import Memory
        print("✓ Memory 导入成功")
    except ImportError as e:
        print(f"✗ Memory 导入失败: {e}")
        return False

    try:
        from superharness_sdk.workflow import DAG
        print("✓ DAG 导入成功")
    except ImportError as e:
        print(f"✗ DAG 导入失败: {e}")
        return False

    return True


def check_version():
    """检查版本"""
    print("\n=== 检查版本 ===\n")

    try:
        import superharness_sdk
        version = superharness_sdk.__version__
        print(f"✓ SDK 版本: {version}")
        return True
    except Exception as e:
        print(f"✗ 版本检查失败: {e}")
        return False


def check_simple_usage():
    """检查简单使用"""
    print("\n=== 检查简单使用 ===\n")

    try:
        from superharness_sdk import Agent
        agent = Agent.__new__(Agent)  # 不实际初始化，只检查类可用
        print("✓ Agent 类可用")
        return True
    except Exception as e:
        print(f"✗ Agent 类不可用: {e}")
        return False


def main():
    print("SuperHarness PyPI 安装验证\n")
    print("=" * 40)

    results = []

    results.append(("导入检查", check_imports()))
    results.append(("版本检查", check_version()))
    results.append(("使用检查", check_simple_usage()))

    print("\n" + "=" * 40)
    print("\n=== 验证结果 ===\n")

    all_pass = True
    for name, passed in results:
        status = "✓ 通过" if passed else "✗ 失败"
        print(f"{name}: {status}")
        if not passed:
            all_pass = False

    print("\n" + "=" * 40)

    if all_pass:
        print("\n🎉 PyPI 安装验证全部通过!")
        sys.exit(0)
    else:
        print("\n❌ PyPI 安装验证失败，请检查安装")
        sys.exit(1)


if __name__ == "__main__":
    main()