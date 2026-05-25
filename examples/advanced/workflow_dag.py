"""
Workflow DAG - 工作流 DAG 示例

DAG 工作流提供:
- 任务依赖管理
- 并行执行
- 循环检测
- ASCII 可视化

运行方式:
    python workflow_dag.py

预期输出:
- 工作流执行图
- 各节点结果
"""

import asyncio
from continuum_sdk.workflow.dag import DAG, Node


def basic_dag():
    """基础 DAG 示例"""
    print("=== 基础 DAG 示例 ===")

    # 创建 DAG
    dag = DAG(name="analysis_pipeline")

    # 添加节点（使用Node类）
    node1 = Node(
        name="fetch_data",
        description="获取数据",
        func=lambda: "数据获取完成"
    )

    node2 = Node(
        name="parse_data",
        description="解析数据",
        func=lambda: "数据解析完成"
    )
    node2.depends_on("fetch_data")

    node3 = Node(
        name="save_results",
        description="保存结果",
        func=lambda: "结果已保存"
    )
    node3.depends_on("parse_data")

    dag.add(node1)
    dag.add(node2)
    dag.add(node3)

    # 可视化 DAG
    print("\n工作流结构:")
    print(dag.visualize())

    # 执行 DAG
    result = asyncio.run(dag.execute())

    print(f"\n执行结果: {result}")


def parallel_execution():
    """并行执行示例"""
    print("=== 并行执行示例 ===")

    dag = DAG(name="parallel_analysis")

    # 添加并行节点
    dag.add(Node(name="analyze_a", description="分析模块A", func=lambda: "模块A: 100% 通过"))
    dag.add(Node(name="analyze_b", description="分析模块B", func=lambda: "模块B: 95% 通过"))
    dag.add(Node(name="analyze_c", description="分析模块C", func=lambda: "模块C: 100% 通过"))

    # 汇总节点（依赖所有分析节点）
    summary = Node(
        name="summary",
        description="生成汇总报告",
        func=lambda: "汇总报告已生成"
    )
    summary.depends_on("analyze_a")
    summary.depends_on("analyze_b")
    summary.depends_on("analyze_c")
    dag.add(summary)

    print("\n并行工作流:")
    print(dag.visualize())

    # 执行
    result = asyncio.run(dag.execute())
    print(f"\n并行执行完成")


def complex_dependencies():
    """复杂依赖示例"""
    print("=== 复杂依赖示例 ===")

    dag = DAG(name="build_pipeline")

    # 构建流水线节点
    nodes = [
        Node(name="checkout", description="检出代码", func=lambda: "代码已检出"),
        Node(name="install_deps", description="安装依赖", func=lambda: "依赖已安装"),
        Node(name="lint", description="代码检查", func=lambda: "检查通过"),
        Node(name="test_unit", description="单元测试", func=lambda: "12/12 通过"),
        Node(name="test_integration", description="集成测试", func=lambda: "5/5 通过"),
        Node(name="build", description="构建", func=lambda: "构建成功"),
        Node(name="deploy_staging", description="部署测试环境", func=lambda: "已部署到测试环境"),
        Node(name="smoke_test", description="冒烟测试", func=lambda: "冒烟测试通过"),
        Node(name="deploy_prod", description="部署生产环境", func=lambda: "已部署到生产环境"),
    ]

    # 设置依赖关系
    nodes[1].depends_on("checkout")  # install_deps depends on checkout
    nodes[2].depends_on("install_deps")  # lint
    nodes[3].depends_on("install_deps")  # test_unit
    nodes[4].depends_on("install_deps")  # test_integration
    nodes[5].depends_on("lint")  # build
    nodes[5].depends_on("test_unit")
    nodes[5].depends_on("test_integration")
    nodes[6].depends_on("build")  # deploy_staging
    nodes[7].depends_on("deploy_staging")  # smoke_test
    nodes[8].depends_on("smoke_test")  # deploy_prod

    for node in nodes:
        dag.add(node)

    print("\n构建流水线:")
    print(dag.visualize())

    result = asyncio.run(dag.execute())
    print(f"\n执行完成")


def cycle_detection():
    """循环检测示例"""
    print("=== 循环检测示例 ===")

    dag = DAG(name="circular")

    # 故意创建循环依赖
    node_a = Node(name="a", func=lambda: "a")
    node_b = Node(name="b", func=lambda: "b")
    node_c = Node(name="c", func=lambda: "c")

    node_a.depends_on("c")
    node_b.depends_on("a")
    node_c.depends_on("b")

    dag.add(node_a)
    dag.add(node_b)
    dag.add(node_c)

    # 验证DAG（会检测循环）
    try:
        dag.validate()
        print("无循环依赖")
    except ValueError as e:
        print(f"检测到循环依赖: {e}")


if __name__ == "__main__":
    basic_dag()
    print("\n" + "=" * 50 + "\n")
    parallel_execution()
    print("\n" + "=" * 50 + "\n")
    complex_dependencies()
    print("\n" + "=" * 50 + "\n")
    cycle_detection()
