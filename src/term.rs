use std::vec::Vec;

use crate::symbol::Symbol;
use crate::types::Set;

use unique::{Uniq, Backed};
use unique::backing::HashBacking;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Var(usize),
    Fun(Symbol, Vec<Uniq<Term>>),
}
use self::Term::*;

lazy_static! {
    static ref TERM_BACKING: HashBacking<Term> = HashBacking::new(0x1000);
}

impl Backed for Term {
    fn unique(self) -> Uniq<Self> {
        TERM_BACKING.unique(self)
    }
}

impl Term {
    pub fn symbols(&self) -> Set<Symbol> {
        match *self {
            Var(_) => set![],
            Fun(f, ref args) => {
                let mut arg_symbols = Set::unions(args.iter().map(|t| t.symbols()));
                arg_symbols.insert(f);
                arg_symbols
            }
        }
    }

    pub fn replace(term: Uniq<Self>, to: Uniq<Self>, from: Uniq<Self>) -> Uniq<Self> {
        if term == to {
            return from
        }
        
        match *term {
            Var(_) => term,
            Fun(f, ref args) => {
                Uniq::new(Fun(f, args.iter().map(|t| Self::replace(*t, to, from)).collect()))
            }
        }
    }

    fn shift_indices(term: Uniq<Self>, shift: usize) -> Uniq<Self> {
        match *term {
            Var(x) => Uniq::new(Var(x + shift)),
            Fun(f, ref args) => Uniq::new(Fun(
                f,
                args.iter().map(|t| Self::shift_indices(*t, shift)).collect(),
            )),
        }
    }

    pub fn instantiate(term: Uniq<Self>, i: Uniq<Self>, index: usize) -> Uniq<Self> {
        match *term {
            Var(x) if x == index => Self::shift_indices(i, index),
            Var(_) => term,
            Fun(f, ref args) => Uniq::new(Fun(
                f,
                args.iter().map(|t| Self::instantiate(*t, i, index)).collect(),
            )),
        }
    }
}
