use crate::workflow::{schema::KeyRule, WorkflowGraph};

use self::literals::Literals;

pub mod literals;

pub struct GraphVerifier {
    graph: WorkflowGraph,
    literals: Literals,
}

impl GraphVerifier {
    pub fn new(graph: WorkflowGraph) -> Self {
        // populate static literals (which is required inputs for all nodes)
        let static_literals = graph
            .nodes
            .iter()
            .flat_map(|node| node.required_inputs.iter())
            .map(|name| name.as_str())
            .collect::<Vec<_>>();
        let mut literals = Literals::new_with_literals(&static_literals);

        // populate dynamic literals
        // for each edge (src, dst)
        //  for each input in required_inputs of dst
        //    for each IdWithPrefix(prefix) key rule in output_schema of src
        //      add prefix + input to literals
        // TODO: need to include more literals
        // for each node, do inverse BFS to find all possible prefixes
        for (src, dsts) in graph.adj_list.iter().enumerate() {
            let required_inputs_union = dsts
                .iter()
                .flat_map(|(dst, _)| graph[*dst].required_inputs.iter())
                .collect::<Vec<_>>();
            let prefixes = graph[src]
                .output_schema
                .dynamic_keys()
                .iter()
                .filter_map(|(key_rule, _)| match key_rule {
                    KeyRule::IdWithPrefix(prefix) => Some(prefix.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            prefixes.iter().for_each(|prefix| {
                required_inputs_union.iter().for_each(|input| {
                    literals.get_or_add_index(&format!("{}{}", prefix, input));
                });
            });
        }

        Self { graph, literals }
    }

    pub fn literals(&self) -> &[String] {
        &self.literals.names
    }

    pub fn graph(&self) -> &WorkflowGraph {
        &self.graph
    }
}
