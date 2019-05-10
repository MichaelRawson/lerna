use unique::Id;

use crate::collections::IdSet;
use crate::formula::Formula;
use Formula::*;

fn trivial_equality(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        Eq(ref ts) => {
            if ts.len() <= 1 {
                Id::new(T)
            } else {
                f.clone()
            }
        }
        _ => f.clone(),
    }
}

fn combine_equations(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        And(ref ps) => {
            let eqs =
                IdSet::combine_overlapping(ps.into_iter().filter_map(|f| {
                    match **f {
                        Eq(ref ts) => Some(ts),
                        _ => None,
                    }
                }))
                .into_iter()
                .map(|class| Id::new(Eq(class)));

            let rest = ps.into_iter().filter_map(|f| match **f {
                Eq(_) => None,
                _ => Some(f.clone()),
            });

            Id::new(And(eqs.chain(rest).collect()))
        }
        _ => f.clone(),
    }
}

pub fn simplify_equational(f: &Id<Formula>) -> Id<Formula> {
    let f = trivial_equality(f);
    combine_equations(&f)
}
