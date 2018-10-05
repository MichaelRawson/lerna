use std::sync::Arc;

use common::{instantiate, replace_in_goal};
use core::Formula::*;
use core::*;
use inferences::Inferred;

fn formula_inferences(goal: &Goal, f: &Arc<Formula>) -> Set<Inferred> {
    if goal.contains(&Formula::negate(f.clone())) {
        return set![set![Goal::new(set![Arc::new(F)])]];
    }

    match **f {
        T | F | Prd(_, _) => set![],
        Eql(ref left, ref right) => set![
            set![replace_in_goal(goal, left, right).with(f.clone())],
            set![replace_in_goal(goal, right, left).with(f.clone())]
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
                let nq = Formula::negate(q.clone());
                set![set![goal.with(p.clone()).with(nq)]]
            }
            Eqv(ref p, ref q) => {
                let npimpq = Formula::negate(Arc::new(Imp(p.clone(), q.clone())));
                let nqimpp = Formula::negate(Arc::new(Imp(q.clone(), p.clone())));
                set![set![goal.with(npimpq), goal.with(nqimpp)]]
            }
            And(ref ps) => {
                let nps = ps.iter().map(|x| Formula::negate(x.clone())).collect();
                set![set![goal.with(Arc::new(Or(nps)))]]
            }
            Or(ref ps) => {
                let nps = ps.iter().map(|x| Formula::negate(x.clone())).collect();
                set![set![goal.with(Arc::new(And(nps)))]]
            }
            All(x, ref p) => {
                let switched = Arc::new(Ex(x, Formula::negate(p.clone())));
                set![set![goal.with(switched)]]
            }
            Ex(x, ref p) => {
                let switched = Arc::new(All(x, Formula::negate(p.clone())));
                set![set![goal.with(switched)]]
            }
        },
        Imp(ref p, ref q) => {
            let np = Formula::negate(p.clone());
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
        Ex(x, ref p) => instantiate(x, goal, p)
            .into_iter()
            .map(|p| set![goal.with(p)])
            .collect(),
        All(x, ref p) => instantiate(x, goal, p)
            .into_iter()
            .map(|p| set![goal.with(p)])
            .collect(),
    }
}

pub fn inferences(goal: &Goal) -> Set<Inferred> {
    Set::unions(goal.formulae().map(|f| formula_inferences(goal, f)))
}
