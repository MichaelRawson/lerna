/*
use std::sync::Arc;

use formula::Formula::*;
use goal::Goal;

pub fn contradiction(goal: Goal) -> Goal {
    for f in goal.formulae() {
        if goal.contains(&Not(f.clone())) {
            return goal.with(Arc::new(F));
        }
    }
    goal
}
*/
