use lazy_static::lazy_static;
use std::fmt;
use unique::allocators::HashAllocator;
use unique::{make_allocator, Allocated};

#[derive(PartialEq, Eq, Hash)]
pub enum Symbol {
    Original(String),
}

make_allocator!(Symbol, __SYMBOL_ALLOC, HashAllocator);

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Symbol::*;
        match self {
            Original(s) => write!(f, "{}", s),
        }
    }
}
