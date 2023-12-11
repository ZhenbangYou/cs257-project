pub mod buy_sell_stock;

use crate::workflow::{NodeIdx, WorkflowGraph};

pub struct WorkflowGraphExt {
    pub graph: WorkflowGraph,
    pub test_reachable_node: NodeIdx,
}

impl WorkflowGraphExt {
    pub fn new(graph: WorkflowGraph, test_reachable_node: NodeIdx) -> Self {
        Self {
            graph,
            test_reachable_node,
        }
    }
}

pub trait MakeGraph {
    const NAME: &'static str;

    fn make_graph(&self) -> WorkflowGraphExt;
}
