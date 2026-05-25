"""Continuum SDK Workflow Module

DAG-based workflow execution for task orchestration.

Features:
    - Task dependency management
    - Topological execution order
    - Parallel node execution
    - Cycle detection
    - Progress tracking

Quick Start:
    >>> from continuum_sdk.workflow import DAG, Node
    >>>
    >>> # Create DAG
    >>> dag = DAG("my_workflow")
    >>>
    >>> # Add nodes with dependencies
    >>> dag.add(Node("fetch", func=fetch_data))
    >>> dag.add(Node("process", func=process).depends_on("fetch"))
    >>> dag.add(Node("save", func=save).depends_on("process"))
    >>>
    >>> # Execute (async)
    >>> result = await dag.execute()
    >>> print(result.get_output("save"))

Parallel Execution:
    >>> dag.add(Node("a", func=task_a))
    >>> dag.add(Node("b", func=task_b).depends_on("a"))
    >>> dag.add(Node("c", func=task_c).depends_on("a"))  # b and c run in parallel
    >>> dag.add(Node("d", func=task_d).depends_on("b", "c"))
"""

from .dag import DAG, Node, NodeStatus, NodeResult, DAGResult

__all__ = [
    "DAG",
    "Node",
    "NodeStatus",
    "NodeResult",
    "DAGResult",
]
