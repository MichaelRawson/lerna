use std::vec::Vec;

use goal::Goal;

pub struct Proof {
    pub goal: Goal,
    pub children: Vec<Proof>,
}

impl Proof {
    pub fn leaf(goal: Goal) -> Self {
        assert!(goal.complete(), "proof leaves must have complete goals");
        Proof {
            goal,
            children: vec![],
        }
    }

    pub fn branch(goal: Goal, children: Vec<Proof>) -> Self {
        Proof { goal, children }
    }
}
