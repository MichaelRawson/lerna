use std::sync::Arc;

use core::Formula::*;
use core::*;
use inferences::Inferred;

fn propositional(goal: Goal, f: &Arc<Formula>) -> Inferred {
    let formulae = goal.formulae;
    match **f {
        Not(ref p) => match **p {
            Not(ref p) => set![Goal::new(formulae.update(p.clone()))],
            Imp(ref p, ref q) => {
                let nq = Formula::negate(q.clone());
                set![Goal::new(formulae.update(p.clone()).update(nq))]
            }
            Eqv(ref p, ref q) => {
                let npimpq = Formula::negate(Arc::new(Imp(p.clone(), q.clone())));
                let nqimpp = Formula::negate(Arc::new(Imp(q.clone(), p.clone())));
                set![
                    Goal::new(formulae.update(npimpq)),
                    Goal::new(formulae.update(nqimpp))
                ]
            }
            ref _other => set![Goal::new(formulae)],
        },
        Imp(ref p, ref q) => {
            let np = Formula::negate(p.clone());
            set![
                Goal::new(formulae.update(np)),
                Goal::new(formulae.update(q.clone()))
            ]
        }
        Eqv(ref p, ref q) => {
            let pimpq = Arc::new(Imp(p.clone(), q.clone()));
            let qimpp = Arc::new(Imp(p.clone(), q.clone()));
            set![Goal::new(formulae.update(pimpq).update(qimpp))]
        }
        ref _other => set![Goal::new(formulae)],
    }
}

pub fn complete(goal: &Goal) -> Set<Inferred> {
    let mut inferences = set![];

    for f in &goal.formulae {
        inferences.insert(propositional((*goal).clone(), f));
    }

    inferences
}
