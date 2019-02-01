use unique::Id;

use crate::formula::*;
use crate::inference::*;
use crate::set::Set;
use crate::simplification::*;

struct Explanation {
    formula: Id<Formula>,
    contradiction: Id<Formula>,
}

impl Explain for Explanation {
    fn explain(&self) -> Justification {
        Justification {
            rule: "contradiction",
            parents: vec![self.formula.clone(), self.contradiction.clone()],
        }
    }
}

pub struct Contradiction;

impl Simplification for Contradiction {
    fn simplify(&self, goal: &Goal, _info: &SimplificationInfo) -> Option<Inference> {
        use self::Formula::{Not, F};

        for f in goal.into_iter().cloned() {
            let not_f = Id::new(Not(f.clone()));
            if goal.contains(&not_f) {
                let false_ = Id::new(F);
                let justification = Box::new(Explanation {
                    formula: f,
                    contradiction: not_f,
                });
                let inference = Inference::new(set![false_], set![], justification);
                return Some(inference);
            }
        }

        None
    }
}
