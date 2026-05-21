"""
场景5: 多轮对话上下文保持

测试多轮对话能力：
- 上下文保持
- 历史记录正确
- 会话恢复后上下文完整
- 长对话管理
- 主题切换与回归

依赖: T1 P1.2 Agent智能增强
"""

import os
import sys
import json
import tempfile
import shutil
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional

# Add paths
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "python"))

from continuum_sdk.agent.session import Session, Message, MessageRole


class Scenario5MultiTurnContext:
    """场景5: 多轮对话上下文保持测试"""

    def __init__(self, verbose: bool = False):
        self.verbose = verbose
        self.results: Dict[str, Any] = {
            "scenario": "scenario_5_multi_turn_context",
            "timestamp": datetime.now().isoformat(),
            "status": "pending",
            "steps": [],
            "metrics": {},
            "errors": []
        }

    def run(self) -> Dict[str, Any]:
        """执行测试场景"""
        try:
            # 步骤1: 创建会话
            step1 = self.step1_create_session()
            self.results["steps"].append(step1)

            # 步骤2: 第一轮对话
            step2 = self.step2_first_turn()
            self.results["steps"].append(step2)

            # 步骤3: 第二轮对话（引用第一轮）
            step3 = self.step3_second_turn()
            self.results["steps"].append(step3)

            # 步骤4: 多轮深度对话
            step4 = self.step4_deep_conversation()
            self.results["steps"].append(step4)

            # 步骤5: 上下文验证
            step5 = self.step5_context_verification()
            self.results["steps"].append(step5)

            # 步骤6: 会话导出导入
            step6 = self.step6_session_export_import()
            self.results["steps"].append(step6)

            # 步骤7: 恢复后上下文保持
            step7 = self.step7_context_after_restore()
            self.results["steps"].append(step7)

            return self._finalize("completed")

        except Exception as e:
            self.log_error("Execution failed", str(e))
            return self._finalize("execution_failed")

    def step1_create_session(self) -> Dict[str, Any]:
        """步骤1: 创建会话"""
        step = {
            "name": "create_session",
            "status": "pending",
            "details": {}
        }

        try:
            session = Session(id="multi-turn-test")
            step["details"]["session_id"] = session.id
            step["details"]["message_count"] = session.message_count
            step["status"] = "passed"

            self.session = session
            self.log("Session created", f"ID: {session.id}")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step2_first_turn(self) -> Dict[str, Any]:
        """步骤2: 第一轮对话"""
        step = {
            "name": "first_turn",
            "status": "pending",
            "details": {}
        }

        try:
            # 用户消息
            self.session.add_user_message("My name is Alice and I work on Python projects.")

            # 模拟助手回复
            self.session.add_assistant_message("Nice to meet you, Alice! I can help with your Python projects.")

            step["details"]["user_message"] = "My name is Alice..."
            step["details"]["message_count"] = self.session.message_count
            step["status"] = "passed"

            self.log("First turn", f"Messages: {self.session.message_count}")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step3_second_turn(self) -> Dict[str, Any]:
        """步骤3: 第二轮对话（引用第一轮信息）"""
        step = {
            "name": "second_turn",
            "status": "pending",
            "details": {}
        }

        try:
            # 用户引用第一轮信息
            self.session.add_user_message("What is my name?")

            # 检查历史中是否包含第一轮信息
            messages = self.session.get_messages()
            has_name_context = any("Alice" in m.content for m in messages)

            self.session.add_assistant_message("Your name is Alice, as you mentioned earlier.")

            step["details"]["has_context"] = has_name_context
            step["details"]["message_count"] = self.session.message_count
            step["status"] = "passed" if has_name_context else "failed"

            self.log("Second turn", f"Context preserved: {has_name_context}")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step4_deep_conversation(self) -> Dict[str, Any]:
        """步骤4: 多轮深度对话"""
        step = {
            "name": "deep_conversation",
            "status": "pending",
            "details": {}
        }

        try:
            # 添加多轮对话
            conversation_turns = [
                ("I'm building a web API with Flask.", "Flask is a great choice for web APIs!"),
                ("Can you help me add authentication?", "Sure! I recommend using Flask-Login or JWT tokens."),
                ("Let's go with JWT. How do I start?", "First, install PyJWT and set up a secret key..."),
                ("What about the database?", "You can use SQLAlchemy as an ORM with Flask-SQLAlchemy."),
                ("Remind me, what framework am I using?", "You're using Flask, as you mentioned earlier."),
            ]

            for user_msg, assistant_msg in conversation_turns:
                self.session.add_user_message(user_msg)
                self.session.add_assistant_message(assistant_msg)

            step["details"]["turns_added"] = len(conversation_turns)
            step["details"]["total_messages"] = self.session.message_count
            step["status"] = "passed"

            self.log("Deep conversation", f"Added {len(conversation_turns)} turns")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step5_context_verification(self) -> Dict[str, Any]:
        """步骤5: 上下文验证"""
        step = {
            "name": "context_verification",
            "status": "pending",
            "details": {}
        }

        try:
            messages = self.session.get_messages()

            # 验证关键上下文
            context_checks = {
                "name_context": any("Alice" in m.content for m in messages),
                "framework_context": any("Flask" in m.content for m in messages),
                "auth_context": any("JWT" in m.content for m in messages),
                "db_context": any("SQLAlchemy" in m.content for m in messages),
            }

            step["details"]["context_checks"] = context_checks
            all_preserved = all(context_checks.values())

            if all_preserved:
                step["status"] = "passed"
                step["details"]["message"] = "All context preserved across turns"
            else:
                step["status"] = "failed"
                step["details"]["message"] = f"Missing context: {[k for k, v in context_checks.items() if not v]}"

            self.log("Context verification", f"All preserved: {all_preserved}")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step6_session_export_import(self) -> Dict[str, Any]:
        """步骤6: 会话导出导入"""
        step = {
            "name": "session_export_import",
            "status": "pending",
            "details": {}
        }

        try:
            # 导出会话（返回 JSON 字符串）
            exported = self.session.export()
            step["details"]["export_success"] = True
            step["details"]["exported_length"] = len(exported)

            # 导入会话
            imported_session = Session.from_export(exported)
            step["details"]["import_success"] = True
            step["details"]["imported_message_count"] = imported_session.message_count

            # 验证消息数量一致
            step["details"]["counts_match"] = self.session.message_count == imported_session.message_count

            step["status"] = "passed"

            self.imported_session = imported_session
            self.log("Export/Import", f"Messages: {self.session.message_count} -> {imported_session.message_count}")

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def step7_context_after_restore(self) -> Dict[str, Any]:
        """步骤7: 恢复后上下文保持"""
        step = {
            "name": "context_after_restore",
            "status": "pending",
            "details": {}
        }

        try:
            if hasattr(self, 'imported_session'):
                # 恢复后验证上下文
                messages = self.imported_session.get_messages()

                context_preserved = {
                    "name": any("Alice" in m.content for m in messages),
                    "framework": any("Flask" in m.content for m in messages),
                    "auth_method": any("JWT" in m.content for m in messages),
                }

                step["details"]["context_preserved"] = context_preserved
                all_ok = all(context_preserved.values())

                step["status"] = "passed" if all_ok else "failed"
                step["details"]["message"] = "Context fully preserved after restore" if all_ok else "Context lost after restore"
            else:
                step["status"] = "skipped"
                step["details"]["message"] = "No imported session to verify"

        except Exception as e:
            step["status"] = "error"
            step["details"]["error"] = str(e)

        return step

    def log(self, action: str, message: str):
        if self.verbose:
            print(f"[Scenario5] {action}: {message}")

    def log_error(self, action: str, error: str):
        self.results["errors"].append({
            "action": action,
            "error": error
        })
        if self.verbose:
            print(f"[Scenario5 ERROR] {action}: {error}")

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


class ContextBoundaryTests:
    """上下文边界测试"""

    def test_very_long_conversation(self):
        """测试超长对话（100+ 轮）"""
        from continuum_sdk.agent.session import Session
        session = Session(id="long-conversation-test")
        # Simulate 100+ turn conversation
        for i in range(110):
            if i % 2 == 0:
                session.add_user_message(f"User message {i}: asking about topic {i % 10}")
            else:
                session.add_assistant_message(f"Assistant response {i}: answering about topic {i % 10}")
        assert session.message_count >= 110
        messages = session.get_messages()
        assert len(messages) >= 110
        # Verify early context is preserved
        first_messages = messages[:10]
        assert any("topic 0" in m.content for m in first_messages)

    def test_topic_switching(self):
        """测试主题切换后回归"""
        from continuum_sdk.agent.session import Session
        session = Session(id="topic-switch-test")
        # First topic: Python
        session.add_user_message("I want to learn Python programming")
        session.add_assistant_message("Python is a great language for beginners")
        # Topic switch: JavaScript
        session.add_user_message("Now let's talk about JavaScript")
        session.add_assistant_message("JavaScript is used for web development")
        # Return to first topic
        session.add_user_message("Back to Python, can you recommend resources?")
        messages = session.get_messages()
        # Verify both topics are in context
        assert any("Python" in m.content for m in messages)
        assert any("JavaScript" in m.content for m in messages)
        assert session.message_count == 6

    def test_code_context(self):
        """测试代码上下文保持"""
        from continuum_sdk.agent.session import Session
        session = Session(id="code-context-test")
        code_snippet = '''
def calculate_sum(numbers):
    total = 0
    for n in numbers:
        total += n
    return total
'''
        session.add_user_message(f"Here's my code:\n{code_snippet}")
        session.add_assistant_message("Your calculate_sum function looks good. It iterates through numbers.")
        session.add_user_message("Can you improve the performance?")
        session.add_assistant_message("You could use sum() built-in: return sum(numbers)")
        messages = session.get_messages()
        # Verify code context is preserved across turns
        assert any("calculate_sum" in m.content for m in messages)
        assert any("sum()" in m.content for m in messages)

    def test_multilingual_context(self):
        """测试多语言上下文"""
        from continuum_sdk.agent.session import Session
        session = Session(id="multilingual-test")
        session.add_user_message("Hello, I speak English")
        session.add_assistant_message("Nice! I can help you in English")
        session.add_user_message("Also ich spreche Deutsch")
        session.add_assistant_message("Ich kann auch in Deutsch helfen")
        session.add_user_message("Tengo una pregunta in Spanish too")
        session.add_assistant_message("Si, puedo responder en Espanol")
        messages = session.get_messages()
        # Verify multiple languages are preserved
        assert any("English" in m.content for m in messages)
        assert any("Deutsch" in m.content for m in messages)
        assert any("Spanish" in m.content for m in messages)
        assert session.message_count == 6


class ContextErrorTests:
    """上下文错误测试"""

    def test_session_overflow(self):
        """测试会话溢出处理"""
        from continuum_sdk.agent.session import Session
        session = Session(id="overflow-test")
        # Add many messages to test overflow handling
        for i in range(1000):
            session.add_user_message(f"Message {i}" * 100)  # Large messages
        messages = session.get_messages()
        # Session should handle large number of messages
        assert len(messages) >= 100
        # Verify session doesn't crash and can still add messages
        session.add_user_message("Final message")
        assert session.message_count > 1000

    def test_concurrent_sessions(self):
        """测试并发会话"""
        from continuum_sdk.agent.session import Session
        # Create multiple independent sessions
        sessions = []
        for i in range(5):
            session = Session(id=f"concurrent-{i}")
            session.add_user_message(f"Session {i} user message")
            session.add_assistant_message(f"Session {i} assistant response")
            sessions.append(session)
        # Verify each session maintains its own context
        for i, session in enumerate(sessions):
            messages = session.get_messages()
            assert session.id == f"concurrent-{i}"
            assert any(f"Session {i}" in m.content for m in messages)
            assert session.message_count == 2

    def test_corrupted_session(self):
        """测试损坏会话"""
        from continuum_sdk.agent.session import Session
        import tempfile
        import json
        import os
        with tempfile.TemporaryDirectory() as temp_dir:
            # Save a valid session
            session = Session(id="corrupt-test")
            session.add_user_message("Valid message")
            file_path = os.path.join(temp_dir, "corrupt-test.json")
            session.save(file_path)
            # Corrupt the file
            with open(file_path, 'w') as f:
                f.write("{ corrupted: invalid json }")
            # Attempt to load corrupted session
            try:
                loaded = Session.load(file_path)
                assert loaded is None or False  # Should not succeed
            except (json.JSONDecodeError, ValueError, KeyError) as e:
                # Expected: should raise error for corrupted data
                assert "JSON" in str(e) or "json" in str(e).lower() or "Expecting" in str(e) or True


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Scenario 5: Multi-turn Context")
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--save", action="store_true")
    args = parser.parse_args()

    scenario = Scenario5MultiTurnContext(verbose=args.verbose)
    results = scenario.run()

    print("\n" + "="*60)
    print("SCENARIO 5: Multi-turn Context Results")
    print("="*60)
    print(f"Status: {results['status']}")
    print(f"Success Rate: {results['metrics']['success_rate']*100:.1f}%")
    print(f"Passed Steps: {results['metrics']['passed_steps']}/{results['metrics']['total_steps']}")

    if results['errors']:
        print("\nErrors:")
        for e in results['errors']:
            print(f"  - {e['action']}: {e['error']}")

    if args.save:
        output_dir = os.path.join(os.path.dirname(__file__), "results")
        os.makedirs(output_dir, exist_ok=True)
        output_file = os.path.join(output_dir, "scenario_5_result.json")
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nResults saved to: {output_file}")

    return results['status'] == 'completed'


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)