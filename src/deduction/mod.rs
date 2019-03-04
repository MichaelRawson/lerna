pub mod axiom;

use unique::Id;

use crate::collections::{list, List};
use crate::goal::{Goal, GoalInfo};
use crate::inference::Inference;
use crate::justification::Justification;
use crate::options::OPTIONS;

pub type Deduced = (Justification, List<Inference>);

pub trait Deduction: Send + Sync {
    fn deduce(&self, goal: &Id<Goal>, info: &GoalInfo) -> List<Deduced>;
}

pub fn deductions(goal: &Id<Goal>, info: &GoalInfo) -> List<Deduced> {
    let mut deduced = list![];

    for d in &OPTIONS.deductions {
        deduced.extend(d.deduce(goal, info));
    }

    deduced
}

mod prelude {
    pub use smallvec::smallvec;
    pub use unique::Id;

    pub use crate::collections::{list, List, Set};
    pub use crate::deduction::{Deduced, Deduction};
    pub use crate::goal::{Goal, GoalInfo};
    pub use crate::inference::Inference;
    pub use crate::justification::Justification;
}
