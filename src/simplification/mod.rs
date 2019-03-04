pub mod contradiction;

use unique::Id;

use crate::collections::{list, List};
use crate::goal::{Goal, GoalInfo};
use crate::inference::Inference;
use crate::justification::Justification;
use crate::options::OPTIONS;

pub type Simplified = (Justification, Inference);

pub trait Simplification: Send + Sync {
    fn simplify(&self, goal: &Id<Goal>, info: &GoalInfo) -> Option<Simplified>;
}

pub fn simplify(
    goal: &Id<Goal>,
    info: &GoalInfo,
) -> (Id<Goal>, List<Justification>) {
    let mut goal = goal.clone();
    let mut log = list![];
    let mut fresh = true;

    while fresh {
        fresh = false;
        for s in &OPTIONS.simplifications {
            if let Some((justification, inference)) = s.simplify(&goal, info) {
                let new = Goal::apply(&goal, &inference);
                fresh |= new != goal;
                goal = new;
                log.push(justification);
            }
        }
    }

    (goal, log)
}

mod prelude {
    pub use smallvec::smallvec;
    pub use unique::Id;

    pub use crate::collections::{list, List, Set};
    pub use crate::formula::{Formula, FALSE};
    pub use crate::goal::{Goal, GoalInfo};
    pub use crate::inference::Inference;
    pub use crate::justification::Justification;
    pub use crate::simplification::{Simplification, Simplified};
}
