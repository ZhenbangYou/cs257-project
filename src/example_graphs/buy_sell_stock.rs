use crate::workflow::{
    schema::{InputCond, KeyRule, OutputSchema},
    WorkflowGraph,
};

use super::{MakeGraph, WorkflowGraphExt};

pub struct BuySellStock {
    include_non_existent: bool,
}

impl BuySellStock {
    pub fn new(include_non_existent: bool) -> Self {
        Self {
            include_non_existent,
        }
    }
}

impl MakeGraph for BuySellStock {
    const NAME: &'static str = "buy_sell_stock";

    fn make_graph(&self) -> WorkflowGraphExt {
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
        let mut report_result_required_input = vec![
            "previous_input.stock_name".to_string(),
            "previous_input.rec".to_string(),
            "quantity".to_string(),
        ];
        if self.include_non_existent {
            report_result_required_input.push("non-existent".to_string());
        }

        let report_result = g.add_node(
            "report_result",
            report_result_required_input,
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
        WorkflowGraphExt::new(g, report_result)
    }
}
