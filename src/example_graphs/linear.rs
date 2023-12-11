use rand::Rng;
use rand::SeedableRng;

use crate::workflow::WorkflowGraph;

use super::MakeGraph;

pub struct Linear(pub usize);

impl MakeGraph for Linear {
    fn make_graph(&self) -> super::WorkflowGraphExt {
        // a graph with a linear chain of nodes
        // each node will pass its input to the next node, and have a fixed output
        // each node will have a distinct required input, and a random output from
        // the last 10 nodes
        // the last node will have a random required input from one of the previous nodes

        let mut g = WorkflowGraph::new();
        let num_nodes = self.0;
        let mut rng = rand_xorshift::XorShiftRng::seed_from_u64(0x12345678);

        let node_indices = (0..num_nodes)
            .map(|i| {
                let name = format!("node_{}", i);
                let mut required_inputs = if i == 0 {
                    vec![]
                } else {
                    let j = rng.gen_range((i.max(10) - 10)..i);
                    vec![format!("output_{}", j)]
                };
                required_inputs.push(format!("input_{}", i));
                let output_schema = crate::workflow::schema::OutputSchema::new()
                    .add_fixed(format!("output_{}", i))
                    .carry_all()
                    .build();
                g.add_node(&name, required_inputs, output_schema)
            })
            .collect::<Vec<_>>();
        node_indices
            .iter()
            .zip(node_indices.iter().skip(1))
            .for_each(|(i, j)| {
                g.add_edge(*i, *j, vec![]);
            });

        let test_reachable_node = node_indices[num_nodes - 1];
        g.set_start(node_indices[0]);
        super::WorkflowGraphExt::new(g, test_reachable_node)
    }

    fn name(&self) -> String {
        format!("linear_{}", self.0)
    }
}
