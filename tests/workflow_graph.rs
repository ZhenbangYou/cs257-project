use std::collections::HashMap;

use cs257_project::{
    verifier::{
        ast::NodeAST, symbol::get_symbol_count, topsort::topological_sort_reversed, GraphVerifier,
    },
    workflow::{
        schema::{InputCond, KeyRule, OutputSchema},
        WorkflowGraph,
    },
};
use z3::{Config, Context};

fn make_graph() -> WorkflowGraph {
    let mut g = WorkflowGraph::new();
    let check_stock_price = g.add_node(
        "check_stock_price",
        vec![],
        OutputSchema::new()
            .add_fixed("stock_price")
            .carry_all()
            .build(),
    );
    let buy_sell_rec = g.add_node(
        "buy_sell_rec",
        vec!["stock_price".to_string()],
        OutputSchema::new().add_fixed("rec").carry_all().build(),
    );
    let buy_or_sell = g.add_node(
        "buy_or_sell",
        vec!["rec".to_string()],
        OutputSchema::new().carry_all().build(),
    );
    let buy = g.add_node(
        "buy",
        vec!["stock_name".to_string(), "stock_price".to_string()],
        OutputSchema::new()
            .add_fixed("quantity")
            .add_rule_for_every_input(
                KeyRule::IdWithPrefix("previous_input.".to_string()),
                InputCond::Always,
            )
            .build(),
    );
    let sell = g.add_node(
        "sell",
        vec!["stock_name".to_string()],
        OutputSchema::new()
            .add_fixed("quantity")
            .add_rule_for_every_input(
                KeyRule::IdWithPrefix("previous_input.".to_string()),
                InputCond::Always,
            )
            .build(),
    );
    let report_result = g.add_node(
        "report_result",
        vec![
            "previous_input.stock_name".to_string(),
            "previous_input.rec".to_string(),
            "quantity".to_string(),
        ],
        OutputSchema::new().carry_all().build(),
    );
    g.add_edge(check_stock_price, buy_sell_rec, vec![])
        .add_edge(buy_sell_rec, buy_or_sell, vec![])
        .add_edge(
            buy_or_sell,
            buy,
            vec![InputCond::MatchesKeyValue(
                "rec".to_string(),
                "buy".to_string(),
            )],
        )
        .add_edge(
            buy_or_sell,
            sell,
            vec![InputCond::MatchesKeyValue(
                "rec".to_string(),
                "sell".to_string(),
            )],
        )
        .add_edge(buy, report_result, vec![])
        .add_edge(sell, report_result, vec![])
        .set_start(check_stock_price);
    g
}

#[test]
fn construct_graph_without_panic() {
    let g = make_graph();
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
    let g = make_graph();
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
    let graph = make_graph();
    for i in 0..graph.nodes.len() {
        let ctx = Context::new(&Config::default());
        let graph_verifier = GraphVerifier::new(&graph, &ctx);
        println!("{}: {:?}", i, graph_verifier.is_reachable(i));
    }
}
