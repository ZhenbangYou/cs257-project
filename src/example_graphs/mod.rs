pub mod buy_sell_stock;
pub mod linear;
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
    fn name(&self) -> String;

    fn make_graph(&self) -> WorkflowGraphExt;
}
