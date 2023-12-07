use std::collections::HashMap;

use z3::{ast::Bool, Context};

use crate::workflow::Node;

use crate::verifier::symbol::symbol;
use crate::workflow::schema::KeyRule;

pub struct NodeAST<'ctx, 'g> {
    pub ctx: &'ctx Context,
    pub node: &'g Node,
    pub input_keys: HashMap<&'g str, Bool<'ctx>>, // input_keys[s] = true iff s is an input key
    pub output_keys: HashMap<&'g str, Bool<'ctx>>, // output_keys[s] = true iff s is an output key
}

impl<'ctx, 'g> NodeAST<'ctx, 'g> {
    pub fn new(ctx: &'ctx Context, node: &'g Node) -> Self {
        let input_keys = node
            .required_inputs
            .iter()
            .map(|s| (s.as_str(), Bool::new_const(ctx, symbol!())))
            .collect::<HashMap<_, _>>();
        let output_keys = HashMap::new();

        Self {
            ctx,
            node,
            input_keys,
            output_keys,
        }
    }

    /// Should be called AFTER transition constraints
    pub fn generate_schema_constraints(&mut self) -> Vec<Bool<'ctx>> {
        let mut result = Vec::new();
        let ctx = self.ctx;
        // fixed keys
        for key in &self.node.output_schema.fixed_keys {
            let b = self
                .output_keys
                .entry(key.as_str())
                .or_insert_with(|| Bool::new_const(ctx, symbol!()));
            result.push(b.clone());
        }

        // dynamic keys
        for (keyrule, cond) in &self.node.output_schema.dynamic_keys {
            match keyrule {
                KeyRule::Identity => {
                    todo!()
                }
                KeyRule::Fixed(key) => {
                    todo!()
                }
                KeyRule::IdWithPrefix(prefix) => {
                    todo!()
                }
            }
        }

        result
    }
}
