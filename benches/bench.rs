use criterion::{criterion_group, criterion_main, Criterion};
use cs257_project::{
    example_graphs::{buy_sell_stock::BuySellStock, MakeGraph, WorkflowGraphExt},
    verifier::GraphVerifier,
};
use z3::Solver;

/// return number of constraints
fn test_reachability_for_graph(graph_ext: &WorkflowGraphExt) -> usize {
    // test reachability
    let ctx = z3::Context::new(&z3::Config::default());
    let graph_verifier = GraphVerifier::new(&graph_ext.graph, &ctx);
    let solver = Solver::new(&ctx);
    let _ = graph_verifier.is_reachable_with_solver(graph_ext.test_reachable_node, &solver);
    let num_constraints = solver.get_assertions().len();
    num_constraints
}

fn benchmark_for_graph<M: MakeGraph>(mk: M, c: &mut Criterion) {
    let graph_ext = mk.make_graph();
    let name = M::NAME;
    let num_constraints = test_reachability_for_graph(&graph_ext);
    c.bench_function(format!("{}-reachability", name).as_str(), |b| {
        b.iter(|| test_reachability_for_graph(&graph_ext))
    });
    println!("{}: {} constraints", name, num_constraints);
}

fn benchmark(c: &mut Criterion) {
    benchmark_for_graph(BuySellStock::new(false), c);
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
