use std::ops::Index;

use self::schema::InputCond;

pub mod schema;

pub type NodeId = usize;

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub required_inputs: Vec<String>,
    pub output_schema: schema::OutputSchema,
}

impl Node {
    pub fn new(
        name: String,
        required_inputs: Vec<String>,
        output_schema: schema::OutputSchema,
    ) -> Self {
        Self {
            name,
            required_inputs,
            output_schema,
        }
    }
}

#[derive(Debug)]
pub struct WorkflowGraph {
    pub nodes: Vec<Node>,
    pub adj_list: Vec<Vec<(NodeId, Vec<InputCond>)>>,
    pub start: Option<NodeId>,
}

impl WorkflowGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            adj_list: Vec::new(),
            start: None,
        }
    }

    pub fn add_node(
        &mut self,
        name: &str,
        required_inputs: Vec<String>,
        output_schema: schema::OutputSchema,
    ) -> NodeId {
        let node = Node::new(name.to_string(), required_inputs, output_schema);
        self.nodes.push(node);
        self.adj_list.push(Vec::new());
        self.nodes.len() - 1
    }
    /**
    Add an edge from `src` to `dst` with additional transition condition.
     A transition is good if and only if required_inputs are satisfied and:
     * for each `InputCond`, exists a (key, value) pair in outputs of `src` that satisfies the condition
     */
    pub fn add_edge(
        &mut self,
        src: NodeId,
        dst: NodeId,
        additional_transition_condition: Vec<InputCond>,
    ) -> &mut Self {
        self.adj_list[src].push((dst, additional_transition_condition));
        self
    }

    pub fn set_start(&mut self, node: NodeId) -> &mut Self {
        if self.start.is_some() {
            panic!("Start node already set");
        }
        self.start = Some(node);
        self
    }

    pub fn get_node(&self, node: NodeId) -> &Node {
        &self.nodes[node]
    }
}

impl Index<usize> for WorkflowGraph {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}
