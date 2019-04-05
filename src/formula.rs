use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use unique::allocators::HashAllocator;
use unique::{make_allocator, Id};

use crate::collections::{IdList, IdSet};
use crate::symbol::Symbol;
use crate::term::Term;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Formula {
    T,
    F,
    Eq(IdSet<Term>),
    Prd(Id<Symbol>, IdList<Term>),
    Not(Id<Formula>),
    Imp(Id<Formula>, Id<Formula>),
    Or(IdSet<Formula>),
    And(IdSet<Formula>),
    Eqv(IdSet<Formula>),
    All(Id<Formula>),
    Ex(Id<Formula>),
}
make_allocator!(Formula, FORMULA_ALLOC, HashAllocator);
use self::Formula::*;

impl Formula {
    pub fn negate(f: &Id<Formula>) -> Id<Formula> {
        Id::new(Not(f.clone()))
    }

    pub fn subst(
        f: &Id<Formula>,
        index: usize,
        symbol: &Id<Symbol>,
        arity: usize,
    ) -> Id<Formula> {
        match **f {
            T | F => f.clone(),
            Eq(ref ts) => Id::new(Eq(ts
                .into_iter()
                .map(|t| Term::subst(t, index, symbol, arity))
                .collect())),
            Prd(ref p, ref ts) => Id::new(Prd(
                p.clone(),
                ts.into_iter()
                    .map(|t| Term::subst(t, index, symbol, arity))
                    .collect(),
            )),
            Not(ref p) => Id::new(Not(Self::subst(p, index, symbol, arity))),
            Imp(ref p, ref q) => Id::new(Imp(
                Self::subst(p, index, symbol, arity),
                Self::subst(q, index, symbol, arity),
            )),
            Or(ref ps) => Id::new(Or(ps
                .into_iter()
                .map(|p| Self::subst(p, index, symbol, arity))
                .collect())),
            And(ref ps) => Id::new(And(ps
                .into_iter()
                .map(|p| Self::subst(p, index, symbol, arity))
                .collect())),
            Eqv(ref ps) => Id::new(Eqv(ps
                .into_iter()
                .map(|p| Self::subst(p, index, symbol, arity))
                .collect())),
            All(ref p) => {
                Id::new(All(Self::subst(p, index + 1, symbol, arity)))
            }
            Ex(ref p) => Id::new(Ex(Self::subst(p, index + 1, symbol, arity))),
        }
    }

    pub fn replace(
        f: &Id<Formula>,
        from: &Id<Term>,
        to: &Id<Term>,
    ) -> Id<Formula> {
        match **f {
            T | F => f.clone(),
            Eq(ref ts) => Id::new(Eq(ts
                .into_iter()
                .map(|t| Term::replace(t, from, to))
                .collect())),
            Prd(ref p, ref ts) => Id::new(Prd(
                p.clone(),
                ts.iter().map(|t| Term::replace(t, from, to)).collect(),
            )),
            Not(ref p) => Id::new(Not(Self::replace(p, from, to))),
            Imp(ref p, ref q) => Id::new(Imp(
                Self::replace(p, from, to),
                Self::replace(q, from, to),
            )),
            Or(ref ps) => Id::new(Or(ps
                .into_iter()
                .map(|p| Self::replace(p, from, to))
                .collect())),
            And(ref ps) => Id::new(And(ps
                .into_iter()
                .map(|p| Self::replace(p, from, to))
                .collect())),
            Eqv(ref ps) => Id::new(Eqv(ps
                .into_iter()
                .map(|p| Self::replace(p, from, to))
                .collect())),
            All(ref p) => Id::new(All(Self::replace(p, from, to))),
            Ex(ref p) => Id::new(Ex(Self::replace(p, from, to))),
        }
    }

    pub fn breadth_first(
        f: &Id<Formula>,
    ) -> impl Iterator<Item = &Id<Formula>> {
        BreadthFirst::new(f)
    }

    pub fn breadth_first_terms(
        f: &Id<Formula>,
    ) -> impl Iterator<Item = &Id<Term>> {
        Self::breadth_first(f)
            .filter_map(|f| match **f {
                Eq(ref ts) => Some(ts.into_iter()),
                Prd(_, ref ts) => Some(ts.iter()),
                _ => None,
            })
            .flatten()
    }

    pub fn predicate_symbols(f: &Id<Formula>) -> HashSet<(&Id<Symbol>, usize)> {
        Self::breadth_first(f)
            .filter_map(|f| match **f {
                Prd(ref p, ref ts) => Some((p, ts.len())),
                _ => None,
            })
            .collect()
    }

    pub fn function_symbols(f: &Id<Formula>) -> HashSet<(&Id<Symbol>, usize)> {
        Self::breadth_first_terms(f)
            .flat_map(|t| Term::function_symbols(t))
            .collect()
    }
}

impl fmt::Debug for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            T => write!(f, "T"),
            F => write!(f, "F"),
            Eq(ts) => write!(f, "={:?}", ts),
            Prd(p, ts) => {
                if ts.is_empty() {
                    write!(f, "{:?}", p)
                } else {
                    write!(f, "{:?}{:?}", p, ts)
                }
            }
            Not(p) => write!(f, "¬{:?}", p),
            Or(ps) => write!(f, "or{:?}", ps),
            And(ps) => write!(f, "and{:?}", ps),
            Eqv(ps) => write!(f, "eqv{:?}", ps),
            Imp(p, q) => write!(f, "imp[{:?}, {:?}]", p, q),
            All(p) => write!(f, "all[{:?}]", p),
            Ex(p) => write!(f, "ex[{:?}]", p),
        }
    }
}

struct BreadthFirst<'a> {
    todo: VecDeque<&'a Id<Formula>>,
}

impl<'a> BreadthFirst<'a> {
    fn new(f: &'a Id<Formula>) -> Self {
        let mut todo = VecDeque::new();
        todo.push_back(f);
        Self { todo }
    }
}

impl<'a> Iterator for BreadthFirst<'a> {
    type Item = &'a Id<Formula>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.todo.pop_front()?;
        match **next {
            T | F | Eq(_) | Prd(_, _) => {}
            Not(ref p) | All(ref p) | Ex(ref p) => self.todo.push_back(p),
            Or(ref ps) | And(ref ps) | Eqv(ref ps) => {
                self.todo.extend(ps.into_iter())
            }
            Imp(ref p, ref q) => {
                self.todo.push_back(p);
                self.todo.push_back(q);
            }
        }
        Some(next)
    }
}
