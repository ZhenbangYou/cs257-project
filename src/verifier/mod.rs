use crate::workflow::WorkflowGraph;

use self::ast::NodeAST;

mod ast;
mod symbol;
mod topsort;

pub struct GraphVerifier<'ctx, 'g> {
    graph: &'g WorkflowGraph,
    node_asts: Vec<NodeAST<'ctx, 'g>>,
    ctr: usize,
}

impl<'ctx, 'g> GraphVerifier<'ctx, 'g> {
    pub fn new(graph: WorkflowGraph) -> Self {
        todo!()
    }

    pub fn graph(&self) -> &WorkflowGraph {
        &self.graph
    }
}
