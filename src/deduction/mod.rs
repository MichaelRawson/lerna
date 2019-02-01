pub mod axiom;

use std::collections::HashSet;
use unique::Id;

use crate::formula::Formula;
use crate::goal::Goal;
use crate::inference::*;
use crate::options::OPTIONS;
use crate::set::Set;
use crate::simplification::simplify_goal;

pub struct DeductionInfo {
    axioms: Id<Set<Formula>>,
}

impl DeductionInfo {
    pub fn new(_goal: &Goal, axioms: Id<Set<Formula>>) -> Self {
        Self { axioms }
    }
}

pub trait Deduction: Sync {
    fn deduce(&self, deductions: &mut HashSet<Inferred>, goal: &Goal, info: &DeductionInfo);
}

pub fn deductions(goal: &Goal, axioms: Id<Set<Formula>>) -> Vec<Vec<Goal>> {
    let info = DeductionInfo::new(goal, axioms);
    let mut inferred = HashSet::new();

    for deduction in &OPTIONS.deductions {
        deduction.deduce(&mut inferred, goal, &info);
    }

    inferred
        .iter()
        .map(|inferred| goal.apply_all(inferred))
        .map(|goals| goals.iter().cloned().map(simplify_goal).collect())
        .collect()
}
