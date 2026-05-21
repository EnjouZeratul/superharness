"""工作流 DAG 示例

展示如何创建和执行 DAG 工作流。
"""

import asyncio
from continuum_sdk.workflow import DAG, Node, NodeStatus


# ==================== 定义节点函数 ====================

async def fetch_data():
    """获取数据"""
    print("   [fetch_data] 正在获取数据...")
    await asyncio.sleep(0.5)
    return {"records": [1, 2, 3, 4, 5], "source": "api"}


def validate_data(fetch_data):
    """验证数据"""
    print("   [validate_data] 正在验证数据...")
    data = fetch_data
    if not data.get("records"):
        raise ValueError("没有数据")
    return {"valid": True, "count": len(data["records"])}


async def process_batch_1(fetch_data):
    """处理批次 1"""
    print("   [process_batch_1] 处理中...")
    await asyncio.sleep(0.3)
    records = fetch_data["records"][:3]
    return {"batch": 1, "processed": [r * 2 for r in records]}


async def process_batch_2(fetch_data):
    """处理批次 2"""
    print("   [process_batch_2] 处理中...")
    await asyncio.sleep(0.4)
    records = fetch_data["records"][3:]
    return {"batch": 2, "processed": [r * 3 for r in records]}


def merge_results(process_batch_1, process_batch_2, validate_data):
    """合并结果"""
    print("   [merge_results] 合并中...")
    return {
        "total_processed": len(process_batch_1["processed"]) + len(process_batch_2["processed"]),
        "batch_1": process_batch_1,
        "batch_2": process_batch_2,
        "validation": validate_data
    }


async def save_results(merge_results):
    """保存结果"""
    print("   [save_results] 保存中...")
    await asyncio.sleep(0.2)
    return {"saved": True, "records": merge_results["total_processed"]}


def notify_success(save_results):
    """通知成功"""
    print("   [notify_success] 发送成功通知...")
    return {"notified": True, "message": "工作流完成"}


def notify_failure(**kwargs):
    """通知失败（备用节点）"""
    print("   [notify_failure] 发送失败通知...")
    return {"notified": False, "message": "工作流失败"}


# ==================== 构建 DAG ====================

def create_data_pipeline():
    """创建数据处理工作流"""
    dag = DAG("data-pipeline", name="数据处理流水线")

    # 添加节点
    dag.add(Node("fetch", func=fetch_data, description="获取数据"))
    dag.add(Node("validate", func=validate_data, description="验证数据"))
    dag.add(Node("batch1", func=process_batch_1, description="处理批次1"))
    dag.add(Node("batch2", func=process_batch_2, description="处理批次2"))
    dag.add(Node("merge", func=merge_results, description="合并结果"))
    dag.add(Node("save", func=save_results, description="保存结果"))
    dag.add(Node("notify_ok", func=notify_success, description="成功通知"))
    dag.add(Node("notify_fail", func=notify_failure, description="失败通知"))

    # 设置依赖
    dag.depends_on("validate", "fetch")
    dag.depends_on("batch1", "fetch")
    dag.depends_on("batch2", "fetch")
    dag.depends_on("merge", "batch1", "batch2", "validate")
    dag.depends_on("save", "merge")
    dag.depends_on("notify_ok", "save")

    return dag


# ==================== 运行示例 ====================

async def main():
    print("=== 工作流 DAG 示例 ===\n")

    # 1. 创建 DAG
    dag = create_data_pipeline()

    # 2. 可视化 DAG
    print("1. DAG 结构:")
    print(dag.visualize())
    print()

    # 3. 验证 DAG
    print("2. 验证 DAG:")
    errors = dag.validate()
    if errors:
        print(f"   验证失败: {errors}")
        return
    print("   验证通过\n")

    # 4. 顺序执行
    print("3. 顺序执行:")
    result = await dag.execute(parallel=False)
    print(f"   状态: {result.status.value}")
    print(f"   执行顺序: {result.execution_order()}")
    print(f"   最终输出: {result.get_output('save')}")
    print()

    # 5. 并行执行
    print("4. 并行执行:")
    dag2 = create_data_pipeline()
    result2 = await dag2.execute(parallel=True)
    print(f"   状态: {result2.status.value}")
    print(f"   所有输出: {result2.get_all_outputs()}")
    print()

    # 6. 测试失败处理
    print("5. 测试失败节点:")

    def failing_func():
        raise RuntimeError("模拟失败")

    dag3 = DAG("test-failure")
    dag3.add(Node("a", func=lambda: "ok"))
    dag3.add(Node("b", func=failing_func).depends_on("a"))
    dag3.add(Node("c", func=lambda: "ok").depends_on("b"))

    result3 = await dag3.execute()
    print(f"   状态: {result3.status.value}")
    print(f"   失败节点: {result3.failed_nodes()}")

    # 检查节点状态
    b_result = result3.get_result("b")
    c_result = result3.get_result("c")
    print(f"   b 状态: {b_result.status.value}, 错误: {b_result.error}")
    print(f"   c 状态: {c_result.status.value}, 原因: {c_result.error}")


if __name__ == "__main__":
    asyncio.run(main())
