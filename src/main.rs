use z3::{Solver, Context, Config, ast::Bool};



fn main() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let tie = Bool::new_const(&ctx, "tie");
    let shirt = Bool::new_const(&ctx, "shirt");

    solver.assert(
        &Bool::or(&ctx, &[&tie, &shirt])
    );
    solver.assert(
        &tie.implies(&shirt.not())
    );
    solver.assert(
        &tie.implies(&shirt)
    );

    println!("Solver: {:?}", solver.check());
    println!("Model: {:?}", solver.get_model());
}
