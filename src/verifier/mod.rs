use std::collections::{hash_map::RandomState, HashMap, HashSet, VecDeque};

use z3::{
    ast::{Ast, Bool, Int},
    Context, Model, SatResult, Solver,
};

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

    /// result[i] contains all children j of i s.t. the transition from i to j is true
    fn build_graph_from_model(&self, model: &Model<'ctx>) -> Vec<Vec<NodeIdx>> {
        self.node_asts
            .iter()
            .map(|node_ast| {
                node_ast
                    .transition_constraints
                    .iter()
                    .enumerate()
                    .map(|(child_idx, bool)| {
                        (
                            self.graph.adj_list[node_ast.node.id][child_idx].0,
                            model.eval(bool, true).unwrap().as_bool().unwrap(),
                        )
                    })
                    .filter(|(_, bool)| *bool)
                    .map(|(child_id, _)| child_id)
                    .collect()
            })
            .collect()
    }

    fn find_path_by_bfs(
        &self,
        graph: &Vec<Vec<NodeIdx>>,
        target_node: NodeIdx,
    ) -> Option<Vec<NodeIdx>> {
        let mut visited = HashSet::new();
        let mut predecessor = HashMap::new();
        let mut queue = VecDeque::new();

        let start_node = self.graph.start.unwrap();
        queue.push_back(start_node);
        visited.insert(start_node);
        while let Some(front) = queue.pop_front() {
            if front == target_node {
                break;
            }
            graph[front].iter().for_each(|id| {
                if !visited.contains(id) {
                    visited.insert(*id);
                    predecessor.insert(*id, front);
                    queue.push_back(*id);
                }
            })
        }
        if visited.contains(&target_node) {
            let mut path = vec![];
            let mut cur = target_node;
            path.push(cur);
            while cur != start_node {
                cur = predecessor.remove(&cur).unwrap();
                path.push(cur);
            }
            path.reverse();
            Some(path)
        } else {
            None
        }
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

        match res {
            SatResult::Sat => {
                let model = solver.get_model().unwrap();
                let reachable_graph = self.build_graph_from_model(&model);
                let execution_path_by_idx = self
                    .find_path_by_bfs(&reachable_graph, target_node)
                    .unwrap();
                let execution_path_by_name = execution_path_by_idx
                    .iter()
                    .map(|idx| &self.graph.nodes[*idx].name)
                    .collect::<Vec<_>>();
                println!("Execution path: {:?}", execution_path_by_name);
                Some((vec![], model))
            }
            SatResult::Unsat => None,
            SatResult::Unknown => panic!("unknown!"),
        }
    }

    fn count_input_set(&self) -> Int<'ctx> {
        let input_as_int = self.node_asts[self.graph.start.unwrap()]
            .input_keys
            .iter()
            .map(|(_, v)| {
                v.ite(
                    &Int::from_i64(self.context, 1),
                    &Int::from_i64(self.context, 0),
                )
            })
            .collect::<Vec<_>>();
        Int::add(self.context, &input_as_int.iter().collect::<Vec<_>>())
    }

    fn try_minimum_input_set(
        &self,
        target_node: NodeIdx,
        input_set_size: usize,
    ) -> Option<(Vec<String>, Vec<ExecutionModel>)> {
        let solver = Solver::new(&self.context);

        // enforce input set size
        solver.assert(&self.count_input_set()._eq(&Int::from_i64(
            self.context,
            input_set_size.try_into().unwrap(),
        )));

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

        match solver.check() {
            SatResult::Sat => Some((vec![], vec![])),
            SatResult::Unsat => None,
            SatResult::Unknown => panic!("unknown!"),
        }
    }

    /// Minimum user provided input to make `target_node` reachable.
    pub fn minimum_input_set(
        &self,
        target_node: NodeIdx,
    ) -> Option<(usize, Vec<String>, Vec<ExecutionModel>)> {
        // e.g. input variables: ["stock_name", "rec", "stock_price"]
        // binary search on CNT (number of input keys) (lo = 0, hi = 3)
        // v_in["stock_name"]: bool, v_in["rec"]: bool, v_in["stock_price"]: bool
        // c_in[s] = if v_in[s] then 1 else 0.
        // add constraint \sum_{s} c_in[s] <= CNT

        // binary search
        let mut left = 0;
        let mut right = self.node_asts[self.graph.start.unwrap()].input_keys.len();
        let mut cur_res = None;
        while left < right {
            let mid = (left + right) - 1;
            if let Some(res) = self.try_minimum_input_set(target_node, mid) {
                right = mid;
                cur_res = Some((mid, res.0, res.1));
            } else {
                left = mid + 1;
            }
        }
        cur_res
    }

    /// Check whether we can start from the start node and can eventually reach any of the target_node in all scenarios.
    pub fn can_eventually_reach(&self, target_nodes: &[NodeIdx]) -> bool {
        let mut conjunctive_clauses = vec![];

        // enforce all schema constraints
        self.node_asts.iter().for_each(|node_ast| {
            conjunctive_clauses.push(Self::aggregate_schema_constraints(node_ast, self.context));
        });

        // enforce all transition constraints
        let mut node_idx_to_transition_constraints = self.get_in_out_transition_constraints();
        target_nodes.iter().for_each(|target_node| {
            node_idx_to_transition_constraints
                .get_mut(target_node)
                .unwrap()
                .1 = Bool::from_bool(self.context, true); // clear the outgoing constraint for target node
        });
        node_idx_to_transition_constraints
            .get_mut(&self.graph.start.unwrap())
            .unwrap()
            .0 = Bool::from_bool(self.context, true);
        let mut transition_constraits_bools = node_idx_to_transition_constraints
            .iter()
            .map(|(_, (incoming, outgoing))| incoming.implies(outgoing))
            .collect::<Vec<_>>();
        conjunctive_clauses.append(&mut transition_constraits_bools);

        let reach_target = target_nodes
            .iter()
            .map(|target_node| {
                &node_idx_to_transition_constraints
                    .get(target_node)
                    .unwrap()
                    .0
            })
            .collect::<Vec<_>>();
        conjunctive_clauses.push(Bool::or(self.context, &reach_target));

        let solver = Solver::new(&self.context);
        solver.assert(
            &Bool::and(
                self.context,
                &conjunctive_clauses.iter().collect::<Vec<_>>(),
            )
            .not(),
        );

        match solver.check() {
            SatResult::Sat => false,
            SatResult::Unsat => true,
            SatResult::Unknown => panic!("unknown!"),
        }
    }

    // TODO: minimum_input_set_to_eventually_reach
}
