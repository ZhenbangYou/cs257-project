//! Topological sort

use crate::workflow::{NodeIdx, WorkflowGraph};

fn dfs(
    graph: &WorkflowGraph,
    node: NodeIdx,
    visited: &mut Vec<bool>,
    post_order: &mut Vec<NodeIdx>,
) {
    visited[node] = true;
    for (dst, _) in &graph.adj_list[node] {
        if !visited[*dst] {
            dfs(graph, *dst, visited, post_order);
        }
    }
    post_order.push(node);
}

pub fn topological_sort_reversed(graph: &WorkflowGraph) -> Vec<NodeIdx> {
    let mut visited = vec![false; graph.nodes.len()];
    let mut post_order = Vec::new();

    let start = graph.start.expect("start node is not set");
    dfs(graph, start, &mut visited, &mut post_order);

    post_order
}
