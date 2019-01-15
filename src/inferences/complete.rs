use std::cmp::{max, min};

use unique::Uniq;

use crate::formula::{Formula, FormulaSet};
use crate::formula::Formula::*;
use crate::goal::Goal;
use crate::inferences::Inferred;
use crate::term::Term;
use crate::types::Set;

fn contradiction() -> Inferred {
    set![Goal::new(set![uniq!(F)])]
}

fn equality(goal: &Goal, left: Uniq<Term>, right: Uniq<Term>) -> Inferred {
    let smaller = min(left, right);
    let larger = max(left, right);
    let mut replaced: Set<_> = goal
        .formulae()
        .map(|f| Formula::replace(*f, smaller, larger))
        .collect();
    replaced.insert(Formula::eql(left, right));
    set![Goal::new(replaced)]
}

fn negated(goal: &Goal, p: Uniq<Formula>) -> Set<Inferred> {
    if goal.contains(p) {
        return set![contradiction()];
    }

    match *p {
        T => set![set![goal.with(Formula::f())]],
        F => set![set![goal.with(Formula::t())]],
        Eql(p, q) if p == q => set![contradiction()],
        Eql(_, _) => set![],
        Prd(_, _) => set![],
        Not(p) => set![set![goal.with(p)]],
        Imp(p, q) => set![set![goal.with(p).with(Formula::not(q))]],
        Eqv(p, q) => {
            let npimpq = Formula::not(uniq!(Imp(p, q)));
            let nqimpp = Formula::not(uniq!(Imp(q, p)));
            set![set![goal.with(npimpq), goal.with(nqimpp)]]
        }
        And(ref ps) => {
            let nps = uniq!(Or(uniq!(FormulaSet(ps.iter().cloned().map(Formula::not).collect()))));
            set![set![goal.with(nps)]]
        }
        Or(ref ps) => {
            let nps = uniq!(And(uniq!(FormulaSet(ps.iter().cloned().map(Formula::not).collect()))));
            set![set![goal.with(nps)]]
        }
        Ex(p) => goal
            .symbols()
            .map(|s| set![goal.with(Formula::not(Formula::instantiate_with_symbol(p, *s)))])
            .collect::<Set<_>>()
            .update(set![goal.with(Formula::not(Formula::instantiate_with_constant(p)))]),
        All(p) => set![set![goal.with(Formula::not(Formula::instantiate_with_constant(p)))]],
    }
}

fn formula_inferences(goal: &Goal, f: Uniq<Formula>) -> Set<Inferred> {
    match *f {
        T | F | Prd(_, _) => set![],
        Eql(p, q) => set![equality(goal, p, q)],
        Not(p) => negated(goal, p),
        Imp(p, q) => set![set![goal.with(Formula::not(p)), goal.with(q)]],
        Eqv(p, q) => {
            let pimpq = uniq!(Imp(p, q));
            let qimpp = uniq!(Imp(q, p));
            set![set![goal.with(pimpq).with(qimpp)]]
        }
        And(ps) => set![set![goal.with_many((**ps).clone())]],
        Or(ref ps) => ps.iter().map(|p| set![goal.with(p.clone())]).collect(),
        Ex(p) => set![set![goal.with(Formula::instantiate_with_constant(p))]],
        All(p) => goal
            .symbols()
            .map(|s| set![goal.with(Formula::instantiate_with_symbol(p, *s))])
            .collect::<Set<_>>()
            .update(set![goal.with(Formula::instantiate_with_constant(p))]),
    }
}

pub fn inferences(goal: &Goal) -> Set<Inferred> {
    Set::unions(goal.formulae().map(|f| formula_inferences(goal, *f)))
}
