use crate::deduction::prelude::*;

pub struct Axiom;

impl Deduction for Axiom {
    fn deduce(&self, goal: &Id<Goal>, info: &GoalInfo) -> List<Deduced> {
        let mut deduced = list![];
        for axiom in &info.axioms {
            if !goal.contains(axiom) {
                let justification = Justification {
                    rule: "axiom",
                    parents: set![],
                };
                let inference = Inference {
                    add: set![axiom.clone()],
                    remove: set![],
                };

                deduced.push((justification, list![inference]));
            }
        }

        deduced
    }
}
