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

    pub fn replace(term: Uniq<Term>, to: Uniq<Term>, from: Uniq<Term>) -> Uniq<Term> {
        if term == to {
            return from
        }
        
        match *term {
            Var(_) => term,
            Fun(f, ref args) => {
                Uniq::new(Fun(f, args.iter().map(|t| Term::replace(*t, to, from)).collect()))
            }
        }
    }

    fn shift_indices(&self, shift: usize) -> Uniq<Term> {
        match *self {
            Var(x) => Uniq::new(Var(x + shift)),
            Fun(f, ref args) => Uniq::new(Fun(
                f,
                args.iter().map(|t| t.shift_indices(shift)).collect(),
            )),
        }
    }

    pub fn instantiate(term: Uniq<Term>, i: Uniq<Term>, index: usize) -> Uniq<Term> {
        match *term {
            Var(x) if x == index => i.shift_indices(index),
            Var(_) => term,
            Fun(f, ref args) => Uniq::new(Fun(
                f,
                args.iter().map(|t| Term::instantiate(*t, i, index)).collect(),
            )),
        }
    }
}
