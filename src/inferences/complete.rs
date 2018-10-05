use std::sync::Arc;

use core::Formula::*;
use core::*;
use inferences::Inferred;

fn propositional(goal: Goal, f: &Arc<Formula>) -> Inferred {
    match **f {
        Not(ref p) => match **p {
            T => set![goal.with(Arc::new(F))],
            F => set![goal.with(Arc::new(T))],
            Not(ref p) => set![goal.with(p.clone())],
            Imp(ref p, ref q) => {
                let nq = Formula::negate(q.clone());
                set![goal.with(p.clone()).with(nq)]
            }
            Eqv(ref p, ref q) => {
                let npimpq = Formula::negate(Arc::new(Imp(p.clone(), q.clone())));
                let nqimpp = Formula::negate(Arc::new(Imp(q.clone(), p.clone())));
                set![goal.with(npimpq), goal.with(nqimpp)]
            }
            ref _other => set![goal],
        },
        Imp(ref p, ref q) => {
            let np = Formula::negate(p.clone());
            set![goal.with(np), goal.with(q.clone())]
        }
        Eqv(ref p, ref q) => {
            let pimpq = Arc::new(Imp(p.clone(), q.clone()));
            let qimpp = Arc::new(Imp(p.clone(), q.clone()));
            set![goal.with(pimpq).with(qimpp)]
        }
        ref _other => set![goal],
    }
}

pub fn complete(goal: &Goal) -> Set<Inferred> {
    let mut inferences = set![];

    for f in goal.formulae() {
        inferences.insert(propositional((*goal).clone(), f));
    }

    inferences
}
