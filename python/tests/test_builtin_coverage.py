"""
Builtin Tools Additional Coverage Tests

Tests to improve builtin.py coverage from 64% to 80%+.
"""

import json
import os
import tempfile

import pytest

from continuum_sdk.tools.builtin import (
    BuiltinTools,
    ToolCategory,
    ToolMeta,
)


class TestBuiltinToolsFallback:
    """Test fallback mode when Rust binding unavailable"""

    def test_fallback_mode_init(self, monkeypatch):
        """Test initialization in fallback mode"""
        # Force fallback mode
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            assert tools._executor is None
            assert len(tools._tools_cache) > 0
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_list_tools(self, monkeypatch):
        """Test list_tools in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            tool_list = tools.list_tools()
            tool_names = [t.name for t in tool_list]
            assert "read_file" in tool_names
            assert "write_file" in tool_names
            assert "grep" in tool_names
            assert "glob" in tool_names
            assert "bash" in tool_names
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_is_available(self, monkeypatch):
        """Test is_available in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            assert tools.is_available("read_file") is True
            assert tools.is_available("write_file") is True
            assert tools.is_available("nonexistent") is False
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_read_file(self, monkeypatch):
        """Test read_file in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, suffix=".txt"
            ) as f:
                f.write("test content for fallback")
                path = f.name

            tools = BuiltinTools()
            content = tools.read_file(path)
            assert "test content" in content

            os.unlink(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_write_file(self, monkeypatch):
        """Test write_file in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                path = os.path.join(tmpdir, "test_write.txt")

                tools = BuiltinTools()
                result = tools.write_file(path, "fallback write test")

                assert "Successfully" in result or "wrote" in result.lower()
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_edit_file(self, monkeypatch):
        """Test edit_file in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, suffix=".py"
            ) as f:
                f.write("old_text = 1\n")
                path = f.name

            tools = BuiltinTools()
            result = tools.edit_file(path, "old_text", "new_text")

            # Verify edit happened
            with open(path) as f:
                content = f.read()
            assert "new_text" in content

            os.unlink(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_list_directory(self, monkeypatch):
        """Test list_directory in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                # Create some files
                open(os.path.join(tmpdir, "file1.txt"), "w").close()
                open(os.path.join(tmpdir, "file2.py"), "w").close()

                tools = BuiltinTools()
                entries = tools.list_directory(tmpdir)

                assert len(entries) >= 2
                names = [e["name"] for e in entries]
                assert "file1.txt" in names
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_list_directory_not_found(self, monkeypatch):
        """Test list_directory with nonexistent path"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            result = tools.list_directory("/nonexistent/path/xyz")
            assert "error" in result[0] or result == []
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_grep(self, monkeypatch):
        """Test grep in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                path = os.path.join(tmpdir, "test.py")
                with open(path, "w") as f:
                    f.write("def test_function():\n    pass\n")

                tools = BuiltinTools()
                result = tools.grep("def\\s+\\w+", path=path)

                assert "test_function" in result or result.strip() != ""
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_glob(self, monkeypatch):
        """Test glob in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                # Create files
                open(os.path.join(tmpdir, "file.py"), "w").close()
                open(os.path.join(tmpdir, "file.txt"), "w").close()

                tools = BuiltinTools()
                result = tools.glob("*.py", path=tmpdir)

                assert "file.py" in result
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_bash(self, monkeypatch):
        """Test bash in fallback mode"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            result = tools.bash("echo hello")

            assert "hello" in result.lower()
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestBuiltinToolsExecute:
    """Test execute method"""

    def test_execute_read_file(self, monkeypatch):
        """Test execute for read_file"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, suffix=".txt"
            ) as f:
                f.write("execute test content")
                path = f.name

            tools = BuiltinTools()
            result = tools.execute("read_file", {"path": path})

            assert "execute test" in result

            os.unlink(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_execute_write_file(self, monkeypatch):
        """Test execute for write_file"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                path = os.path.join(tmpdir, "exec_write.txt")

                tools = BuiltinTools()
                result = tools.execute("write_file", {"path": path, "content": "test"})

                assert os.path.exists(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_execute_edit_file(self, monkeypatch):
        """Test execute for edit_file"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, suffix=".txt"
            ) as f:
                f.write("original text")
                path = f.name

            tools = BuiltinTools()
            result = tools.execute("edit_file", {
                "path": path,
                "old": "original",
                "new": "modified",
            })

            with open(path) as f:
                assert "modified" in f.read()

            os.unlink(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_execute_list_directory(self, monkeypatch):
        """Test execute for list_directory"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                open(os.path.join(tmpdir, "file.txt"), "w").close()

                tools = BuiltinTools()
                result = tools.execute("list_directory", {"path": tmpdir})

                # Result should be JSON array
                data = json.loads(result)
                assert len(data) >= 1
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_execute_grep(self, monkeypatch):
        """Test execute for grep"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                path = os.path.join(tmpdir, "test.py")
                with open(path, "w") as f:
                    f.write("def grep_target():\n    pass\n")

                tools = BuiltinTools()
                result = tools.execute("grep", {
                    "pattern": "grep_target",
                    "path": tmpdir,
                })

                assert "grep_target" in result or result.strip() != ""
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_execute_glob(self, monkeypatch):
        """Test execute for glob"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                open(os.path.join(tmpdir, "test.py"), "w").close()

                tools = BuiltinTools()
                result = tools.execute("glob", {
                    "pattern": "*.py",
                    "path": tmpdir,
                })

                assert "test.py" in result
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_execute_bash(self, monkeypatch):
        """Test execute for bash"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            result = tools.execute("bash", {"command": "echo execute_bash_test"})

            assert "execute_bash_test" in result
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_execute_unknown_tool(self, monkeypatch):
        """Test execute for unknown tool"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            with pytest.raises(NotImplementedError, match="not available"):
                tools.execute("unknown_tool_xyz", {})
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestToolCategory:
    """Test ToolCategory enum"""

    def test_all_categories(self):
        """Test all category values"""
        assert ToolCategory.FILE_OPS.value == "file_ops"
        assert ToolCategory.SEARCH.value == "search"
        assert ToolCategory.SHELL.value == "shell"
        assert ToolCategory.NETWORK.value == "network"
        assert ToolCategory.CODE_ANALYSIS.value == "code_analysis"
        assert ToolCategory.MEMORY.value == "memory"
        assert ToolCategory.WORKFLOW.value == "workflow"
        assert ToolCategory.SYSTEM.value == "system"
        assert ToolCategory.OTHER.value == "other"


class TestToolMeta:
    """Test ToolMeta dataclass"""

    def test_tool_meta_minimal(self):
        """Test minimal ToolMeta"""
        meta = ToolMeta(name="test", description="A test tool", category=ToolCategory.OTHER)
        assert meta.name == "test"
        assert meta.requires_confirmation is False
        assert meta.is_dangerous is False
        assert meta.parameters == {}

    def test_tool_meta_full(self):
        """Test full ToolMeta"""
        params = {"path": {"type": "string"}}
        meta = ToolMeta(
            name="write",
            description="Write file",
            category=ToolCategory.FILE_OPS,
            requires_confirmation=True,
            is_dangerous=True,
            parameters=params,
        )
        assert meta.requires_confirmation is True
        assert meta.is_dangerous is True
        assert meta.parameters == params


class TestBuiltinToolsSingleton:
    """Test get_builtin_tools singleton"""

    def test_get_builtin_tools_singleton(self):
        """Test singleton creation"""
        from continuum_sdk.tools.builtin import get_builtin_tools, _builtin_tools

        # Reset singleton
        import continuum_sdk.tools.builtin as builtin_module
        builtin_module._builtin_tools = None

        tools1 = get_builtin_tools()
        tools2 = get_builtin_tools()

        assert tools1 is tools2
        assert isinstance(tools1, BuiltinTools)

    def test_singleton_persistence(self):
        """Test singleton persists across calls"""
        from continuum_sdk.tools.builtin import get_builtin_tools

        tools1 = get_builtin_tools()
        tools2 = get_builtin_tools()

        # Same instance
        assert id(tools1) == id(tools2)


class TestBuiltinToolsCategoryGuess:
    """Test _guess_category edge cases"""

    def test_guess_category_search_patterns(self):
        """Test SEARCH category detection"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            assert tools._guess_category("grep") == ToolCategory.SEARCH
            assert tools._guess_category("glob_tool") == ToolCategory.SEARCH
            # Note: "search_files" contains "file" so it matches FILE_OPS first
            assert tools._guess_category("search_content") == ToolCategory.SEARCH
            assert tools._guess_category("find_pattern") == ToolCategory.SEARCH
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_guess_category_shell_patterns(self):
        """Test SHELL category detection"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            assert tools._guess_category("bash_execute") == ToolCategory.SHELL
            assert tools._guess_category("run_bash") == ToolCategory.SHELL
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_guess_category_file_ops_patterns(self):
        """Test FILE_OPS category detection"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            assert tools._guess_category("file_reader") == ToolCategory.FILE_OPS
            assert tools._guess_category("directory_list") == ToolCategory.FILE_OPS
            assert tools._guess_category("list_files") == ToolCategory.FILE_OPS
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestBuiltinToolsReadWithParameters:
    """Test read_file with various parameters"""

    def test_fallback_read_file_with_offset_limit(self):
        """Test read_file with offset and limit"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, suffix=".txt"
            ) as f:
                for i in range(10):
                    f.write(f"line {i+1}\n")
                path = f.name

            tools = BuiltinTools()
            content = tools.read_file(path, offset=3, limit=2)

            assert "line 3" in content
            assert "line 4" in content
            assert "line 1" not in content

            os.unlink(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_read_file_empty_result(self):
        """Test read_file returns empty for offset beyond file"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, suffix=".txt"
            ) as f:
                f.write("short file\n")
                path = f.name

            tools = BuiltinTools()
            content = tools.read_file(path, offset=100, limit=10)

            # Should return empty or minimal content
            assert content.strip() == "" or "short" not in content

            os.unlink(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestBuiltinToolsWriteVariations:
    """Test write_file variations"""

    def test_fallback_write_empty_content(self):
        """Test write_file with empty content"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                path = os.path.join(tmpdir, "empty.txt")

                tools = BuiltinTools()
                result = tools.write_file(path, "")

                # File should be created (empty or with newline)
                assert os.path.exists(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestBuiltinToolsBashVariations:
    """Test bash variations"""

    def test_fallback_bash_with_timeout(self):
        """Test bash with timeout parameter"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            tools = BuiltinTools()
            result = tools.bash("echo timeout_test", timeout_ms=5000)

            assert "timeout_test" in result.lower()
        finally:
            builtin_module.HAS_RUST_BINDING = original

    def test_fallback_bash_with_working_dir(self):
        """Test bash with working_dir parameter"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                tools = BuiltinTools()
                result = tools.bash("echo wd_test", working_dir=tmpdir)

                assert "wd_test" in result.lower()
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestBuiltinToolsGlobVariations:
    """Test glob variations"""

    def test_fallback_glob_with_path(self):
        """Test glob with path parameter"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                open(os.path.join(tmpdir, "file1.py"), "w").close()
                open(os.path.join(tmpdir, "file2.txt"), "w").close()

                tools = BuiltinTools()
                result = tools.glob("*.py", path=tmpdir)

                assert "file1.py" in result
                assert "file2.txt" not in result
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestBuiltinToolsGrepVariations:
    """Test grep variations"""

    def test_fallback_grep_with_glob(self):
        """Test grep with glob pattern"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.TemporaryDirectory() as tmpdir:
                py_file = os.path.join(tmpdir, "test.py")
                txt_file = os.path.join(tmpdir, "test.txt")

                with open(py_file, "w") as f:
                    f.write("python_pattern_match\n")
                with open(txt_file, "w") as f:
                    f.write("python_pattern_match\n")

                tools = BuiltinTools()
                result = tools.grep("python_pattern", path=tmpdir, glob="*.py")

                assert "python_pattern" in result
        finally:
            builtin_module.HAS_RUST_BINDING = original


class TestEditFileVariations:
    """Test edit_file variations"""

    def test_fallback_edit_file_args_key_mapping(self):
        """Test edit_file with different argument keys"""
        import continuum_sdk.tools.builtin as builtin_module
        original = builtin_module.HAS_RUST_BINDING
        builtin_module.HAS_RUST_BINDING = False

        try:
            with tempfile.NamedTemporaryFile(
                mode="w", delete=False, suffix=".txt"
            ) as f:
                f.write("original_text\n")
                path = f.name

            tools = BuiltinTools()
            # Test via execute with correct parameter keys (old/new for fallback)
            result = tools.execute("edit_file", {
                "path": path,
                "old": "original",
                "new": "modified",
            })

            with open(path) as f:
                content = f.read()
            assert "modified" in content

            os.unlink(path)
        finally:
            builtin_module.HAS_RUST_BINDING = original


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--cov=continuum_sdk.tools.builtin", "--cov-report=term-missing"])
