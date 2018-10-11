mod complete;

use collections::Set;
use goal::Goal;
use simplifications::simplify;

pub type Inferred = Set<Goal>;

fn all_inferences(goal: &Goal) -> Set<Inferred> {
    complete::inferences(goal)
}

fn inferences(goal: Goal) -> Set<Inferred> {
    let all = all_inferences(&goal);
    if all.is_empty() {
        set![set![goal]]
    } else {
        all
    }
}

pub fn infer(goal: Goal) -> Set<Inferred> {
    let result: Set<Inferred> = inferences(goal)
        .into_iter()
        .map(|inferred| inferred.into_iter().map(simplify).collect::<Inferred>())
        .collect();
    result
}
