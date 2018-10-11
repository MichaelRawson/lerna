use std::sync::Arc;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::Mutex;

use core::{Symbol};
use util::BiMap;

lazy_static! {
    static ref FRESH: Atomic<usize> = Atomic::new(0);
    static ref SYMBOLS: Mutex<BiMap<(Arc<String>, usize), Symbol>> = Mutex::new(BiMap::new());
}

pub fn fresh_symbol(arity: usize) -> Symbol {
    let fresh = FRESH.fetch_add(1, Relaxed);
    let name = Arc::new(format!("_k{}", fresh));
    symbol_for(&name, arity)
}

pub fn symbol_for(name: &Arc<String>, arity: usize) -> Symbol {
    let mut symbols = SYMBOLS.lock();
    let entry = (name.clone(), arity);

    if let Some(symbol) = symbols.forward(&entry) {
        return *symbol;
    }

    let symbol = Symbol(symbols.len());
    symbols.insert(&entry, &symbol);
    symbol
}

pub fn symbol_name(symbol: Symbol) -> Arc<String> {
    SYMBOLS.lock().back(&symbol).unwrap().0.clone()
}

pub fn symbol_arity(symbol: Symbol) -> usize {
    SYMBOLS.lock().back(&symbol).unwrap().1
}
