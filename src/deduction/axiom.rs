use crate::deduction::*;
use crate::formula::Formula;
use crate::inference::Justification;

struct Explanation;

impl Explain for Explanation {
    fn explain(&self) -> Justification {
        Justification {
            rule: "axiom",
            parents: vec![],
        }
    }
}

fn infer_axiom(axiom: Id<Formula>) -> Inferred {
    let explanation = Box::new(Explanation);
    let inference = Inference::new(set![axiom], set![], explanation);
    Inferred::new(vec![inference])
}

pub struct Axiom;

impl Deduction for Axiom {
    fn deduce(&self, deductions: &mut HashSet<Inferred>, goal: &Goal, info: &DeductionInfo) {
        deductions.extend(
            info.axioms
                .into_iter()
                .filter(|f| !goal.contains(*f))
                .map(|f| infer_axiom(f.clone())),
        );
    }
}
