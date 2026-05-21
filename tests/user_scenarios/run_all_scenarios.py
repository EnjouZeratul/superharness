"""
运行所有用户测试场景

执行所有场景并汇总结果
"""

import os
import sys
import json
import time
from datetime import datetime
from pathlib import Path

# 添加路径
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def run_scenario(scenario_file: str, verbose: bool = False) -> dict:
    """运行单个场景"""
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "scenario_module",
        scenario_file
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    # 获取场景类（查找 Scenario 前缀的类）
    scenario_class = None
    for name in dir(module):
        if name.startswith('Scenario'):
            scenario_class = getattr(module, name)
            break

    if scenario_class:
        scenario = scenario_class(verbose=verbose)
        return scenario.run()
    else:
        return {
            "status": "error",
            "error": f"No Scenario class found in {scenario_file}"
        }


def main():
    """执行所有场景"""
    import argparse

    parser = argparse.ArgumentParser(description="Run all user scenarios")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose output")
    parser.add_argument("--scenario", type=int, help="Run specific scenario (1-5)")
    args = parser.parse_args()

    scenarios_dir = os.path.dirname(os.path.abspath(__file__))
    results_dir = os.path.join(scenarios_dir, "results")
    os.makedirs(results_dir, exist_ok=True)

    # 场景文件列表
    scenario_files = [
        ("scenario_1_fizzbuzz_fix.py", "FizzBuzz Bug Fix", "T1 P1.2"),
        ("scenario_2_git_workflow.py", "Git Workflow", "T2 P1.3"),
        ("scenario_3_mcp_tools.py", "MCP Tools", "T2 P1.4"),
        ("scenario_4_session_recovery.py", "Session Recovery", "None"),
        ("scenario_5_multi_turn_context.py", "Multi-turn Context", "T1 P1.2"),
    ]

    # 如果指定了特定场景
    if args.scenario:
        if 1 <= args.scenario <= 5:
            scenario_files = [scenario_files[args.scenario - 1]]
        else:
            print(f"Invalid scenario number: {args.scenario}")
            return False

    print("="*70)
    print("CONTINUUM USER SCENARIOS TEST")
    print("="*70)
    print(f"Timestamp: {datetime.now().isoformat()}")
    print(f"Scenarios to run: {len(scenario_files)}")
    print()

    all_results = []
    summary = {
        "timestamp": datetime.now().isoformat(),
        "total_scenarios": len(scenario_files),
        "passed": 0,
        "failed": 0,
        "skipped": 0,
        "results": []
    }

    for filename, name, dependency in scenario_files:
        filepath = os.path.join(scenarios_dir, filename)

        print(f"\n[{name}] (Dependency: {dependency})")
        print("-" * 50)

        if not os.path.exists(filepath):
            print(f"  ERROR: File not found: {filepath}")
            continue

        start_time = time.time()

        try:
            result = run_scenario(filepath, verbose=args.verbose)
            elapsed = time.time() - start_time

            status = result.get("status", "unknown")
            success_rate = result.get("metrics", {}).get("success_rate", 0) * 100

            print(f"  Status: {status}")
            print(f"  Success Rate: {success_rate:.1f}%")
            print(f"  Time: {elapsed:.2f}s")

            if status == "completed":
                summary["passed"] += 1
            elif status == "skipped":
                summary["skipped"] += 1
            else:
                summary["failed"] += 1

            # 保存单个结果
            result_file = os.path.join(results_dir, filename.replace('.py', '_result.json'))
            with open(result_file, 'w') as f:
                json.dump(result, f, indent=2)

            summary["results"].append({
                "name": name,
                "status": status,
                "success_rate": success_rate,
                "elapsed": elapsed,
                "dependency": dependency
            })

        except Exception as e:
            elapsed = time.time() - start_time
            print(f"  ERROR: {str(e)}")
            summary["failed"] += 1
            summary["results"].append({
                "name": name,
                "status": "error",
                "error": str(e),
                "elapsed": elapsed
            })

    # 打印汇总
    print("\n" + "="*70)
    print("SUMMARY")
    print("="*70)
    print(f"Total: {summary['total_scenarios']}")
    print(f"Passed: {summary['passed']}")
    print(f"Failed: {summary['failed']}")
    print(f"Skipped: {summary['skipped']}")

    # 保存汇总结果
    summary_file = os.path.join(results_dir, "all_scenarios_summary.json")
    with open(summary_file, 'w') as f:
        json.dump(summary, f, indent=2)
    print(f"\nSummary saved to: {summary_file}")

    # 检查依赖状态
    print("\nDEPENDENCY STATUS:")
    for r in summary["results"]:
        dep = r.get("dependency", "Unknown")
        status = r.get("status", "unknown")
        print(f"  {r['name']}: {dep} -> {status}")

    return summary["failed"] == 0


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)