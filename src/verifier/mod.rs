use std::collections::{hash_map::RandomState, HashMap};

use z3::{ast::Bool, Context, Model, SatResult, Solver};

use crate::workflow::{Node, NodeIdx, WorkflowGraph};

use self::ast::NodeAST;

pub mod ast;
pub mod symbol;
pub mod topsort;

pub struct GraphVerifier<'ctx, 'g> {
    context: &'ctx Context,
    pub graph: &'g WorkflowGraph,
    pub node_asts: Vec<NodeAST<'ctx, 'g>>,
}

#[derive(Debug)]
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
            .map(|node| node_idx_to_ast.remove(&node.id).unwrap())
            .collect();
        Self {
            context,
            graph,
            node_asts,
        }
    }

    fn aggregate_schema_constraints(
        node_ast: &NodeAST<'ctx, '_>,
        context: &'ctx Context,
    ) -> Bool<'ctx> {
        Bool::and(
            context,
            &node_ast.schema_constraints.iter().collect::<Vec<_>>(),
        )
    }

    /// return value: Key: Node Index, Value: (incoming constraint bool, outgoing constraint bool)
    fn get_in_out_transition_constraints(&self) -> HashMap<NodeIdx, (Bool<'ctx>, Bool<'ctx>)> {
        // value: (incoming constraints, outgoing constraints)
        let mut in_out_transition_constraints: HashMap<
            usize,
            (Vec<&Bool<'_>>, Vec<&Bool<'_>>),
            RandomState,
        > = HashMap::from_iter(
            self.graph
                .nodes
                .iter()
                .map(|node| (node.id, (vec![], vec![]))),
        );

        self.node_asts.iter().for_each(|node_ast| {
            node_ast
                .transition_constraints
                .iter()
                .enumerate()
                .for_each(|(child_idx, bool)| {
                    let child_id = self.graph.adj_list[node_ast.node.id][child_idx].0;
                    // edge direction: node_ast.node.id -> child_id
                    in_out_transition_constraints
                        .get_mut(&child_id)
                        .unwrap()
                        .0 // incoming
                        .push(bool);
                    in_out_transition_constraints
                        .get_mut(&node_ast.node.id)
                        .unwrap()
                        .1 // outgoing
                        .push(bool);
                })
        });

        let constraint_bools_iter =
            in_out_transition_constraints
                .iter()
                .map(|(&node_idx, (incoming, outgoing))| {
                    let incoming_constraint = Bool::or(self.context, &incoming);
                    let outgoing_constraint = Bool::or(self.context, &outgoing);
                    (node_idx, (incoming_constraint, outgoing_constraint))
                });

        HashMap::from_iter(constraint_bools_iter)
    }

    pub fn is_reachable(&self, target_node: NodeIdx) -> Option<(Vec<ExecutionModel>, Model)> {
        let solver = Solver::new(&self.context);

        // enforce all schema constraints
        self.node_asts.iter().for_each(|node_ast| {
            solver.assert(&Self::aggregate_schema_constraints(node_ast, self.context))
        });

        // enforce all transition constraints
        let mut node_idx_to_transition_constraints = self.get_in_out_transition_constraints();
        node_idx_to_transition_constraints
            .get_mut(&target_node)
            .unwrap()
            .1 = Bool::from_bool(self.context, true); // clear the outgoing constraint for target node
        node_idx_to_transition_constraints
            .get_mut(&self.graph.start.unwrap())
            .unwrap()
            .0 = Bool::from_bool(self.context, true);
        let transition_constraits_bools = node_idx_to_transition_constraints
            .iter()
            .map(|(_, (incoming, outgoing))| incoming.implies(outgoing))
            .collect::<Vec<_>>();
        let transition_constraits_bools = transition_constraits_bools.iter().collect::<Vec<_>>();
        solver.assert(&Bool::and(self.context, &transition_constraits_bools));
        solver.assert(
            &node_idx_to_transition_constraints
                .get(&target_node)
                .unwrap()
                .0, // incoming
        );

        let res = solver.check();

        // println!("{:?}", solver.get_model());

        match res {
            SatResult::Sat => Some((vec![], solver.get_model().unwrap())),
            _ => None,
        }
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
