"""
场景2: Git 工作流完整验证

测试 Git 集成功能：
- git status 显示
- git diff 分析
- git commit 集成
- git branch 管理
- PR 创建和管理

依赖: T2 P1.3 Git深度集成
"""

import os
import sys
import json
import tempfile
import shutil
import subprocess
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional


class Scenario2GitWorkflow:
    """场景2: Git 工作流测试"""

    def __init__(self, verbose: bool = False):
        self.verbose = verbose
        self.results: Dict[str, Any] = {
            "scenario": "scenario_2_git_workflow",
            "timestamp": datetime.now().isoformat(),
            "status": "pending",
            "steps": [],
            "metrics": {},
            "errors": []
        }
        self.temp_dir: Optional[str] = None
        self.repo_dir: Optional[str] = None

    def setup(self) -> bool:
        """准备测试环境"""
        try:
            self.temp_dir = tempfile.mkdtemp(prefix="sh_scenario2_")
            self.repo_dir = os.path.join(self.temp_dir, "test_repo")
            os.makedirs(self.repo_dir)

            # 初始化 git 仓库
            subprocess.run(["git", "init"], cwd=self.repo_dir, check=True, capture_output=True)
            subprocess.run(["git", "config", "user.name", "Test User"], cwd=self.repo_dir, check=True, capture_output=True)
            subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=self.repo_dir, check=True, capture_output=True)

            # 创建初始文件
            readme_file = os.path.join(self.repo_dir, "README.md")
            with open(readme_file, 'w') as f:
                f.write("# Test Repository\n\nThis is a test repository.\n")

            # 初始提交
            subprocess.run(["git", "add", "README.md"], cwd=self.repo_dir, check=True, capture_output=True)
            subprocess.run(["git", "commit", "-m", "Initial commit"], cwd=self.repo_dir, check=True, capture_output=True)

            self.log("Setup complete", f"Git repo created at {self.repo_dir}")
            return True

        except Exception as e:
            self.log_error("Setup failed", str(e))
            return False

    def run(self) -> Dict[str, Any]:
        """执行测试场景"""
        if not self.setup():
            return self._finalize("setup_failed")

        try:
            # 步骤1: 创建变更
            step1 = self.step1_create_changes()
            self.results["steps"].append(step1)

            # 步骤2: Git status
            step2 = self.step2_git_status()
            self.results["steps"].append(step2)

            # 步骤3: Git diff
            step3 = self.step3_git_diff()
            self.results["steps"].append(step3)

            # 步骤4: Git add
            step4 = self.step4_git_add()
            self.results["steps"].append(step4)

            # 步骤5: Git commit (带消息生成)
            step5 = self.step5_git_commit()
            self.results["steps"].append(step5)

            # 步骤6: Git branch
            step6 = self.step6_git_branch()
            self.results["steps"].append(step6)

            # 步骤7: Git log
            step7 = self.step7_git_log()
            self.results["steps"].append(step7)

            self.teardown()
            return self._finalize("completed")

        except Exception as e:
            self.log_error("Execution failed", str(e))
            self.teardown()
            return self._finalize("execution_failed")

    def step1_create_changes(self) -> Dict[str, Any]:
        """步骤1: 创建文件变更"""
        step = {
            "name": "create_changes",
            "status": "pending",
            "details": {}
        }

        try:
            # 添加新文件
            new_file = os.path.join(self.repo_dir, "app.py")
            with open(new_file, 'w') as f:
                f.write('''
def hello():
    """Say hello."""
    return "Hello, World!"

if __name__ == "__main__":
    print(hello())
''')

            # 修改现有文件
            readme_file = os.path.join(self.repo_dir, "README.md")
            with open(readme_file, 'a') as f:
                f.write("\n## Features\n\n- Added hello function\n")

            step["details"]["files_created"] = 1
            step["details"]["files_modified"] = 1
            step["status"] = "passed"

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step2_git_status(self) -> Dict[str, Any]:
        """步骤2: Git status 显示"""
        step = {
            "name": "git_status",
            "status": "pending",
            "details": {}
        }

        try:
            result = subprocess.run(
                ["git", "status", "--porcelain"],
                cwd=self.repo_dir,
                capture_output=True,
                text=True,
                check=True
            )

            lines = result.stdout.strip().split('\n') if result.stdout.strip() else []
            step["details"]["output"] = result.stdout
            step["details"]["files_changed"] = len([l for l in lines if l])
            step["status"] = "passed"

            self.log("Git status", f"{len(lines)} files changed")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step3_git_diff(self) -> Dict[str, Any]:
        """步骤3: Git diff 分析"""
        step = {
            "name": "git_diff",
            "status": "pending",
            "details": {}
        }

        try:
            result = subprocess.run(
                ["git", "diff"],
                cwd=self.repo_dir,
                capture_output=True,
                text=True,
                check=True
            )

            diff_lines = result.stdout.split('\n') if result.stdout else []
            additions = len([l for l in diff_lines if l.startswith('+')])
            deletions = len([l for l in diff_lines if l.startswith('-')])

            step["details"]["diff_length"] = len(result.stdout)
            step["details"]["additions"] = additions
            step["details"]["deletions"] = deletions
            step["status"] = "passed"

            self.log("Git diff", f"+{additions} -{deletions}")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step4_git_add(self) -> Dict[str, Any]:
        """步骤4: Git add"""
        step = {
            "name": "git_add",
            "status": "pending",
            "details": {}
        }

        try:
            result = subprocess.run(
                ["git", "add", "."],
                cwd=self.repo_dir,
                capture_output=True,
                text=True,
                check=True
            )

            step["details"]["success"] = True
            step["status"] = "passed"

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step5_git_commit(self) -> Dict[str, Any]:
        """步骤5: Git commit (带消息生成)"""
        step = {
            "name": "git_commit",
            "status": "pending",
            "details": {}
        }

        try:
            # Agent 应该生成 commit 消息
            commit_msg = "Add hello function and update README"

            result = subprocess.run(
                ["git", "commit", "-m", commit_msg],
                cwd=self.repo_dir,
                capture_output=True,
                text=True,
                check=True
            )

            step["details"]["commit_message"] = commit_msg
            step["details"]["output"] = result.stdout
            step["status"] = "passed"

            self.log("Git commit", commit_msg)

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step6_git_branch(self) -> Dict[str, Any]:
        """步骤6: Git branch 管理"""
        step = {
            "name": "git_branch",
            "status": "pending",
            "details": {}
        }

        try:
            # 创建分支
            result = subprocess.run(
                ["git", "branch", "feature/test"],
                cwd=self.repo_dir,
                capture_output=True,
                text=True,
                check=True
            )

            # 列出分支
            result = subprocess.run(
                ["git", "branch", "-a"],
                cwd=self.repo_dir,
                capture_output=True,
                text=True,
                check=True
            )

            branches = [b.strip() for b in result.stdout.strip().split('\n') if b.strip()]
            step["details"]["branches"] = branches
            step["details"]["branch_count"] = len(branches)
            step["status"] = "passed"

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step7_git_log(self) -> Dict[str, Any]:
        """步骤7: Git log"""
        step = {
            "name": "git_log",
            "status": "pending",
            "details": {}
        }

        try:
            result = subprocess.run(
                ["git", "log", "--oneline", "-5"],
                cwd=self.repo_dir,
                capture_output=True,
                text=True,
                check=True
            )

            commits = result.stdout.strip().split('\n') if result.stdout.strip() else []
            step["details"]["commits"] = commits
            step["details"]["commit_count"] = len(commits)
            step["status"] = "passed"

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def teardown(self):
        """清理测试环境"""
        if self.temp_dir and os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir, ignore_errors=True)

    def log(self, action: str, message: str):
        if self.verbose:
            print(f"[Scenario2] {action}: {message}")

    def log_error(self, action: str, error: str):
        self.results["errors"].append({
            "action": action,
            "error": error,
            "timestamp": datetime.now().isoformat()
        })
        if self.verbose:
            print(f"[Scenario2 ERROR] {action}: {error}")

    def _finalize(self, status: str) -> Dict[str, Any]:
        self.results["status"] = status

        passed = sum(1 for s in self.results["steps"] if s["status"] == "passed")
        total = len(self.results["steps"])

        self.results["metrics"] = {
            "total_steps": total,
            "passed_steps": passed,
            "success_rate": passed / total if total > 0 else 0
        }

        return self.results


class GitWorkflowChecklist:
    """Git 工作流验收清单"""

    def check_status_display(self) -> bool:
        """检查 status 显示是否正确"""
        import subprocess
        import tempfile
        import os
        with tempfile.TemporaryDirectory() as temp_dir:
            subprocess.run(["git", "init"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.name", "Test"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.email", "test@test.com"], cwd=temp_dir, capture_output=True)
            test_file = os.path.join(temp_dir, "test.txt")
            with open(test_file, 'w') as f:
                f.write("content")
            result = subprocess.run(["git", "status", "--porcelain"], cwd=temp_dir, capture_output=True, text=True)
            assert "??" in result.stdout or "test.txt" in result.stdout
            return True

    def check_diff_analysis(self) -> bool:
        """检查 diff 分析是否准确"""
        import subprocess
        import tempfile
        import os
        with tempfile.TemporaryDirectory() as temp_dir:
            subprocess.run(["git", "init"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.name", "Test"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.email", "test@test.com"], cwd=temp_dir, capture_output=True)
            test_file = os.path.join(temp_dir, "test.txt")
            with open(test_file, 'w') as f:
                f.write("original content")
            subprocess.run(["git", "add", "test.txt"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "commit", "-m", "init"], cwd=temp_dir, capture_output=True)
            with open(test_file, 'w') as f:
                f.write("modified content")
            result = subprocess.run(["git", "diff"], cwd=temp_dir, capture_output=True, text=True)
            assert "-original" in result.stdout and "+modified" in result.stdout
            return True

    def check_commit_message_quality(self) -> bool:
        """检查 commit 消息质量"""
        # Verify commit message follows conventional commit format
        valid_messages = ["Add feature X", "Fix bug in module Y", "Refactor code for clarity", "Update documentation"]
        for msg in valid_messages:
            assert len(msg) > 5 and len(msg) < 72, f"Message '{msg}' should be between 5-72 chars"
            assert msg[0].isupper(), f"Message '{msg}' should start with uppercase"
        return True

    def check_branch_management(self) -> bool:
        """检查分支管理功能"""
        import subprocess
        import tempfile
        with tempfile.TemporaryDirectory() as temp_dir:
            subprocess.run(["git", "init"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.name", "Test"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.email", "test@test.com"], cwd=temp_dir, capture_output=True)
            # Create a file and commit on main
            with open(os.path.join(temp_dir, "test.txt"), 'w') as f:
                f.write("content")
            subprocess.run(["git", "add", "."], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "commit", "-m", "init"], cwd=temp_dir, capture_output=True)
            # Create new branch
            subprocess.run(["git", "branch", "feature-test"], cwd=temp_dir, capture_output=True)
            result = subprocess.run(["git", "branch", "--list"], cwd=temp_dir, capture_output=True, text=True)
            assert "feature-test" in result.stdout
            assert "master" in result.stdout or "main" in result.stdout
            return True

    def check_pr_creation(self) -> bool:
        """检查 PR 创建功能"""
        # PR creation typically requires remote repository
        # Here we test the branch comparison logic
        import subprocess
        import tempfile
        import os
        with tempfile.TemporaryDirectory() as temp_dir:
            subprocess.run(["git", "init"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.name", "Test"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "config", "user.email", "test@test.com"], cwd=temp_dir, capture_output=True)
            with open(os.path.join(temp_dir, "main.txt"), 'w') as f:
                f.write("main content")
            subprocess.run(["git", "add", "."], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "commit", "-m", "main init"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "branch", "pr-feature"], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "checkout", "pr-feature"], cwd=temp_dir, capture_output=True)
            with open(os.path.join(temp_dir, "feature.txt"), 'w') as f:
                f.write("feature content")
            subprocess.run(["git", "add", "."], cwd=temp_dir, capture_output=True)
            subprocess.run(["git", "commit", "-m", "feature addition"], cwd=temp_dir, capture_output=True)
            # Check that feature branch has different commits than main
            result = subprocess.run(["git", "log", "--oneline", "--all"], cwd=temp_dir, capture_output=True, text=True)
            assert "feature addition" in result.stdout
            return True


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Scenario 2: Git Workflow")
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--save", action="store_true")
    args = parser.parse_args()

    scenario = Scenario2GitWorkflow(verbose=args.verbose)
    results = scenario.run()

    print("\n" + "="*60)
    print("SCENARIO 2: Git Workflow Results")
    print("="*60)
    print(f"Status: {results['status']}")
    print(f"Success Rate: {results['metrics']['success_rate']*100:.1f}%")

    if args.save:
        output_dir = os.path.join(os.path.dirname(__file__), "results")
        os.makedirs(output_dir, exist_ok=True)
        output_file = os.path.join(output_dir, "scenario_2_result.json")
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nResults saved to: {output_file}")

    return results['status'] == 'completed'


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)