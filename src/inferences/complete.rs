use std::cmp::{max, min};

use unique::Uniq;

use crate::formula::Formula;
use crate::formula::Formula::*;
use crate::goal::Goal;
use crate::inferences::Inferred;
use crate::term::Term;
use crate::types::Set;

fn contradiction() -> Inferred {
    set![Goal::new(set![Uniq::new(F)])]
}

fn equality(goal: &Goal, left: Uniq<Term>, right: Uniq<Term>) -> Inferred {
    let smaller = min(left, right);
    let larger = max(left, right);
    let mut replaced: Set<_> = goal
        .formulae()
        .map(|f| f.replace(smaller, larger))
        .collect();
    replaced.insert(Uniq::new(Eql(left, right)));
    set![Goal::new(replaced)]
}

fn negated(goal: &Goal, p: Uniq<Formula>) -> Set<Inferred> {
    if goal.contains(p) {
        return set![contradiction()];
    }

    match *p {
        T => set![set![goal.with(Uniq::new(F))]],
        F => set![set![goal.with(Uniq::new(T))]],
        Eql(p, q) if p == q => set![contradiction()],
        Eql(_, _) => set![],
        Prd(_, _) => set![],
        Not(p) => set![set![goal.with(p)]],
        Imp(p, q) => set![set![goal.with(p).with(Formula::negate(q))]],
        Eqv(p, q) => {
            let npimpq = Formula::negate(Uniq::new(Imp(p, q)));
            let nqimpp = Formula::negate(Uniq::new(Imp(q, p)));
            set![set![goal.with(npimpq), goal.with(nqimpp)]]
        }
        And(ref ps) => {
            let nps = Uniq::new(Or(ps.iter().cloned().map(Formula::negate).collect()));
            set![set![goal.with(nps)]]
        }
        Or(ref ps) => {
            let nps = Uniq::new(And(ps.iter().cloned().map(Formula::negate).collect()));
            set![set![goal.with(nps)]]
        }
        Ex(p) => goal
            .symbols()
            .map(|s| set![goal.with(Formula::negate(p.instantiate_with_symbol(*s)))])
            .collect::<Set<_>>()
            .update(set![goal.with(Formula::negate(p.instantiate_with_constant()))]),
        All(p) => set![set![goal.with(Formula::negate(p.instantiate_with_constant()))]],
    }
}

fn formula_inferences(goal: &Goal, f: Uniq<Formula>) -> Set<Inferred> {
    match *f {
        T | F | Prd(_, _) => set![],
        Eql(p, q) => set![equality(goal, p, q)],
        Not(p) => negated(goal, p),
        Imp(p, q) => set![set![goal.with(Formula::negate(p)), goal.with(q)]],
        Eqv(p, q) => {
            let pimpq = Uniq::new(Imp(p, q));
            let qimpp = Uniq::new(Imp(q, p));
            set![set![goal.with(pimpq).with(qimpp)]]
        }
        And(ref ps) => set![set![goal.with_many(ps.clone())]],
        Or(ref ps) => ps.iter().map(|p| set![goal.with(p.clone())]).collect(),
        Ex(p) => set![set![goal.with(p.instantiate_with_constant())]],
        All(p) => goal
            .symbols()
            .map(|s| set![goal.with(p.instantiate_with_symbol(*s))])
            .collect::<Set<_>>()
            .update(set![goal.with(p.instantiate_with_constant())]),
    }
}

pub fn inferences(goal: &Goal) -> Set<Inferred> {
    Set::unions(goal.formulae().map(|f| formula_inferences(goal, *f)))
}
