mod complete;

use crate::goal::Goal;
use crate::simplifications::simplify;
use crate::types::Set;

pub type Inferred = Set<Goal>;

fn all_inferences(goal: &Goal) -> Set<Inferred> {
    complete::inferences(goal)
}

fn inferences(goal: Goal) -> Set<Inferred> {
    let mut all = all_inferences(&goal);
    all.insert(set![goal]); //ensure never empty
    all
}

pub fn infer(goal: Goal) -> Set<Inferred> {
    let result: Set<Inferred> = inferences(goal)
        .into_iter()
        .map(|inferred| inferred.iter().map(simplify).collect::<Inferred>())
        .collect();
    result
}
