use criterion::{criterion_group, criterion_main, Criterion};
use cs257_project::{
    example_graphs::{
        buy_sell_stock::BuySellStockGraph, linear::Linear, MakeGraph, WorkflowGraphExt,
    },
    verifier::GraphVerifier,
};
use z3::{Solver, StatisticsEntry};

/// return number of constraints
fn test_reachability_for_graph(
    graph_ext: &WorkflowGraphExt,
    get_statistics: bool,
) -> Vec<StatisticsEntry> {
    // test reachability
    let ctx = z3::Context::new(&z3::Config::default());
    let graph_verifier = GraphVerifier::new(&graph_ext.graph, &ctx);
    let solver = Solver::new(&ctx);
    let _ = graph_verifier.is_reachable_with_solver(graph_ext.test_reachable_node, &solver);
    let result = if get_statistics {
        solver.get_statistics().entries().collect()
    } else {
        vec![]
    };
    result
}

fn benchmark_for_graph<M: MakeGraph>(mk: &M, c: &mut Criterion) {
    let graph_ext = mk.make_graph();
    let name = mk.name();
    let statistics = test_reachability_for_graph(&graph_ext, true);
    c.bench_function(format!("{}-reachability", name).as_str(), |b| {
        b.iter(|| test_reachability_for_graph(&graph_ext, false))
    });
    println!("{}: {:?}", name, statistics);
}

fn benchmark(c: &mut Criterion) {
    benchmark_for_graph(&BuySellStockGraph::new(false), c);
    benchmark_for_graph(&Linear(20), c);
    benchmark_for_graph(&Linear(40), c);
    benchmark_for_graph(&Linear(60), c);
    benchmark_for_graph(&Linear(80), c);
    benchmark_for_graph(&Linear(100), c);
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
