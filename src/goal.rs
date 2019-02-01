use std::fmt;
use std::ops::Deref;
use unique::Id;

use crate::formula::Formula;
use crate::inference::{Inference, Inferred};
use crate::set::Set;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Goal(Id<Set<Formula>>);

impl Goal {
    pub fn new(refute: Id<Set<Formula>>) -> Self {
        Goal(refute)
    }

    pub fn is_trivial(&self) -> bool {
        self.0.contains(&Id::new(Formula::F))
    }

    pub fn apply(&self, inference: &Inference) -> Self {
        Self::new(Id::new(
            self.0.difference(&inference.remove).union(&inference.add),
        ))
    }

    pub fn apply_all(&self, inferred: &Inferred) -> Vec<Self> {
        inferred.inferences.iter().map(|i| self.apply(i)).collect()
    }
}

impl Deref for Goal {
    type Target = Set<Formula>;

    fn deref(&self) -> &Set<Formula> {
        &*self.0
    }
}

impl fmt::Debug for Goal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
