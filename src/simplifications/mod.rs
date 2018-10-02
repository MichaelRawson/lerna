mod contradiction;

use core::Goal;

fn simplify_step(goal: Goal) -> Goal {
    let goal = contradiction::contradiction(goal);
    goal
}

pub fn simplify(goal: Goal) -> Goal {
    trace!("simplify {:?}", goal.formulae);
    let mut current = goal;
    loop {
        let next = simplify_step(current.clone());
        if current == next {
            break;
        }
        current = next;
    }
    trace!("simplified to {:?}", current.formulae);

    current
}
