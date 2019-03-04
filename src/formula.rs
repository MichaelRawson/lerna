use lazy_static::lazy_static;
use std::fmt;
use unique::allocators::HashAllocator;
use unique::{make_allocator, Id};

use crate::collections::Set;
use crate::symbol::Symbol;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Formula {
    T,
    F,
    Prd(Id<Symbol>),
    Not(Id<Formula>),
    Or(Set<Formula>),
    And(Set<Formula>),
    Imp(Id<Formula>, Id<Formula>),
    Eqv(Id<Formula>, Id<Formula>),
}
make_allocator!(Formula, FORMULA_ALLOC, HashAllocator);
use self::Formula::*;

lazy_static! {
    pub static ref FALSE: Id<Formula> = Id::new(F);
}

impl Formula {
    pub fn negate(formula: &Id<Formula>) -> Id<Formula> {
        Id::new(Not(formula.clone()))
    }
}

impl fmt::Debug for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            T => write!(f, "T"),
            F => write!(f, "F"),
            Prd(s) => write!(f, "{:?}", s),
            Not(p) => write!(f, "not({:?})", p),
            Or(ps) => write!(f, "or({:?})", ps),
            And(ps) => write!(f, "and({:?})", ps),
            Imp(p, q) => write!(f, "imp({:?}, {:?})", p, q),
            Eqv(p, q) => write!(f, "eqv({:?}, {:?})", p, q),
        }
    }
}
