use std::collections::HashMap;

use z3::Context;

use crate::workflow::{NodeIdx, WorkflowGraph};

use self::ast::NodeAST;

pub mod ast;
pub mod symbol;
pub mod topsort;

pub struct GraphVerifier<'ctx, 'g> {
    context: &'ctx Context,
    pub graph: &'g WorkflowGraph,
    pub node_asts: Vec<NodeAST<'ctx, 'g>>,
}

pub struct ExecutionModel {
    node_idx: NodeIdx,
    input_keys: Vec<String>,
    output_keys: Vec<String>,
}

impl<'ctx, 'g> GraphVerifier<'ctx, 'g> {
    pub fn new(graph: &'g WorkflowGraph, context: &'ctx Context) -> Self {
        // construct node_asts (tests/workflow_graph.rs)
        let mut node_idx_to_ast = HashMap::new();
        topsort::topological_sort_reversed(graph)
            .into_iter()
            .for_each(|node_idx| {
                let node_ast = NodeAST::new(
                    context,
                    &graph.nodes[node_idx],
                    graph,
                    &graph.adj_list[node_idx]
                        .iter()
                        .map(|(child_idx, _)| node_idx_to_ast.get(child_idx).unwrap())
                        .collect::<Vec<_>>(),
                );
                node_idx_to_ast.insert(node_idx, node_ast);
            });
        let node_asts = graph
            .nodes
            .iter()
            .map(|node| node_idx_to_ast.get(&node.id).unwrap().clone())
            .collect::<Vec<_>>();
        Self {
            context,
            graph,
            node_asts,
        }
    }

    pub fn is_reachable(&self, target_node: NodeIdx) -> Option<Vec<ExecutionModel>> {
        todo!()
    }

    /// Minimum user provided input to make `target_node` reachable.
    pub fn minimum_input_set(
        &self,
        target_node: NodeIdx,
    ) -> Option<(Vec<String>, Vec<ExecutionModel>)> {
        // e.g. input variables: ["stock_name", "rec", "stock_price"]
        // binary search on CNT (number of input keys) (lo = 0, hi = 3)
        // v_in["stock_name"]: bool, v_in["rec"]: bool, v_in["stock_price"]: bool
        // c_in[s] = if v_in[s] then 1 else 0.
        // add constraint \sum_{s} c_in[s] <= CNT
        todo!()
    }

    /// Check whether we can start from the start node and can eventually reach any of the target_node in all scenarios.
    pub fn can_eventually_reach(&self, target_nodes: &[NodeIdx]) -> bool {
        todo!()
    }

    // TODO: minimum_input_set_to_eventually_reach
}
