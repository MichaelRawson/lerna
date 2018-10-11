use std::sync::Arc;
use std::vec::Vec;

use formula::Formula;
use goal::Goal;

pub enum RawProof {
    Leaf,
    Branch(Goal, Vec<Box<RawProof>>),
}

pub type Proof = Vec<(Arc<Formula>, Vec<usize>, &'static str)>;

impl RawProof {
    pub fn leaf(goal: &Goal) -> Self {
        assert!(goal.complete(), "proof leaves must have complete goals");
        RawProof::Leaf
    }

    pub fn branch(goal: Goal, children: Vec<Box<RawProof>>) -> Self {
        RawProof::Branch(goal, children)
    }

    pub fn reconstruct(self) -> Proof {
        panic!("")
    }
}
