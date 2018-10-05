use std::sync::Arc;

use core::Goal;
use core::Formula::*;

pub fn contradiction(goal: Goal) -> Goal {
    for f in goal.formulae() {
        if goal.contains(&Not(f.clone())) {
            trace!("contradiction found for {:?}", f);
            return goal.with(Arc::new(F))
        }
    }
    goal
}
