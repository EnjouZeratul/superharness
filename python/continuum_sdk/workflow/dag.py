"""工作流 DAG API

定义和执行 DAG（有向无环图）工作流。

Features:
    - 任务依赖管理：定义任务间的依赖关系
    - 并行执行：自动并行执行独立任务
    - 循环检测：检测并阻止循环依赖
    - ASCII 可视化：生成工作流结构图
    - 执行结果追踪：记录每个任务的执行状态

Quick Start:
    >>> from continuum_sdk.workflow import DAG, Node
    >>>
    >>> dag = DAG(name="data_pipeline")
    >>> dag.add_node(Node("fetch", func=lambda: "数据"))
    >>> dag.add_node(Node("process", func=lambda: "处理", depends_on=["fetch"]))
    >>> dag.add_node(Node("save", func=lambda: "保存", depends_on=["process"]))
    >>>
    >>> print(dag.visualize())  # 显示 DAG 结构
    >>> result = await dag.execute()

并行执行:
    >>> dag = DAG(name="parallel_analysis")
    >>> dag.add_node(Node("analyze_a", func=lambda: "A结果"))
    >>> dag.add_node(Node("analyze_b", func=lambda: "B结果"))
    >>> dag.add_node(Node("analyze_c", func=lambda: "C结果"))
    >>> dag.add_node(Node("summary", func=lambda: "汇总",
    ...     depends_on=["analyze_a", "analyze_b", "analyze_c"]))
    >>>
    >>> # analyze_a, analyze_b, analyze_c 将并行执行
    >>> result = await dag.execute(max_workers=3)

循环检测:
    >>> dag = DAG(name="circular")
    >>> dag.add_node(Node("a", depends_on=["c"]))  # a -> c
    >>> dag.add_node(Node("b", depends_on=["a"]))  # c -> a -> b
    >>> dag.add_node(Node("c", depends_on=["b"]))  # b -> c (循环!)
    >>>
    >>> has_cycle, path = dag.detect_cycle()
    >>> if has_cycle:
    ...     print(f"检测到循环: {' -> '.join(path)}")

节点状态:
    - PENDING: 等待执行
    - RUNNING: 正在执行
    - SUCCESS: 执行成功
    - FAILED: 执行失败
    - SKIPPED: 已跳过

执行结果:
    >>> for node_id, result in result.results.items():
    ...     print(f"{node_id}: {result.status.value}")
    ...     print(f"  输出: {result.output}")
    ...     print(f"  耗时: {result.duration_ms}ms")

DAGExecutor:
    >>> from continuum_sdk.workflow import DAGExecutor
    >>>
    >>> executor = DAGExecutor(dag, max_workers=4)
    >>> result = await executor.execute()
    >>> print(f"执行顺序: {result.execution_order}")
    >>> print(f"总耗时: {result.duration:.2f}s")

See Also:
    Node: DAG 节点定义
    NodeResult: 执行结果容器
    DAGExecutor: DAG 执行器
"""

import asyncio
from collections.abc import Callable
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class NodeStatus(Enum):
    """节点状态"""

    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class NodeResult:
    """节点执行结果"""

    node_id: str
    status: NodeStatus
    output: Any = None
    error: str | None = None
    duration_ms: int = 0


@dataclass
class Node:
    """工作流节点

    Usage:
        from continuum_sdk.workflow import Node

        # 创建节点
        node = Node("process", func=process_data)

        # 添加依赖
        node.depends_on("fetch")
    """

    id: str
    func: Callable | None = None
    name: str | None = None
    description: str | None = None
    dependencies: set[str] = field(default_factory=set)

    def depends_on(self, *node_ids: str) -> "Node":
        """添加依赖节点

        Args:
            *node_ids: 依赖的节点 ID

        Returns:
            self（支持链式调用）
        """
        self.dependencies.update(node_ids)
        return self

    def set_func(self, func: Callable) -> "Node":
        """设置执行函数"""
        self.func = func
        return self


class DAGResult:
    """DAG 执行结果"""

    def __init__(self, dag_id: str):
        self.dag_id = dag_id
        self._results: dict[str, NodeResult] = {}
        self._execution_order: list[str] = []

    @property
    def status(self) -> NodeStatus:
        """整体状态"""
        if not self._results:
            return NodeStatus.PENDING

        for result in self._results.values():
            if result.status == NodeStatus.FAILED:
                return NodeStatus.FAILED
            if result.status == NodeStatus.RUNNING:
                return NodeStatus.RUNNING

        return NodeStatus.SUCCESS

    def get_output(self, node_id: str) -> Any | None:
        """获取节点输出

        Args:
            node_id: 节点 ID

        Returns:
            节点输出结果
        """
        result = self._results.get(node_id)
        return result.output if result else None

    def get_result(self, node_id: str) -> NodeResult | None:
        """获取节点结果

        Args:
            node_id: 节点 ID

        Returns:
            节点执行结果
        """
        return self._results.get(node_id)

    def get_all_outputs(self) -> dict[str, Any]:
        """获取所有节点输出"""
        return {
            node_id: result.output
            for node_id, result in self._results.items()
            if result.output is not None
        }

    def failed_nodes(self) -> list[str]:
        """获取失败的节点 ID"""
        return [
            node_id
            for node_id, result in self._results.items()
            if result.status == NodeStatus.FAILED
        ]

    def execution_order(self) -> list[str]:
        """获取实际执行顺序"""
        return self._execution_order.copy()

    def _set_result(self, node_id: str, result: NodeResult) -> None:
        """设置节点结果"""
        self._results[node_id] = result
        self._execution_order.append(node_id)


class DAG:
    """工作流 DAG

    Usage:
        from continuum_sdk.workflow import DAG, Node

        # 创建 DAG
        dag = DAG("my_workflow")

        # 添加节点
        dag.add(Node("fetch", func=fetch_data))
        dag.add(Node("process", func=process).depends_on("fetch"))
        dag.add(Node("save", func=save).depends_on("process"))

        # 执行
        result = await dag.execute()

        # 获取结果
        output = result.get_output("save")
    """

    def __init__(self, id: str, name: str | None = None):
        """初始化 DAG

        Args:
            id: DAG ID
            name: 显示名称
        """
        self.id = id
        self.name = name or id
        self._nodes: dict[str, Node] = {}

    def add(self, node: Node) -> "DAG":
        """添加节点

        Args:
            node: 工作流节点

        Returns:
            self（支持链式调用）
        """
        self._nodes[node.id] = node
        return self

    def get(self, node_id: str) -> Node | None:
        """获取节点"""
        return self._nodes.get(node_id)

    def remove(self, node_id: str) -> bool:
        """移除节点"""
        if node_id in self._nodes:
            del self._nodes[node_id]
            # 清理依赖
            for node in self._nodes.values():
                node.dependencies.discard(node_id)
            return True
        return False

    def depends_on(self, node_id: str, *depends: str) -> "DAG":
        """添加依赖关系

        Args:
            node_id: 节点 ID
            *depends: 依赖的节点 ID

        Returns:
            self
        """
        node = self.get(node_id)
        if node:
            node.depends_on(*depends)
        return self

    def validate(self) -> list[str]:
        """验证 DAG

        Returns:
            错误消息列表（空列表表示验证通过）
        """
        errors = []

        # 检查循环依赖
        visited = set()
        rec_stack = set()

        def has_cycle(node_id: str) -> bool:
            visited.add(node_id)
            rec_stack.add(node_id)

            node = self.get(node_id)
            if node:
                for dep in node.dependencies:
                    if dep not in visited:
                        if has_cycle(dep):
                            return True
                    elif dep in rec_stack:
                        return True

            rec_stack.remove(node_id)
            return False

        for node_id in self._nodes:
            if node_id not in visited:
                if has_cycle(node_id):
                    errors.append("Cycle detected in DAG")
                    break

        # 检查缺失的依赖
        for node in self._nodes.values():
            for dep in node.dependencies:
                if dep not in self._nodes:
                    errors.append(
                        f"Node '{node.id}' depends on non-existent node '{dep}'"
                    )

        return errors

    def _get_execution_order(self) -> list[str]:
        """获取拓扑排序的执行顺序"""
        in_degree = {node_id: 0 for node_id in self._nodes}
        order = []
        queue = []

        # 计算入度
        for node in self._nodes.values():
            for dep in node.dependencies:
                if dep in in_degree:
                    in_degree[node.id] += 1

        # 入度为 0 的节点入队
        for node_id, degree in in_degree.items():
            if degree == 0:
                queue.append(node_id)

        # 拓扑排序
        while queue:
            node_id = queue.pop(0)
            order.append(node_id)

            # 更新依赖此节点的节点
            for other in self._nodes.values():
                if node_id in other.dependencies:
                    in_degree[other.id] -= 1
                    if in_degree[other.id] == 0:
                        queue.append(other.id)

        return order

    async def execute(
        self, inputs: dict[str, Any] | None = None, parallel: bool = True
    ) -> DAGResult:
        """执行工作流

        Args:
            inputs: 输入参数
            parallel: 是否并行执行独立节点

        Returns:
            执行结果
        """
        result = DAGResult(self.id)
        inputs = inputs or {}

        # 验证
        errors = self.validate()
        if errors:
            # 验证失败，标记所有节点为 SKIPPED
            for node_id in self._nodes:
                result._set_result(
                    node_id,
                    NodeResult(
                        node_id=node_id,
                        status=NodeStatus.FAILED,
                        error="; ".join(errors),
                    ),
                )
            return result

        # 获取执行顺序
        order = self._get_execution_order()
        outputs: dict[str, Any] = dict(inputs)

        if parallel:
            # 并行执行（按层级）
            levels = self._get_levels()
            for level in levels:
                tasks = []
                for node_id in level:
                    node = self.get(node_id)
                    if node:
                        tasks.append(self._execute_node(node, outputs, result))
                if tasks:
                    await asyncio.gather(*tasks)
        else:
            # 顺序执行
            for node_id in order:
                node = self.get(node_id)
                if node:
                    await self._execute_node(node, outputs, result)

        return result

    def _get_levels(self) -> list[list[str]]:
        """获取按层级分组的节点（用于并行执行）"""
        levels = []
        assigned = set()

        while len(assigned) < len(self._nodes):
            level = []
            for node_id, node in self._nodes.items():
                if node_id in assigned:
                    continue
                # 所有依赖都已分配
                if all(
                    dep in assigned for dep in node.dependencies if dep in self._nodes
                ):
                    level.append(node_id)
            if not level:
                break
            levels.append(level)
            assigned.update(level)

        return levels

    async def _execute_node(
        self, node: Node, outputs: dict[str, Any], result: DAGResult
    ) -> None:
        """执行单个节点"""
        import time

        start = time.time()

        # 检查依赖是否成功
        for dep in node.dependencies:
            dep_result = result.get_result(dep)
            if dep_result and dep_result.status != NodeStatus.SUCCESS:
                result._set_result(
                    node.id,
                    NodeResult(
                        node_id=node.id,
                        status=NodeStatus.SKIPPED,
                        error=f"Dependency '{dep}' failed",
                    ),
                )
                return

        # 执行节点
        try:
            if node.func is None:
                output = None
            else:
                # 收集依赖输出
                dep_outputs = {
                    dep: outputs.get(dep) for dep in node.dependencies if dep in outputs
                }

                # 调用函数
                func_result = node.func(**dep_outputs) if dep_outputs else node.func()

                if asyncio.iscoroutine(func_result):
                    output = await func_result
                else:
                    output = func_result

            outputs[node.id] = output
            duration = int((time.time() - start) * 1000)

            result._set_result(
                node.id,
                NodeResult(
                    node_id=node.id,
                    status=NodeStatus.SUCCESS,
                    output=output,
                    duration_ms=duration,
                ),
            )

        except Exception as e:
            duration = int((time.time() - start) * 1000)
            result._set_result(
                node.id,
                NodeResult(
                    node_id=node.id,
                    status=NodeStatus.FAILED,
                    error=str(e),
                    duration_ms=duration,
                ),
            )

    def visualize(self) -> str:
        """生成可视化字符串

        Returns:
            ASCII 图形表示
        """
        lines = [f"DAG: {self.id}"]
        lines.append("-" * 40)

        for node_id, node in self._nodes.items():
            deps = ", ".join(node.dependencies) if node.dependencies else "none"
            lines.append(f"  {node_id} <- [{deps}]")

        return "\n".join(lines)
