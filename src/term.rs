use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use unique::allocators::HashAllocator;
use unique::{make_allocator, Id};

use crate::collections::IdList;
use crate::symbol::Symbol;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Var(usize),
    Fn(Id<Symbol>, IdList<Term>),
}
make_allocator!(Term, TERM_ALLOC, HashAllocator);
use self::Term::*;

impl Term {
    pub fn subst(
        t: &Id<Term>,
        index: usize,
        symbol: &Id<Symbol>,
        arity: usize,
    ) -> Id<Term> {
        match **t {
            Var(n) => {
                if n == index {
                    let vars = (index..index + arity)
                        .rev()
                        .map(|n| Id::new(Var(n)))
                        .collect();
                    Id::new(Fn(symbol.clone(), vars))
                } else {
                    t.clone()
                }
            }
            Fn(ref f, ref ts) => Id::new(Fn(
                f.clone(),
                ts.iter()
                    .map(|t| Self::subst(t, index, symbol, arity))
                    .collect(),
            )),
        }
    }

    pub fn replace(t: &Id<Term>, from: &Id<Term>, to: &Id<Term>) -> Id<Term> {
        if t == from {
            to.clone()
        } else {
            match **t {
                Var(_) => t.clone(),
                Fn(ref f, ref ts) => Id::new(Fn(
                    f.clone(),
                    ts.iter().map(|t| Self::replace(t, from, to)).collect(),
                )),
            }
        }
    }

    pub fn breadth_first(t: &Id<Term>) -> impl Iterator<Item = &Id<Term>> {
        BreadthFirst::new(t)
    }

    pub fn is_function(&self) -> bool {
        match self {
            Fn(_, _) => true,
            _ => false,
        }
    }

    pub fn function_symbols(t: &Id<Term>) -> HashSet<(&Id<Symbol>, usize)> {
        Term::breadth_first(t)
            .filter(|t| t.is_function())
            .map(|t| match **t {
                Fn(ref symbol, ref ts) => (symbol, ts.len()),
                _ => unreachable!(),
            })
            .collect()
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Var(n) => write!(f, "{}", n),
            Fn(p, ts) => {
                if ts.is_empty() {
                    write!(f, "{:?}", p)
                } else {
                    write!(f, "{:?}{:?}", p, ts)
                }
            }
        }
    }
}

struct BreadthFirst<'a> {
    todo: VecDeque<&'a Id<Term>>,
}

impl<'a> BreadthFirst<'a> {
    fn new(f: &'a Id<Term>) -> Self {
        let mut todo = VecDeque::new();
        todo.push_back(f);
        Self { todo }
    }
}

impl<'a> Iterator for BreadthFirst<'a> {
    type Item = &'a Id<Term>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.todo.pop_front()?;
        match **next {
            Var(_) => {}
            Fn(_, ref ts) => self.todo.extend(ts.into_iter()),
        }
        Some(next)
    }
}
