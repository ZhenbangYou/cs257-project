use cs257_project::{
    verifier::GraphVerifier,
    workflow::{
        schema::{InputCond, KeyRule, OutputSchema},
        WorkflowGraph,
    },
};

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
            vec![InputCond::matches_key_value("rec", "buy")],
        )
        .add_edge(
            buy_or_sell,
            sell,
            vec![InputCond::matches_key_value("rec", "sell")],
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
fn get_literals_without_panic() {
    let g = make_graph();
    let verifier = GraphVerifier::new(g);
    println!("Literals: {:?}", verifier.literals());
}
