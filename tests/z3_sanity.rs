use z3::{
    ast::{Array, Ast, Int},
    Config, Context, FuncDecl, Solver, Sort,
};

#[test]
fn sanity_check() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let sort_z = Sort::int(&ctx);
    let f = FuncDecl::new(&ctx, "f", &[&sort_z], &sort_z);
    let x = Int::new_const(&ctx, "x");
    let y = Int::new_const(&ctx, "y");
    let _z = Int::new_const(&ctx, "z");
    let a = Array::new_const(&ctx, "A", &sort_z, &sort_z);

    // fml = Implies(x + 2 == y, f(Store(A, x, 3)[y - 2]) == f(y - x + 1))
    let left = (&x + Int::from_i64(&ctx, 2))._eq(&y);
    let right = f
        .apply(&[&a
            .store(&x, &Int::from_i64(&ctx, 3))
            .select(&(&y - &Int::from_i64(&ctx, 2)))])
        ._eq(&f.apply(&[&(&(&y - &x) + &Int::from_i64(&ctx, 1))]));
    let fml = left.implies(&right);

    solver.assert(&fml.not());

    println!("Solver: {:?}", solver.check());
    println!("Model: {:?}", solver.get_model());
}
