//! # DAG Implementation
//!
//! 有向无环图结构实现。

use std::collections::{HashMap, HashSet};

use crate::types::Layer2Result;

use super::node::Node;

/// DAG 结构
pub struct Dag {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Vec<String>>,
    reverse_edges: HashMap<String, Vec<String>>,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: Node) -> Layer2Result<()> {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);

        if !self.edges.contains_key(&id) {
            self.edges.insert(id.clone(), Vec::new());
        }

        if !self.reverse_edges.contains_key(&id) {
            self.reverse_edges.insert(id, Vec::new());
        }

        Ok(())
    }

    /// 添加边
    pub fn add_edge(&mut self, from: &str, to: &str) -> Layer2Result<()> {
        // 验证节点存在
        if !self.nodes.contains_key(from) {
            return Err(anyhow::anyhow!("Source node not found: {}", from));
        }

        if !self.nodes.contains_key(to) {
            return Err(anyhow::anyhow!("Target node not found: {}", to));
        }

        // 添加边
        self.edges.get_mut(from).unwrap().push(to.to_string());
        self.reverse_edges
            .get_mut(to)
            .unwrap()
            .push(from.to_string());

        Ok(())
    }

    /// 验证是否有环
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node_id in self.nodes.keys() {
            if self.dfs_cycle(node_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    fn dfs_cycle(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if rec_stack.contains(node_id) {
            return true;
        }

        if visited.contains(node_id) {
            return false;
        }

        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        if let Some(neighbors) = self.edges.get(node_id) {
            for neighbor in neighbors {
                if self.dfs_cycle(neighbor, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(node_id);
        false
    }

    /// 拓扑排序
    pub fn topological_sort(&self) -> Layer2Result<Vec<String>> {
        if self.has_cycle() {
            return Err(anyhow::anyhow!("DAG contains cycle"));
        }

        let mut in_degree: HashMap<String, i32> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = Vec::new();

        // 计算入度
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
        }

        for node_id in self.nodes.keys() {
            if let Some(neighbors) = self.edges.get(node_id) {
                for neighbor in neighbors {
                    *in_degree.get_mut(neighbor).unwrap() += 1;
                }
            }
        }

        // 找到所有入度为 0 的节点
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push(node_id.clone());
            }
        }

        // Kahn 算法
        while !queue.is_empty() {
            let node_id = queue.remove(0);
            result.push(node_id.clone());

            if let Some(neighbors) = self.edges.get(&node_id) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        Ok(result)
    }

    /// 获取节点的依赖
    pub fn get_dependencies(&self, node_id: &str) -> Vec<String> {
        self.reverse_edges.get(node_id).cloned().unwrap_or_default()
    }

    /// 获取节点的后继
    pub fn get_successors(&self, node_id: &str) -> Vec<String> {
        self.edges.get(node_id).cloned().unwrap_or_default()
    }

    /// 获取节点
    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    /// 获取所有节点 ID
    pub fn node_ids(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    /// 节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 边数量
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_creation() {
        let dag = Dag::new();
        assert_eq!(dag.node_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut dag = Dag::new();
        let node = Node::new("test", "Test Node");
        dag.add_node(node).unwrap();

        assert_eq!(dag.node_count(), 1);
    }

    #[test]
    fn test_topological_sort() {
        let mut dag = Dag::new();

        let node_a = Node::new("a", "Node A");
        let node_b = Node::new("b", "Node B");
        let node_c = Node::new("c", "Node C");

        dag.add_node(node_a).unwrap();
        dag.add_node(node_b).unwrap();
        dag.add_node(node_c).unwrap();

        dag.add_edge("a", "b").unwrap();
        dag.add_edge("b", "c").unwrap();

        let sorted = dag.topological_sort().unwrap();
        assert_eq!(sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut dag = Dag::new();

        let node_a = Node::new("a", "Node A");
        let node_b = Node::new("b", "Node B");

        dag.add_node(node_a).unwrap();
        dag.add_node(node_b).unwrap();

        dag.add_edge("a", "b").unwrap();
        dag.add_edge("b", "a").unwrap();

        assert!(dag.has_cycle());
    }
}
