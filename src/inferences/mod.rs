mod complete;

use core::*;
use simplifications::simplify;

pub type Inferred = Set<Goal>;

fn all_inferences(goal: &Goal) -> Set<Inferred> {
    complete::complete(goal)
}

fn inferences(goal: Goal) -> Set<Inferred> {
    let all = all_inferences(&goal);
    if all.is_empty() {
        trace!("no inferences for {:?}", goal);
        trace!("returning identity");
        set![set![goal]]
    } else {
        all
    }
}

pub fn infer(goal: Goal) -> Set<Inferred> {
    trace!("inferring from {:?}", goal);
    let result: Set<Inferred> = inferences(goal)
        .into_iter()
        .map(|inferred| inferred.into_iter().map(simplify).collect::<Inferred>())
        .collect();
    trace!("inferred: {:?}", result);
    result
}
