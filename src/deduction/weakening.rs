use std::collections::HashSet;
use unique::Id;

use crate::collections::IdSet;
use crate::formula::Formula;

use Formula::*;

fn weaken(f: &Id<Formula>) -> IdSet<Formula> {
    match **f {
        T | F | Prd(_, _) | Not(_) | Imp(_, _) | Or(_) | All(_) | Ex(_) => {
            idset![]
        }
        Eq(ref ts) => {
            if ts.len() > 2 {
                ts.into_iter().map(|t| Id::new(Eq(ts.without(t)))).collect()
            } else {
                idset![]
            }
        }
        And(ref ps) => ps
            .into_iter()
            .map(|p| Id::new(And(ps.without(p))))
            .chain(ps.into_iter().flat_map(|p| {
                weaken(p)
                    .into_iter()
                    .map(move |q| Id::new(And(ps.without(p).with(q))))
            }))
            .collect(),
        Eqv(ref ps) => {
            if ps.len() > 2 {
                ps.into_iter()
                    .map(|p| Id::new(Eqv(ps.without(p))))
                    .collect()
            } else {
                idset![]
            }
        }
    }
}

pub fn weakening_deductions(
    deduced: &mut HashSet<IdSet<Formula>>,
    f: &Id<Formula>,
) {
    for f in &weaken(f) {
        deduced.insert(idset![f.clone()]);
    }
}
