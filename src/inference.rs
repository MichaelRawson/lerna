use std::hash::{Hash, Hasher};
use std::vec::Vec;
use unique::Id;

use crate::formula::Formula;
use crate::set::Set;

pub struct Justification {
    pub rule: &'static str,
    pub parents: Vec<Id<Formula>>,
}

pub trait Explain {
    fn explain(&self) -> Justification;
}

pub struct Inference {
    pub add: Set<Formula>,
    pub remove: Set<Formula>,
    explanation: Box<dyn Explain>,
}

impl PartialEq for Inference {
    fn eq(&self, other: &Self) -> bool {
        self.add == other.add && self.remove == other.remove
    }
}

impl Eq for Inference {}

impl Hash for Inference {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.add.hash(hasher);
        self.remove.hash(hasher);
    }
}

impl Inference {
    pub fn new(add: Set<Formula>, remove: Set<Formula>, explanation: Box<dyn Explain>) -> Self {
        Inference {
            add,
            remove,
            explanation,
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Inferred {
    pub inferences: Vec<Inference>,
}

impl Inferred {
    pub fn new(inferences: Vec<Inference>) -> Self {
        Self { inferences }
    }
}
