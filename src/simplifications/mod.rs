mod contradiction;

use core::Goal;

fn simplify_step(goal: Goal) -> Goal {
    //contradiction::contradiction(goal)
    goal
}

pub fn simplify(goal: Goal) -> Goal {
    let mut current = goal;
    loop {
        let next = simplify_step(current.clone());
        if current == next {
            break;
        }
        current = next;
    }

    current
}
