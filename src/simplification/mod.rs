mod propositional;
mod equational;

use unique::Id;

use crate::formula::Formula;
use crate::system::check_for_timeout;

fn simplify_children(f: &Id<Formula>) -> Id<Formula> {
    use Formula::*;
    match **f {
        T | F | Prd(_, _) | Eq(_) => f.clone(),
        Not(ref p) => Id::new(Not(simplify(p))),
        Imp(ref p, ref q) => Id::new(Imp(simplify(p), simplify(q))),
        And(ref ps) => Id::new(And(ps.into_iter().map(simplify).collect())),
        Or(ref ps) => Id::new(Or(ps.into_iter().map(simplify).collect())),
        Eqv(ref ps) => Id::new(Eqv(ps.into_iter().map(simplify).collect())),
        All(ref p) => Id::new(All(simplify(p))),
        Ex(ref p) => Id::new(Ex(simplify(p))),
    }
}

fn simplify_step(f: &Id<Formula>) -> Id<Formula> {
    let f = simplify_children(f);
    let f = propositional::simplify_propositional(&f);
    equational::simplify_equational(&f)
}

pub fn simplify(f: &Id<Formula>) -> Id<Formula> {
    let mut f = f.clone();

    loop {
        let simplified = simplify_step(&f);
        check_for_timeout();
        if f == simplified {
            return f;
        }
        f = simplified;
    }
}
