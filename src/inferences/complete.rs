use std::cmp::{max, min};
use crate::types::Dag;

use crate::formula::Formula;
use crate::formula::Formula::*;
use crate::goal::Goal;
use crate::inferences::Inferred;
use crate::term::Term;
use crate::types::Set;

fn contradiction() -> Inferred {
    set![Goal::new(set![dag!(F)])]
}

fn equality(goal: &Goal, left: &Dag<Term>, right: &Dag<Term>) -> Inferred {
    let smaller = min(left, right);
    let larger = max(left, right);
    let mut replaced: Set<_> = goal
        .formulae()
        .map(|f| f.replace(smaller, larger))
        .collect();
    replaced.insert(dag!(Eql(left.clone(), right.clone())));
    set![Goal::new(replaced)]
}

fn negated(goal: &Goal, p: &Dag<Formula>) -> Set<Inferred> {
    if goal.contains(p) {
        return set![contradiction()];
    }

    match **p {
        T => set![set![goal.with(dag!(F))]],
        F => set![set![goal.with(dag!(T))]],
        Eql(ref p, ref q) if p == q => set![contradiction()],
        Eql(_, _) => set![],
        Prd(_, _) => set![],
        Not(ref p) => set![set![goal.with(p.clone())]],
        Imp(ref p, ref q) => set![set![goal.with(p.clone()).with(q.negated())]],
        Eqv(ref p, ref q) => {
            let npimpq = Imp(p.clone(), q.clone()).negated();
            let nqimpp = Imp(q.clone(), p.clone()).negated();
            set![set![goal.with(npimpq), goal.with(nqimpp)]]
        }
        And(ref ps) => {
            let nps = dag!(Or(ps.iter().map(|x| x.negated()).collect()));
            set![set![goal.with(nps)]]
        }
        Or(ref ps) => {
            let nps = dag!(And(ps.iter().map(|x| x.negated()).collect()));
            set![set![goal.with(nps)]]
        }
        Ex(ref p) => goal
            .symbols()
            .map(|s| set![goal.with(p.instantiate_with_symbol(*s).negated())])
            .collect::<Set<_>>()
            .update(set![goal.with(p.instantiate_with_constant().negated())]),
        All(ref p) => set![set![goal.with(p.instantiate_with_constant().negated())]],
    }
}

fn formula_inferences(goal: &Goal, f: &Dag<Formula>) -> Set<Inferred> {
    match **f {
        T | F | Prd(_, _) => set![],
        Eql(ref p, ref q) => set![equality(goal, p, q)],
        Not(ref p) => negated(goal, p),
        Imp(ref p, ref q) => set![set![goal.with(p.negated()), goal.with(q.clone())]],
        Eqv(ref p, ref q) => {
            let pimpq = dag!(Imp(p.clone(), q.clone()));
            let qimpp = dag!(Imp(q.clone(), p.clone()));
            set![set![goal.with(pimpq).with(qimpp)]]
        }
        And(ref ps) => set![set![goal.with_many(ps.clone())]],
        Or(ref ps) => ps.iter().map(|p| set![goal.with(p.clone())]).collect(),
        Ex(ref p) => set![set![goal.with(p.instantiate_with_constant())]],
        All(ref p) => goal
            .symbols()
            .map(|s| set![goal.with(p.instantiate_with_symbol(*s))])
            .collect::<Set<_>>()
            .update(set![goal.with(p.instantiate_with_constant())]),
    }
}

pub fn inferences(goal: &Goal) -> Set<Inferred> {
    Set::unions(goal.formulae().map(|f| formula_inferences(goal, f)))
}
