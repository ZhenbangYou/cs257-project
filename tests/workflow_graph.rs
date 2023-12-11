use std::collections::HashMap;

use cs257_project::{
    example_graphs::{buy_sell_stock::BuySellStock, MakeGraph as _},
    verifier::{
        ast::NodeAST, symbol::get_symbol_count, topsort::topological_sort_reversed, GraphVerifier,
    },
};
use z3::{Config, Context};

#[test]
fn construct_graph_without_panic() {
    let g = BuySellStock::new(true).make_graph().graph;
    println!("Nodes: {:#?}", g.nodes);
    println!("Edges: ");
    g.adj_list.iter().enumerate().for_each(|(i, adj)| {
        adj.iter().for_each(|(j, cond)| {
            println!("{} -> {} with {:?}", &g[i].name, &g[*j].name, cond);
        });
    });
    println!("Start: {:?}", g[g.start.unwrap()].name);
}

#[test]
fn find_literals_without_panic() {
    let g = BuySellStock::new(true).make_graph().graph;
    let order = topological_sort_reversed(&g);
    let ctx = Context::new(&Config::default());
    let mut asts = HashMap::new();
    let mut symbol_counts = (0..g.nodes.len()).map(|_| 0).collect::<Vec<_>>();
    for i in order.iter().copied() {
        let children_ast = g.adj_list[i]
            .iter()
            .map(|(j, _)| asts.get(j).unwrap())
            .collect::<Vec<_>>();
        let cnt0 = get_symbol_count();
        let ast = NodeAST::new(&ctx, &g[i], &g, &children_ast);
        let cnt1 = get_symbol_count();
        asts.insert(i, ast);
        symbol_counts[i] = cnt1 - cnt0;
    }
    for ast in order.iter().rev().map(|i| asts.get(i).unwrap()) {
        println!("name: {}", ast.node.name);
        println!("required_input: {:?}", ast.node.required_inputs);
        println!("output_schema: {:?}", ast.node.output_schema);
        println!("input variables: {:?}", ast.input_keys.keys());
        println!("output variables: {:?}", ast.output_keys.keys());
        println!("Added symbols: {}", symbol_counts[ast.node.id]);
        println!()
    }
}

#[test]
fn test_reachable() {
    let graph = BuySellStock::new(true).make_graph().graph;
    for i in 0..graph.nodes.len() {
        let ctx = Context::new(&Config::default());
        let graph_verifier = GraphVerifier::new(&graph, &ctx);
        let result = graph_verifier.is_reachable(i);
        println!(
            "{}: {:?}",
            &graph.get_node(i).name,
            if result.is_some() {
                "reachable"
            } else {
                "not reachable"
            }
        );
        if let Some((_, model)) = result {
            println!("Model:");
            for j in 0..graph.nodes.len() {
                println!(
                    "model required input: {:?}",
                    graph.get_node(j).required_inputs
                );
                println!(
                    "model input variables: {:?}",
                    graph_verifier
                        .node_asts
                        .get(j)
                        .unwrap()
                        .eval_input_keys(&model)
                );
                println!(
                    "model output variables: {:?}",
                    graph_verifier
                        .node_asts
                        .get(j)
                        .unwrap()
                        .eval_output_keys(&model)
                );
                println!();
            }
        }

        println!();
    }
}

#[test]
fn test_can_eventually_reach() {
    let graph = BuySellStock::new(true).make_graph().graph;
    for i in 0..graph.nodes.len() {
        let ctx = Context::new(&Config::default());
        let graph_verifier = GraphVerifier::new(&graph, &ctx);
        let result = graph_verifier.can_eventually_reach(&[i]);
        println!("{}", result);
    }
}

#[test]
fn test_minimum_input_set_for_reachable() {
    let graph = BuySellStock::new(true).make_graph().graph;
    for i in 0..graph.nodes.len() {
        let ctx = Context::new(&Config::default());
        let graph_verifier = GraphVerifier::new(&graph, &ctx);
        let result = graph_verifier.minimum_input_set_for_reachable(i);
        println!("{:?}", result);
    }
}

#[test]
fn test_minimum_input_set_for_can_eventually_reach() {
    let graph = BuySellStock::new(true).make_graph().graph;
    for i in 0..graph.nodes.len() {
        let ctx = Context::new(&Config::default());
        let graph_verifier = GraphVerifier::new(&graph, &ctx);
        let result = graph_verifier.minimum_input_set_for_can_eventually_reach(&[i]);
        println!("{:?}", result);
    }
}
