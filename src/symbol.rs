use std::fmt;
use unique::allocators::HashAllocator;
use unique::make_allocator;

#[derive(PartialEq, Eq, Hash)]
pub enum Symbol {
    Original(String),
    Introduced(usize),
}
make_allocator!(Symbol, SYMBOL_ALLOC, HashAllocator);

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Symbol::*;
        match self {
            Original(s) => write!(f, "{}", s),
            Introduced(n) => write!(f, "k_{:x}", n),
        }
    }
}
