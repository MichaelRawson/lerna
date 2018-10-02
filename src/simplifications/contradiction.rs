use std::sync::Arc;

use core::{Formula, Goal};

pub fn contradiction(goal: Goal) -> Goal {
    for f in &goal.formulae {
        if goal.formulae.contains(&Formula::Not(f.clone())) {
            trace!("contradiction found for {:?}", f);
            let false_ = Arc::new(Formula::F);
            let formulae = set![false_.clone()];
            return Goal { formulae };
        }
    }
    goal
}
