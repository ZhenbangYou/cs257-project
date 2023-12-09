use std::collections::HashMap;

use z3::ast::{Ast, Int};
use z3::{ast::Bool, Context};

use crate::workflow::{Node, WorkflowGraph};

use crate::verifier::symbol::symbol;
use crate::workflow::schema::{InputCond, KeyRule};

#[derive(Clone)]
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
                        b_in._eq(b_out) // TODO: check whether use eq or implies
                    })
                    .collect::<Vec<_>>();
                Bool::and(ctx, &(implications.iter().collect::<Vec<_>>()))
            })
            .collect();

        // add schema constraints.
        let mut disjuncts = output_keys
            .keys()
            .map(|s| (*s, Vec::new()))
            .collect::<HashMap<_, _>>();
        // disjuncts[s] is a list of booleans such that if output[s] is true,
        // then at least one of the booleans in disjuncts[s] must be true
        node.output_schema.fixed_keys().for_each(|s| {
            if let Some(v) = disjuncts.get_mut(s) {
                v.push(Bool::from_bool(ctx, true));
            }
        });
        output_keys.keys().for_each(|s| {
            node.output_schema
                .dynamic_keys
                .iter()
                .for_each(|(rule, cond)| {
                    let constraint = match rule {
                        KeyRule::Identity => match cond {
                            InputCond::Always => Some(
                                input_keys
                                    .entry(s)
                                    .or_insert_with(|| Bool::new_const(ctx, symbol!()))
                                    .clone(),
                            ),
                            InputCond::MatchesKey(ss) if ss == s => Some(
                                input_keys
                                    .entry(s)
                                    .or_insert_with(|| Bool::new_const(ctx, symbol!()))
                                    .clone(),
                            ),
                            _ => None,
                        },
                        KeyRule::Fixed(ss) if ss == s => match cond {
                            InputCond::Always => Some(Bool::from_bool(ctx, true)),
                            InputCond::MatchesKey(s_input) => Some(
                                input_keys
                                    .entry(s_input)
                                    .or_insert_with(|| Bool::new_const(ctx, symbol!()))
                                    .clone(),
                            ),
                            InputCond::MatchesKeyValue(s_input, _) => Some(
                                input_keys
                                    .entry(s_input)
                                    .or_insert_with(|| Bool::new_const(ctx, symbol!()))
                                    .clone(),
                                // TODO: value is ignored for now
                            ),
                        },
                        KeyRule::IdWithPrefix(prefix) if s.starts_with(prefix) => {
                            let expected_input = s.strip_prefix(prefix).unwrap();
                            match cond {
                                InputCond::Always => Some(
                                    input_keys
                                        .entry(expected_input)
                                        .or_insert_with(|| Bool::new_const(ctx, symbol!()))
                                        .clone(),
                                ),
                                InputCond::MatchesKey(s_input) if s_input == expected_input => {
                                    Some(
                                        input_keys
                                            .entry(s_input)
                                            .or_insert_with(|| Bool::new_const(ctx, symbol!()))
                                            .clone(),
                                    )
                                }
                                InputCond::MatchesKeyValue(s_input, _)
                                    if s_input == expected_input =>
                                {
                                    Some(
                                        input_keys
                                            .entry(s_input)
                                            .or_insert_with(|| Bool::new_const(ctx, symbol!()))
                                            .clone(),
                                    )
                                } // TODO: value is ignored for now
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    if let Some(c) = constraint {
                        disjuncts.get_mut(s).unwrap().push(c);
                    }
                })
        });

        for (s, v) in disjuncts {
            let or = match v.len() {
                0 => Bool::from_bool(ctx, false),
                _ => Bool::or(ctx, &(v.iter().collect::<Vec<_>>())),
            };
            schema_constraints.push(output_keys[s]._eq(&or)); // TODO: check whether use implication or equivalence
        }

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
