use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec::Vec;

use formula::Formula;
use goal::Goal;

pub enum RawProof {
    Leaf,
    Branch(Goal, Vec<Box<RawProof>>),
}

impl RawProof {
    pub fn leaf(goal: &Goal) -> Self {
        assert!(goal.complete(), "proof leaves must have complete goals");
        RawProof::Leaf
    }

    pub fn branch(goal: Goal, children: Vec<Box<RawProof>>) -> Self {
        RawProof::Branch(goal, children)
    }

    pub fn reconstruct(self, start: Goal) -> ReconstructedProof {
        panic!("")
    }
}

pub struct ReconstructedProof {
    goal: Goal,
    justification: BTreeMap<Arc<Formula>, (&'static str, Vec<Arc<Formula>>)>,
    children: Vec<ReconstructedProof>
}

impl ReconstructedProof {
}
