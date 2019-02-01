use lazy_static::lazy_static;
use std::fmt;
use unique::allocators::HashAllocator;
use unique::{make_allocator, Allocated, Id};

use crate::set::Set;
use crate::symbol::Symbol;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Formula {
    T,
    F,
    Prd(Id<Symbol>),
    Not(Id<Formula>),
    Or(Id<Set<Formula>>),
    And(Id<Set<Formula>>),
    Imp(Id<Formula>, Id<Formula>),
    Eqv(Id<Formula>, Id<Formula>),
}

make_allocator!(Formula, __FORMULA_ALLOC, HashAllocator);
make_allocator!(Set<Formula>, __FORMULA_SET_ALLOC, HashAllocator);

impl fmt::Debug for Formula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Formula::*;
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
