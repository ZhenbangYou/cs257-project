from workflow import Always, IdWithPrefix, WorkflowVerifier, DummyVerifier, OutputSchema, MatchesKey, MatchesValue

"""
- Check Stock Price
    - required_inputs: stock_name
    - output:
        - add_fixed(`stock_price`)
        - carry()

- Generate Buy/Sell Rec
    - inputs: stock_price
    - output:
        - add_fixed(`rec`)
        - carry()

- Buy/Sell
    - inputs: rec
    - output:
        - carry()
    - edges:
        - to "Sell Stock": `&[MatchesKey("rec"), MatchesValue("sell")]`,
        - to "Buy Stock": `&[MatchesKey("rec"), MatchesValue("buy")]`,

- Buy (assume output is `{previous_input: (previous_input)`)
    - inputs: stock_name, stock_price
    - output: 
        - `add_rule_for_every_input(IdWithPrefix("previous_input"), Always)`
"""

def check_stock_price_workflow(verifier: WorkflowVerifier):
    verifier.add_node("check_stock_price", [], OutputSchema()
                      .add_fixed("stock_price")
                      .carry_all())
    verifier.add_node("buy_sell_rec", ["stock_price"], OutputSchema()
                      .add_fixed("rec")
                      .carry_all())
    verifier.add_node("buy_or_sell", ["rec"], OutputSchema()
                        .carry_all())
    verifier.add_node("buy", ["stock_name", "stock_price"], OutputSchema()
                      .add_fixed("quantity")
                        .add_rule_for_every_input(IdWithPrefix("previous_input."), Always()))
    verifier.add_node("sell", ["stock_name"], OutputSchema()
                      .add_fixed("quantity")
                      .add_rule_for_every_input(IdWithPrefix("previous_input."), Always()))
    verifier.add_node("report_result", ["previous_input.stock_name",
                                         "previous_input.rec", "quantity"], OutputSchema()
                        .carry_all())
    
    verifier.add_edge("check_stock_price", "buy_sell_rec", [])
    verifier.add_edge("buy_sell_rec", "buy_or_sell", [])

    # branch: choose between buy and sell based on rec
    verifier.add_edge("buy_or_sell", "buy", [MatchesKey("rec"), MatchesValue("buy")])
    verifier.add_edge("buy_or_sell", "sell", [MatchesKey("rec"), MatchesValue("sell")])

    # go to report_result
    verifier.add_edge("buy", "report_result", [])
    verifier.add_edge("sell", "report_result", [])

    verifier.set_start_node("check_stock_price")


def main():
    verifier = DummyVerifier()
    check_stock_price_workflow(verifier)
    verifier.print_graph()

if __name__ == "__main__":
    main()