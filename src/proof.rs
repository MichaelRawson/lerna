use std::vec::Vec;

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
}
