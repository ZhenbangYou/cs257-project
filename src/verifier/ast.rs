use std::collections::HashMap;

use z3::{ast::Bool, Context};

use crate::workflow::{Node, WorkflowGraph};

use crate::verifier::symbol::symbol;
use crate::workflow::schema::KeyRule;

pub struct NodeAST<'ctx, 'g> {
    pub ctx: &'ctx Context,
    pub node: &'g Node,
    pub input_keys: HashMap<&'g str, Bool<'ctx>>, // input_keys[s] = true iff s is an input key
    pub output_keys: HashMap<&'g str, Bool<'ctx>>, // output_keys[s] = true iff s is an output key
    pub transition_constraints: Vec<Bool<'ctx>>, // transition_constraints[i] corresponds to adj[nodeIdx][i]
    pub schema_constraints: Vec<Bool<'ctx>>,     // schema_constraints should ALL be satisfied
}

impl<'ctx, 'g> NodeAST<'ctx, 'g> {
    pub fn new(
        ctx: &'ctx Context,
        node: &'g Node,
        graph: &'g WorkflowGraph,
        children_ast: &[&NodeAST<'ctx, 'g>],
    ) -> Self {
        let mut input_keys = node
            .required_inputs
            .iter()
            .map(|s| (s.as_str(), Bool::new_const(ctx, symbol!())))
            .collect::<HashMap<_, _>>();
        let mut output_keys = HashMap::new();

        let mut schema_constraints = Vec::new();

        // first, sanity check if children_ast is valid
        if graph.adj_list[node.id].len() != children_ast.len() {
            panic!("invalid children_ast");
        };

        // add transition constraints. For each child, for each s, if s is an input key of the child, then s must be an output key of current
        let transition_constraints = children_ast
            .iter()
            .map(|c| {
                let implications = c
                    .input_keys
                    .iter()
                    .map(|(s, b_in)| {
                        let b_out = output_keys
                            .entry(*s)
                            .or_insert_with(|| Bool::new_const(ctx, symbol!()));
                        b_in.implies(b_out)
                    })
                    .collect::<Vec<_>>();
                Bool::and(ctx, &(implications.iter().collect::<Vec<_>>()))
            })
            .collect();

        // TODO: add transition constraints

        Self {
            ctx,
            node,
            input_keys,
            output_keys,
            transition_constraints,
            schema_constraints,
        }
    }
}
