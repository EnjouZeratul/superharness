"""Tests for workflow DAG module.

Covers:
- DAG construction (add, get, remove nodes)
- Dependency management
- Cycle detection
- Topological sort / execution order
- Parallel and sequential execution
- Result handling
"""

import asyncio

import pytest

from continuum_sdk.workflow.dag import (
    DAG,
    DAGResult,
    Node,
    NodeResult,
    NodeStatus,
)


class TestNode:
    """Node class tests."""

    def test_node_creation(self):
        """Test creating a node with id."""
        node = Node("task_a")
        assert node.id == "task_a"
        assert node.func is None
        assert node.dependencies == set()

    def test_node_with_func(self):
        """Test creating a node with a function."""
        def my_func():
            return 42

        node = Node("task_a", func=my_func)
        assert node.func is my_func

    def test_node_depends_on_single(self):
        """Test adding single dependency."""
        node = Node("task_b")
        node.depends_on("task_a")
        assert node.dependencies == {"task_a"}

    def test_node_depends_on_multiple(self):
        """Test adding multiple dependencies."""
        node = Node("task_c")
        node.depends_on("task_a", "task_b")
        assert node.dependencies == {"task_a", "task_b"}

    def test_node_depends_on_chain(self):
        """Test chainable depends_on calls."""
        node = Node("task_d")
        result = node.depends_on("task_a").depends_on("task_b")
        assert result is node
        assert node.dependencies == {"task_a", "task_b"}

    def test_node_set_func_chain(self):
        """Test chainable set_func."""
        node = Node("task_a")
        result = node.set_func(lambda: 1)
        assert result is node
        assert node.func is not None


class TestDAGConstruction:
    """DAG construction tests."""

    def test_dag_creation(self):
        """Test creating a DAG."""
        dag = DAG("workflow_1")
        assert dag.id == "workflow_1"
        assert dag.name == "workflow_1"

    def test_dag_with_name(self):
        """Test creating a DAG with custom name."""
        dag = DAG("workflow_1", name="My Workflow")
        assert dag.id == "workflow_1"
        assert dag.name == "My Workflow"

    def test_dag_add_node(self):
        """Test adding a node."""
        dag = DAG("test")
        node = Node("task_a")
        result = dag.add(node)
        assert result is dag
        assert dag.get("task_a") is node

    def test_dag_add_multiple_nodes(self):
        """Test adding multiple nodes."""
        dag = DAG("test")
        dag.add(Node("a")).add(Node("b")).add(Node("c"))
        assert dag.get("a") is not None
        assert dag.get("b") is not None
        assert dag.get("c") is not None

    def test_dag_remove_node(self):
        """Test removing a node."""
        dag = DAG("test")
        dag.add(Node("a"))
        assert dag.remove("a") is True
        assert dag.get("a") is None

    def test_dag_remove_nonexistent(self):
        """Test removing non-existent node."""
        dag = DAG("test")
        assert dag.remove("nonexistent") is False

    def test_dag_remove_cleans_dependencies(self):
        """Test that removing a node cleans up dependencies."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.remove("a")
        # b's dependencies should no longer include 'a'
        assert "a" not in dag.get("b").dependencies

    def test_dag_depends_on(self):
        """Test adding dependencies via DAG method."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b"))
        result = dag.depends_on("b", "a")
        assert result is dag
        assert dag.get("b").dependencies == {"a"}


class TestDAGValidation:
    """DAG validation tests."""

    def test_validate_empty_dag(self):
        """Test validating empty DAG."""
        dag = DAG("empty")
        errors = dag.validate()
        assert errors == []

    def test_validate_single_node(self):
        """Test validating single node DAG."""
        dag = DAG("test")
        dag.add(Node("a"))
        errors = dag.validate()
        assert errors == []

    def test_validate_valid_dag(self):
        """Test validating valid DAG with dependencies."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("b"))
        errors = dag.validate()
        assert errors == []

    def test_validate_detects_cycle(self):
        """Test that validation detects cycles."""
        dag = DAG("test")
        dag.add(Node("a").depends_on("b"))
        dag.add(Node("b").depends_on("a"))
        errors = dag.validate()
        assert "Cycle detected in DAG" in errors

    def test_validate_detects_self_cycle(self):
        """Test that validation detects self-referential cycle."""
        dag = DAG("test")
        dag.add(Node("a").depends_on("a"))
        errors = dag.validate()
        assert "Cycle detected in DAG" in errors

    def test_validate_detects_missing_dependency(self):
        """Test that validation detects missing dependency."""
        dag = DAG("test")
        dag.add(Node("a").depends_on("nonexistent"))
        errors = dag.validate()
        assert any("non-existent" in e for e in errors)

    def test_validate_complex_cycle(self):
        """Test detecting complex cycle A->B->C->A."""
        dag = DAG("test")
        dag.add(Node("a").depends_on("c"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("b"))
        errors = dag.validate()
        assert "Cycle detected in DAG" in errors


class TestTopologicalSort:
    """Topological sort tests."""

    def test_empty_dag_order(self):
        """Test execution order for empty DAG."""
        dag = DAG("empty")
        order = dag._get_execution_order()
        assert order == []

    def test_single_node_order(self):
        """Test execution order for single node."""
        dag = DAG("test")
        dag.add(Node("a"))
        order = dag._get_execution_order()
        assert order == ["a"]

    def test_linear_dependency_order(self):
        """Test order for linear chain A->B->C."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("b"))
        order = dag._get_execution_order()
        assert order == ["a", "b", "c"]

    def test_parallel_nodes_order(self):
        """Test order when B and C both depend on A."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("a"))
        order = dag._get_execution_order()
        # A must come first, then B and C in some order
        assert order[0] == "a"
        assert set(order[1:]) == {"b", "c"}

    def test_diamond_dependency_order(self):
        """Test order for diamond A->B, A->C, B->D, C->D."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("a"))
        dag.add(Node("d").depends_on("b", "c"))
        order = dag._get_execution_order()
        # A first, D last, B and C in middle
        assert order[0] == "a"
        assert order[-1] == "d"
        assert set(order[1:3]) == {"b", "c"}


class TestDAGExecution:
    """DAG execution tests."""

    @pytest.mark.asyncio
    async def test_execute_empty_dag(self):
        """Test executing empty DAG."""
        dag = DAG("empty")
        result = await dag.execute()
        assert result.status == NodeStatus.PENDING

    @pytest.mark.asyncio
    async def test_execute_single_node(self):
        """Test executing single node."""
        dag = DAG("test")
        dag.add(Node("a", func=lambda: 42))
        result = await dag.execute()
        assert result.status == NodeStatus.SUCCESS
        assert result.get_output("a") == 42

    @pytest.mark.asyncio
    async def test_execute_with_dependencies(self):
        """Test executing nodes with dependencies."""
        dag = DAG("test")
        dag.add(Node("a", func=lambda: 10))
        dag.add(Node("b", func=lambda a: a + 5).depends_on("a"))
        result = await dag.execute()
        assert result.status == NodeStatus.SUCCESS
        assert result.get_output("a") == 10
        assert result.get_output("b") == 15

    @pytest.mark.asyncio
    async def test_execute_invalid_dag(self):
        """Test executing invalid DAG (with cycle)."""
        dag = DAG("test")
        dag.add(Node("a").depends_on("b"))
        dag.add(Node("b").depends_on("a"))
        result = await dag.execute()
        # Should fail validation
        assert result.status == NodeStatus.FAILED

    @pytest.mark.asyncio
    async def test_execute_node_without_func(self):
        """Test executing node without function."""
        dag = DAG("test")
        dag.add(Node("a"))  # No func
        result = await dag.execute()
        assert result.status == NodeStatus.SUCCESS
        assert result.get_output("a") is None

    @pytest.mark.asyncio
    async def test_execute_node_failure(self):
        """Test handling node failure."""
        def failing_func():
            raise ValueError("oops")

        dag = DAG("test")
        dag.add(Node("a", func=failing_func))
        result = await dag.execute()
        assert result.status == NodeStatus.FAILED
        assert "a" in result.failed_nodes()

    @pytest.mark.asyncio
    async def test_dependency_failure_skips_dependent(self):
        """Test that dependent nodes are skipped when dependency fails."""
        dag = DAG("test")
        dag.add(Node("a", func=lambda: 1/0))  # Fails
        dag.add(Node("b", func=lambda: 42).depends_on("a"))
        result = await dag.execute()
        assert result.get_result("a").status == NodeStatus.FAILED
        assert result.get_result("b").status == NodeStatus.SKIPPED

    @pytest.mark.asyncio
    async def test_sequential_execution(self):
        """Test sequential execution mode."""
        dag = DAG("test")
        dag.add(Node("a", func=lambda: 1))
        dag.add(Node("b", func=lambda: 2))
        result = await dag.execute(parallel=False)
        assert result.status == NodeStatus.SUCCESS

    @pytest.mark.asyncio
    async def test_async_node_func(self):
        """Test executing async node function."""
        async def async_func():
            await asyncio.sleep(0.01)
            return "async_result"

        dag = DAG("test")
        dag.add(Node("a", func=async_func))
        result = await dag.execute()
        assert result.get_output("a") == "async_result"

    @pytest.mark.asyncio
    async def test_execution_order_recorded(self):
        """Test that execution order is recorded."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("b"))
        result = await dag.execute()
        order = result.execution_order()
        assert order == ["a", "b", "c"]


class TestDAGResult:
    """DAGResult tests."""

    def test_result_creation(self):
        """Test creating a DAGResult."""
        result = DAGResult("test_dag")
        assert result.dag_id == "test_dag"
        assert result.status == NodeStatus.PENDING

    def test_get_output_none(self):
        """Test getting output for non-existent node."""
        result = DAGResult("test")
        assert result.get_output("nonexistent") is None

    def test_get_result_none(self):
        """Test getting result for non-existent node."""
        result = DAGResult("test")
        assert result.get_result("nonexistent") is None

    def test_failed_nodes(self):
        """Test getting failed nodes."""
        result = DAGResult("test")
        result._set_result("a", NodeResult("a", NodeStatus.SUCCESS))
        result._set_result("b", NodeResult("b", NodeStatus.FAILED, error="oops"))
        result._set_result("c", NodeResult("c", NodeStatus.SUCCESS))

        failed = result.failed_nodes()
        assert failed == ["b"]

    def test_get_all_outputs(self):
        """Test getting all outputs."""
        result = DAGResult("test")
        result._set_result("a", NodeResult("a", NodeStatus.SUCCESS, output=1))
        result._set_result("b", NodeResult("b", NodeStatus.SUCCESS, output=2))
        result._set_result("c", NodeResult("c", NodeStatus.SUCCESS, output=None))

        outputs = result.get_all_outputs()
        assert outputs == {"a": 1, "b": 2}

    def test_status_running(self):
        """Test status when nodes are running."""
        result = DAGResult("test")
        result._set_result("a", NodeResult("a", NodeStatus.SUCCESS))
        result._set_result("b", NodeResult("b", NodeStatus.RUNNING))
        assert result.status == NodeStatus.RUNNING


class TestNodeResult:
    """NodeResult tests."""

    def test_node_result_creation(self):
        """Test creating a NodeResult."""
        result = NodeResult(
            node_id="a",
            status=NodeStatus.SUCCESS,
            output=42,
            duration_ms=100
        )
        assert result.node_id == "a"
        assert result.status == NodeStatus.SUCCESS
        assert result.output == 42
        assert result.duration_ms == 100

    def test_node_result_defaults(self):
        """Test NodeResult default values."""
        result = NodeResult(node_id="a", status=NodeStatus.PENDING)
        assert result.output is None
        assert result.error is None
        assert result.duration_ms == 0


class TestDAGLevels:
    """DAG level grouping tests."""

    def test_get_levels_single_node(self):
        """Test levels for single node."""
        dag = DAG("test")
        dag.add(Node("a"))
        levels = dag._get_levels()
        assert levels == [["a"]]

    def test_get_levels_linear(self):
        """Test levels for linear chain."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("b"))
        levels = dag._get_levels()
        assert levels == [["a"], ["b"], ["c"]]

    def test_get_levels_parallel(self):
        """Test levels for parallel nodes."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("a"))
        levels = dag._get_levels()
        assert levels[0] == ["a"]
        assert set(levels[1]) == {"b", "c"}

    def test_get_levels_diamond(self):
        """Test levels for diamond pattern."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        dag.add(Node("c").depends_on("a"))
        dag.add(Node("d").depends_on("b", "c"))
        levels = dag._get_levels()
        assert levels[0] == ["a"]
        assert set(levels[1]) == {"b", "c"}
        assert levels[2] == ["d"]


class TestVisualize:
    """DAG visualization tests."""

    def test_visualize_empty(self):
        """Test visualizing empty DAG."""
        dag = DAG("test")
        viz = dag.visualize()
        assert "DAG: test" in viz
        assert "-" * 40 in viz

    def test_visualize_with_nodes(self):
        """Test visualizing DAG with nodes."""
        dag = DAG("test")
        dag.add(Node("a"))
        dag.add(Node("b").depends_on("a"))
        viz = dag.visualize()
        assert "a <- [none]" in viz
        assert "b <- [a]" in viz
