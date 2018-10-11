use std::sync::Arc;

use collections::Set;
use formula::Formula;
use formula::Formula::*;
use goal::Goal;
use inferences::Inferred;

fn formula_inferences(goal: &Goal, f: &Arc<Formula>) -> Set<Inferred> {
    if goal.contains(&f.negated()) {
        return set![set![Goal::new(set![Arc::new(F)])]];
    }

    match **f {
        T | F | Prd(_, _) => set![],
        Eql(ref left, ref right) => set![
            set![goal.replace(left, right).with(f.clone())],
            set![goal.replace(right, left).with(f.clone())]
        ],
        Not(ref p) => match **p {
            T => set![set![goal.with(Arc::new(F))]],
            F => set![set![goal.with(Arc::new(T))]],
            Eql(ref p, ref q) => if p == q {
                set![set![goal.with(Arc::new(F))]]
            } else {
                set![]
            },
            Prd(_, _) => set![],
            Not(ref p) => set![set![goal.with(p.clone())]],
            Imp(ref p, ref q) => {
                let nq = q.negated();
                set![set![goal.with(p.clone()).with(nq)]]
            }
            Eqv(ref p, ref q) => {
                let npimpq = Imp(p.clone(), q.clone()).negated();
                let nqimpp = Imp(q.clone(), p.clone()).negated();
                set![set![goal.with(npimpq), goal.with(nqimpp)]]
            }
            And(ref ps) => {
                let nps = ps.iter().map(|x| x.negated()).collect();
                set![set![goal.with(Arc::new(Or(nps)))]]
            }
            Or(ref ps) => {
                let nps = ps.iter().map(|x| x.negated()).collect();
                set![set![goal.with(Arc::new(And(nps)))]]
            }
            Ex(ref p) => goal
                .symbols()
                .map(|s| set![goal.with(p.instantiate_with_symbol(*s).negated())])
                .collect::<Set<_>>()
                .update(set![goal.with(p.instantiate_with_constant().negated())]),
            All(ref p) => set![set![goal.with(p.instantiate_with_constant().negated())]],
        },
        Imp(ref p, ref q) => {
            let np = p.negated();
            set![set![goal.with(np), goal.with(q.clone())]]
        }
        Eqv(ref p, ref q) => {
            let pimpq = Arc::new(Imp(p.clone(), q.clone()));
            let qimpp = Arc::new(Imp(q.clone(), p.clone()));
            set![set![goal.with(pimpq).with(qimpp)]]
        }
        And(ref ps) => set![set![goal.with_many(ps.clone())]],
        Or(ref ps) => {
            if ps.len() < 2 {
                set![set![goal.with_many(ps.clone())]]
            } else {
                ps.iter()
                    .map(|p| set![goal.with(p.clone()), goal.with(Arc::new(Or(ps.without(p))))])
                    .collect()
            }
        }
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
