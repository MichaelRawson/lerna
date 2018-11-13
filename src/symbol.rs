use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::Mutex;

use crate::types::BiMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Flavour {
    Functor,
    Distinct,
}

lazy_static! {
    static ref FRESH: Atomic<usize> = Atomic::new(0);
    static ref SYMBOLS: Mutex<BiMap<(String, usize, Flavour), Symbol>> = Mutex::new(BiMap::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(usize);

impl Symbol {
    pub fn get(name: String, arity: usize, flavour: Flavour) -> Self {
        let mut symbols = SYMBOLS.lock();
        let entry = (name, arity, flavour);

        if let Some(symbol) = symbols.forward(&entry) {
            return *symbol;
        }

        let symbol = Symbol(symbols.len());
        symbols.insert(&entry, &symbol);
        symbol
    }

    pub fn fresh(arity: usize) -> Self {
        let fresh = FRESH.fetch_add(1, Relaxed);
        let name = format!("_k{}", fresh);
        Symbol::get(name, arity, Flavour::Functor)
    }

    pub fn name(self) -> String {
        SYMBOLS.lock().back(&self).unwrap().0.clone()
    }

    pub fn arity(self) -> usize {
        SYMBOLS.lock().back(&self).unwrap().1
    }

    pub fn flavour(self) -> Flavour {
        SYMBOLS.lock().back(&self).unwrap().2
    }
}
