pub mod contradiction;

use crate::goal::Goal;
use crate::inference::Inference;
use crate::options::OPTIONS;

pub struct SimplificationInfo;

impl SimplificationInfo {
    pub fn new(_goal: &Goal) -> Self {
        SimplificationInfo
    }
}

pub trait Simplification: Sync {
    fn simplify(&self, goal: &Goal, info: &SimplificationInfo) -> Option<Inference>;
}

fn simplify_goal_pass(start: Goal, info: &SimplificationInfo) -> Goal {
    OPTIONS.simplifications.iter().fold(start, |g, s| {
        s.simplify(&g, info).map(|i| g.apply(&i)).unwrap_or(g)
    })
}

pub fn simplify_goal(mut original: Goal) -> Goal {
    let info = SimplificationInfo::new(&original);

    loop {
        let simplified = simplify_goal_pass(original.clone(), &info);
        if simplified == original {
            return original;
        }
        original = simplified;
    }
}
