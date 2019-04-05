use std::vec::Vec;
use unique::Id;

use crate::collections::IdSet;
use crate::formula::Formula;
use Formula::*;

fn boolean_propagation(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        Not(ref p) => match **p {
            T => Id::new(F),
            F => Id::new(T),
            _ => f.clone(),
        },
        Imp(ref p, ref q) => match (&**p, &**q) {
            (T, _) => q.clone(),
            (F, _) => Id::new(T),
            (_, T) => Id::new(T),
            (_, F) => Formula::negate(&p.clone()),
            _ => f.clone(),
        },
        And(ref ps) => {
            if ps.into_iter().any(|f| **f == F) {
                Id::new(F)
            } else {
                Id::new(And(ps
                    .into_iter()
                    .filter(|f| ***f != T)
                    .cloned()
                    .collect()))
            }
        }
        Or(ref ps) => {
            if ps.into_iter().any(|f| **f == T) {
                Id::new(T)
            } else {
                Id::new(Or(ps
                    .into_iter()
                    .filter(|f| ***f != F)
                    .cloned()
                    .collect()))
            }
        }
        Eqv(ref ps) => {
            if ps.into_iter().any(|f| **f == F) {
                Id::new(And(ps
                    .into_iter()
                    .map(|f| Formula::negate(f))
                    .collect()))
            } else {
                Id::new(Eqv(ps
                    .into_iter()
                    .filter(|f| ***f != T)
                    .cloned()
                    .collect()))
            }
        }
        _ => f.clone(),
    }
}

fn contradiction(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        Imp(ref p, ref q) => {
            if p == &Formula::negate(q) || q == &Formula::negate(p) {
                Id::new(F)
            } else {
                f.clone()
            }
        }
        And(ref ps) => {
            if ps.into_iter().any(|f| ps.contains(&Formula::negate(f))) {
                Id::new(F)
            } else {
                f.clone()
            }
        }
        Or(ref ps) => {
            if ps.into_iter().any(|f| ps.contains(&Formula::negate(f))) {
                Id::new(T)
            } else {
                f.clone()
            }
        }
        Eqv(ref ps) => {
            if ps.into_iter().any(|f| ps.contains(&Formula::negate(f))) {
                Id::new(F)
            } else {
                f.clone()
            }
        }
        _ => f.clone(),
    }
}

fn combine_equivalence_classes(f: &Id<Formula>) -> Id<Formula> {
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
            let eqvs =
                IdSet::combine_overlapping(ps.into_iter().filter_map(|f| {
                    match **f {
                        Eqv(ref ps) => Some(ps),
                        _ => None,
                    }
                }))
                .into_iter()
                .map(|class| Id::new(Eqv(class)));

            let neither = ps.into_iter().filter_map(|f| match **f {
                Eq(_) | Eqv(_) => None,
                _ => Some(f.clone()),
            });

            Id::new(And(eqs.chain(eqvs).chain(neither).collect()))
        }
        _ => f.clone(),
    }
}

fn combine_implications(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        And(ref ps) => {
            for p in ps.into_iter() {
                if let Imp(ref left, ref right) = **p {
                    let reverse = Id::new(Imp(right.clone(), left.clone()));
                    if ps.contains(&reverse) {
                        let eqv =
                            Id::new(Eqv(idset![left.clone(), right.clone()]));
                        return Id::new(And(ps
                            .without(&reverse)
                            .without(p)
                            .with(eqv)));
                    }
                }
            }
            f.clone()
        }
        _ => f.clone(),
    }
}

fn double_negation(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        Not(ref p) => match **p {
            Not(ref p) => p.clone(),
            _ => f.clone(),
        },
        _ => f.clone(),
    }
}

fn trivial_nary(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        Eq(ref ts) => {
            if ts.len() < 2 {
                Id::new(T)
            } else {
                f.clone()
            }
        }
        And(ref ps) => {
            if ps.is_empty() {
                Id::new(T)
            } else if ps.len() == 1 {
                ps.as_ref()[0].clone()
            } else {
                f.clone()
            }
        }
        Or(ref ps) => {
            if ps.is_empty() {
                Id::new(F)
            } else if ps.len() == 1 {
                ps.as_ref()[0].clone()
            } else {
                f.clone()
            }
        }
        Eqv(ref ps) => {
            if ps.len() < 2 {
                Id::new(T)
            } else {
                f.clone()
            }
        }
        _ => f.clone(),
    }
}

fn lift_associative(f: &Id<Formula>) -> Id<Formula> {
    match **f {
        And(ref ps) => {
            let (lift, keep) =
                ps.into_iter().partition::<Vec<_>, _>(|f| match ***f {
                    And(_) => true,
                    _ => false,
                });
            let lifted = lift.into_iter().flat_map(|f| match **f {
                And(ref ps) => ps.into_iter(),
                _ => unreachable!(),
            });
            let result = keep.into_iter().chain(lifted).cloned().collect();
            Id::new(And(result))
        }
        Or(ref ps) => {
            let (lift, keep) =
                ps.into_iter().partition::<Vec<_>, _>(|f| match ***f {
                    Or(_) => true,
                    _ => false,
                });
            let lifted = lift.into_iter().flat_map(|f| match **f {
                Or(ref ps) => ps.into_iter(),
                _ => unreachable!(),
            });
            let result = keep.into_iter().chain(lifted).cloned().collect();
            Id::new(Or(result))
        }
        _ => f.clone(),
    }
}

pub fn simplify_propositional(f: &Id<Formula>) -> Id<Formula> {
    let f = boolean_propagation(f);
    let f = contradiction(&f);
    let f = combine_equivalence_classes(&f);
    let f = combine_implications(&f);
    let f = double_negation(&f);
    let f = lift_associative(&f);
    trivial_nary(&f)
}
