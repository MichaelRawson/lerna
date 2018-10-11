use std::sync::Arc;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::Mutex;

use collections::BiMap;

lazy_static! {
    static ref FRESH: Atomic<usize> = Atomic::new(0);
    static ref SYMBOLS: Mutex<BiMap<(Arc<String>, usize), Symbol>> = Mutex::new(BiMap::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(usize);

impl Symbol {
    pub fn get(name: &Arc<String>, arity: usize) -> Self {
        let mut symbols = SYMBOLS.lock();
        let entry = (name.clone(), arity);

        if let Some(symbol) = symbols.forward(&entry) {
            return *symbol;
        }

        let symbol = Symbol(symbols.len());
        symbols.insert(&entry, &symbol);
        symbol
    }

    pub fn fresh(arity: usize) -> Self {
        let fresh = FRESH.fetch_add(1, Relaxed);
        let name = Arc::new(format!("_k{}", fresh));
        Symbol::get(&name, arity)
    }

    pub fn name(self) -> Arc<String> {
        SYMBOLS.lock().back(&self).unwrap().0.clone()
    }

    pub fn arity(self) -> usize {
        SYMBOLS.lock().back(&self).unwrap().1
    }
}
