use std::collections::HashSet;
use unique::Id;

use crate::collections::IdSet;
use crate::formula::Formula;
use crate::symbol::Symbol;

use Formula::*;

fn introduced(f: &Id<Formula>) -> Id<Symbol> {
    Id::new(Symbol::Introduced(Id::id(f)))
}

fn complete(
    deduced: &mut HashSet<IdSet<Formula>>,
    symbols: &HashSet<(&Id<Symbol>, usize)>,
    f: &Id<Formula>,
) {
    match **f {
        T | F | Prd(_, _) | Eq(_) => {}
        Not(ref f) => match **f {
            T => {
                deduced.insert(idset![Id::new(F)]);
            }
            F => {
                deduced.insert(idset![Id::new(F)]);
            }
            Prd(_, _) => {}
            Eq(ref ts) => {
                if ts.len() > 2 {
                    deduced.insert(
                        ts.pairs()
                            .map(|(t, s)| {
                                Id::new(Formula::Not(Id::new(Eq(idset![
                                    t.clone(),
                                    s.clone()
                                ]))))
                            })
                            .collect(),
                    );
                }
            }
            Not(ref f) => {
                deduced.insert(idset![f.clone()]);
            }
            Imp(ref p, ref q) => {
                deduced.insert(idset![Id::new(And(idset![
                    p.clone(),
                    Formula::negate(q)
                ]))]);
            }
            Or(ref ps) => {
                deduced.insert(idset![Id::new(And(ps
                    .into_iter()
                    .map(Formula::negate)
                    .collect()))]);
            }
            And(ref ps) => {
                deduced.insert(ps.into_iter().map(Formula::negate).collect());
            }
            Eqv(ref ps) => {
                deduced.insert(
                    ps.pairs()
                        .flat_map(|(p, q)| {
                            std::iter::once((p, q))
                                .chain(std::iter::once((q, p)))
                                .map(|(p, q)| {
                                    Id::new(And(idset![
                                        p.clone(),
                                        Formula::negate(q)
                                    ]))
                                })
                        })
                        .collect(),
                );
            }
            All(ref p) => {
                deduced.insert(idset![Id::new(Ex(Formula::negate(p)))]);
            }
            Ex(ref p) => {
                deduced.insert(idset![Id::new(All(Formula::negate(p)))]);
            }
        },
        Imp(ref p, ref q) => {
            deduced.insert(idset![Formula::negate(p), q.clone()]);
        }
        Or(ref ps) => {
            deduced.insert(ps.clone());
        }
        And(ref ps) => {
            for p in ps {
                let mut subdeductions = HashSet::new();
                complete(&mut subdeductions, &symbols, p);
                for sd in subdeductions {
                    let background: IdSet<Formula> = ps.without(p);
                    let combined: IdSet<Formula> = sd
                        .into_iter()
                        .map(|f| Id::new(And(background.with(f))))
                        .collect();
                    deduced.insert(combined);
                }
            }
        }
        Eqv(ref ps) => {
            for (p, q) in ps.pairs() {
                let rest = Id::new(Eqv(ps.without(p)));
                let positive =
                    Id::new(And(idset![rest.clone(), p.clone(), q.clone()]));
                let negative = Id::new(And(idset![
                    rest,
                    Formula::negate(p),
                    Formula::negate(q)
                ]));
                deduced.insert(idset![positive, negative]);
            }
        }
        All(ref p) => {
            for (symbol, arity) in symbols {
                let mut instantiated = Formula::subst(p, 0, symbol, *arity);
                for _ in 0..*arity {
                    instantiated = Id::new(All(instantiated))
                }
                let combined = Id::new(And(idset![f.clone(), instantiated]));
                deduced.insert(idset![combined]);
            }
            let intro = introduced(p);
            let instantiated = Formula::subst(p, 0, &intro, 0);
            deduced.insert(idset![instantiated]);
        }
        Ex(ref p) => {
            let intro = introduced(p);
            let instantiated = Formula::subst(p, 0, &intro, 0);
            deduced.insert(idset![instantiated]);
        }
    }
}

pub fn complete_deductions(
    deduced: &mut HashSet<IdSet<Formula>>,
    f: &Id<Formula>,
) {
    complete(deduced, &Formula::function_symbols(f), f);
}
