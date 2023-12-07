use z3::{
    ast::{self, Array, Ast, Int, Real},
    Config, Context, FuncDecl, RecFuncDecl, SatResult, Solver, Sort,
};

#[test]
fn sanity_check() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let sort_int = Sort::int(&ctx);
    let f = FuncDecl::new(&ctx, "f", &[&sort_int], &sort_int);
    let x = Int::new_const(&ctx, "x");
    let y = Int::new_const(&ctx, "y");
    let a = Array::new_const(&ctx, "A", &sort_int, &sort_int);

    // fml = Implies(x + 2 == y, f(Store(A, x, 3)[y - 2]) == f(y - x + 1))
    let left = (&x + Int::from_i64(&ctx, 2))._eq(&y);
    let right = f
        .apply(&[&a
            .store(&x, &Int::from_i64(&ctx, 3))
            .select(&(&y - &Int::from_i64(&ctx, 2)))])
        ._eq(&f.apply(&[&(&(&y - &x) + &Int::from_i64(&ctx, 1))]));
    let fml = left.implies(&right);

    solver.assert(&fml.not());

    assert_eq!(solver.check(), SatResult::Unsat);
    println!("Model: {:?}", solver.get_model());
}

#[test]
fn test_real_cmp() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let x = Real::new_const(&ctx, "x");
    let sort_real = Sort::real(&ctx);
    let f = FuncDecl::new(&ctx, "f", &[&sort_real], &sort_real);
    let f_x = f.apply(&[&x]);
    // forall x, f(x) = x + 1
    let forall = ast::forall_const(
        &ctx,
        &[&x],
        &[],
        &(&x + &Real::from_real(&ctx, 1, 1))._eq(&f_x.as_real().unwrap()),
    );

    solver.assert(&forall);
    assert_eq!(solver.check(), SatResult::Sat);
    let model = solver.get_model().unwrap();
    println!("Model: {:?}", model);

    let f_100 = model
        .eval(
            &f.apply(&[&Real::from_int(&Int::from_i64(&ctx, 100))]),
            true,
        )
        .unwrap()
        .as_real()
        .unwrap()
        .as_real()
        .unwrap();
    println!("{:?}", f_100);
}

#[test]
fn test_rec_func_def() {
    let cfg = Config::new();

    let ctx = Context::new(&cfg);

    let fac = RecFuncDecl::new(&ctx, "fac", &[&Sort::int(&ctx)], &Sort::int(&ctx));
    let n = Int::new_const(&ctx, "n");
    let n_minus_1 = Int::sub(&ctx, &[&n, &ast::Int::from_i64(&ctx, 1)]);
    let fac_of_n_minus_1 = fac.apply(&[&n_minus_1]);
    let cond = n.le(&Int::from_i64(&ctx, 0));
    let body = cond.ite(
        &ast::Int::from_i64(&ctx, 1),
        &ast::Int::mul(&ctx, &[&n, &fac_of_n_minus_1.as_int().unwrap()]),
    );

    fac.add_def(&[&n], &body);

    let x = ast::Int::new_const(&ctx, "x");
    let y = ast::Int::new_const(&ctx, "y");

    let solver = Solver::new(&ctx);

    solver.assert(&x._eq(&fac.apply(&[&ast::Int::from_i64(&ctx, 4)]).as_int().unwrap()));
    solver.assert(&y._eq(&ast::Int::mul(&ctx, &[&ast::Int::from_i64(&ctx, 5), &x])));
    solver.assert(&y._eq(&fac.apply(&[&ast::Int::from_i64(&ctx, 5)]).as_int().unwrap()));
    solver.assert(&y._eq(&ast::Int::from_i64(&ctx, 120)));

    assert_eq!(solver.check(), SatResult::Sat);
    println!("Model: {:?}", solver.get_model());
}
