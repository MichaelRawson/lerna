use std::fmt;
use std::ops::Deref;
use unique::allocators::HashAllocator;
use unique::{make_allocator, Id};

use crate::collections::Set;
use crate::formula::Formula;
use crate::inference::Inference;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Goal(Set<Formula>);
make_allocator!(Goal, GOAL_ALLOC, HashAllocator);

impl Goal {
    pub fn new(refute: Set<Formula>) -> Self {
        Goal(refute)
    }

    pub fn is_trivial(goal: &Id<Self>) -> bool {
        goal.contains(&Id::new(Formula::F))
    }

    pub fn apply(goal: &Id<Self>, inference: &Inference) -> Id<Self> {
        let refute = goal.difference(&inference.remove).union(&inference.add);
        Id::new(Goal::new(refute))
    }
}

impl Deref for Goal {
    type Target = Set<Formula>;

    fn deref(&self) -> &Set<Formula> {
        &self.0
    }
}

impl fmt::Debug for Goal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct GoalInfo {
    pub axioms: Set<Formula>,
}

impl GoalInfo {
    pub fn new(_goal: &Goal, axioms: &Set<Formula>) -> Self {
        let axioms = axioms.clone();

        Self { axioms }
    }
}
