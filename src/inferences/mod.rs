use core::*;
use simplifications::simplify;

pub type Inferred = Set<Goal>;

fn inferences(goal: Goal) -> Set<Inferred> {
    set![set![goal]]
}

pub fn infer(goal: Goal) -> Set<Inferred> {
    inferences(goal).into_iter()
        .map(|inferred| inferred.into_iter()
             .map(|x| simplify(x))
             .collect::<Inferred>())
        .collect()
}
