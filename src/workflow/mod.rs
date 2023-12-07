use std::ops::Index;

use self::schema::InputCond;

pub mod schema;

pub type NodeIdx = usize;

#[derive(Debug)]
pub struct Node {
    pub id: NodeIdx,
    pub name: String,
    pub required_inputs: Vec<String>,
    pub output_schema: schema::OutputSchema,
}

impl Node {
    pub fn new(
        id: NodeIdx,
        name: String,
        required_inputs: Vec<String>,
        output_schema: schema::OutputSchema,
    ) -> Self {
        Self {
            id,
            name,
            required_inputs,
            output_schema,
        }
    }
}

#[derive(Debug)]
pub struct WorkflowGraph {
    pub nodes: Vec<Node>,
    pub adj_list: Vec<Vec<(NodeIdx, Vec<InputCond>)>>,
    pub start: Option<NodeIdx>,
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
    ) -> NodeIdx {
        let id = self.nodes.len();
        self.nodes.push(Node::new(
            id,
            name.to_owned(),
            required_inputs,
            output_schema,
        ));
        self.adj_list.push(Vec::new());
        id
    }

    /// Add an edge from `src` to `dst` with additional transition condition.
    ///  A transition is good if and only if required_inputs are satisfied and:
    ///  * for each `InputCond`, exists a (key, value) pair in outputs of `src` that satisfies the condition
    pub fn add_edge(
        &mut self,
        src: NodeIdx,
        dst: NodeIdx,
        additional_transition_condition: Vec<InputCond>,
    ) -> &mut Self {
        self.adj_list[src].push((dst, additional_transition_condition));
        self
    }

    pub fn set_start(&mut self, node: NodeIdx) -> &mut Self {
        if self.start.is_some() {
            panic!("Start node already set");
        }
        self.start = Some(node);
        self
    }

    pub fn get_node(&self, node: NodeIdx) -> &Node {
        &self.nodes[node]
    }
}

impl Index<usize> for WorkflowGraph {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}
