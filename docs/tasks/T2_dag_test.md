# T2 任务：dag.py 测试

## 基本信息

| 项目 | 内容 |
|------|------|
| 终端 | T2 |
| 模块 | `python/continuum_sdk/workflow/dag.py` |
| 当前覆盖率 | 0% |
| 目标覆盖率 | 80% |
| 优先级 | P0 |
| 参考规范 | `docs/TESTING_STANDARDS.md` |

---

## 任务目标

为工作流 DAG（有向无环图）模块编写完整测试，确保图构建、依赖解析、执行顺序正确。

---

## 需测试的功能

### 1. DAG 构建
- [ ] 添加节点
- [ ] 添加边（依赖关系）
- [ ] 节点属性设置
- [ ] 图的序列化/反序列化

### 2. 依赖解析
- [ ] 拓扑排序
- [ ] 获取节点依赖
- [ ] 获取依赖节点的节点
- [ ] 多层依赖解析

### 3. 执行顺序
- [ ] 计算执行顺序
- [ ] 并行节点识别
- [ ] 入度为0的节点获取

### 4. 循环检测
- [ ] 检测循环依赖
- [ ] 抛出 CycleError
- [ ] 定位循环位置

### 5. 边界条件
- [ ] 空图处理
- [ ] 单节点图
- [ ] 孤立节点
- [ ] 重复添加节点/边

---

## 测试要求

### 禁止行为
```python
# ❌ 不要这样写
def test_dag(self):
    dag = DAG()
    assert dag is not None  # 太弱

def test_topological_sort(self):
    dag = DAG()
    result = dag.topological_sort()
    # 无断言！
```

### 必须这样写
```python
# ✅ 正确示例
def test_topological_sort_simple_dag(self):
    """测试简单DAG的拓扑排序"""
    dag = DAG()
    dag.add_node("A")
    dag.add_node("B")
    dag.add_node("C")
    dag.add_edge("A", "B")  # A -> B
    dag.add_edge("B", "C")  # B -> C
    
    result = dag.topological_sort()
    
    assert result == ["A", "B", "C"]

def test_cycle_detection_raises_error(self):
    """测试循环依赖检测"""
    dag = DAG()
    dag.add_node("A")
    dag.add_node("B")
    dag.add_edge("A", "B")
    dag.add_edge("B", "A")  # 循环
    
    with pytest.raises(CycleError):
        dag.validate()

def test_parallel_nodes_identification(self):
    """测试并行节点识别"""
    dag = DAG()
    dag.add_node("A")
    dag.add_node("B")
    dag.add_node("C")
    dag.add_edge("A", "B")
    dag.add_edge("A", "C")
    
    parallel = dag.get_parallel_nodes("A")
    
    assert set(parallel) == {"B", "C"}
```

---

## 测试文件位置

```
python/tests/test_dag.py
```

---

## 验收标准

1. 覆盖率 ≥ 80%
2. 所有公开方法均有测试
3. 循环检测必须测试
4. 无占位测试、无 pass、无 assert True
5. CI 通过

---

## 提交方式

```bash
git checkout -b test/dag-coverage
git add python/tests/test_dag.py
git commit -m "test: add comprehensive tests for DAG workflow"
git push origin test/dag-coverage
# 创建 PR，标题格式：test: dag.py coverage to 80%
```
