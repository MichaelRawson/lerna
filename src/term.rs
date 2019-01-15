use std::ops::Deref;

use crate::symbol::Symbol;
use crate::types::{List, Set};

use unique::{Uniq, Backed};
use unique::backing::HashBacking;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermList(pub List<Uniq<Term>>);

impl Deref for TermList {
    type Target = List<Uniq<Term>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Var(usize),
    Fun(Symbol, Uniq<TermList>),
}
use self::Term::*;

lazy_static! {
    static ref TERM_BACKING: HashBacking<Term> = HashBacking::new(0x1000);
    static ref TERM_LIST_BACKING: HashBacking<TermList> = HashBacking::new(0x1000);
}

impl Backed for Term {
    fn unique(term: Self) -> Uniq<Self> {
        TERM_BACKING.unique(term)
    }
}

impl Backed for TermList {
    fn unique(list: Self) -> Uniq<Self> {
        TERM_LIST_BACKING.unique(list)
    }
}


impl Term {
    pub fn var(index: usize) -> Uniq<Self> {
        uniq!(Var(index))
    }

    pub fn fun<T: Iterator<Item=Uniq<Term>>>(name: Symbol, args: T) -> Uniq<Self> {
        uniq!(Fun(name, uniq!(TermList(args.collect()))))
    }

    pub fn symbols(&self) -> Set<Symbol> {
        match *self {
            Var(_) => set![],
            Fun(f, args) => Set::unions(args.iter().map(|t| t.symbols())).update(f)
        }
    }

    pub fn replace(term: Uniq<Self>, to: Uniq<Self>, from: Uniq<Self>) -> Uniq<Self> {
        if term == to {
            return from
        }
        
        match *term {
            Var(_) => term,
            Fun(f, args) => Self::fun(f, args.iter().map(|t| Self::replace(*t, to, from)))
        }
    }

    fn shift_indices(term: Uniq<Self>, shift: usize) -> Uniq<Self> {
        match *term {
            Var(x) => Self::var(x + shift),
            Fun(f, args) => Self::fun(f, args.iter().map(|t| Self::shift_indices(*t, shift))),
        }
    }

    pub fn instantiate(term: Uniq<Self>, i: Uniq<Self>, index: usize) -> Uniq<Self> {
        match *term {
            Var(x) if x == index => Self::shift_indices(i, index),
            Var(_) => term,
            Fun(f, args) => Self::fun(f, args.iter().map(|t| Self::instantiate(*t, i, index))),
        }
    }
}
