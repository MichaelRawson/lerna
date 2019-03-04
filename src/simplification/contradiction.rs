use crate::simplification::prelude::*;

pub struct Contradiction;

impl Simplification for Contradiction {
    fn simplify(
        &self,
        goal: &Id<Goal>,
        _info: &GoalInfo,
    ) -> Option<Simplified> {
        for f in &***goal {
            let not_f = Formula::negate(f);
            if goal.contains(&not_f) {
                let justification = Justification {
                    rule: "contradiction",
                    parents: list![f.clone(), not_f].into(),
                };
                let inference = Inference {
                    add: set![FALSE.clone()],
                    remove: set![],
                };
                return Some((justification, inference));
            }
        }

        None
    }
}
