use crate::goal::Goal;

fn simplify_step(goal: &Goal) -> Goal {
    goal.clone()
}

pub fn simplify(goal: &Goal) -> Goal {
    let mut current = goal.clone();
    loop {
        let next = simplify_step(&current);
        if current == next {
            break;
        }
        current = next;
    }

    current
}
