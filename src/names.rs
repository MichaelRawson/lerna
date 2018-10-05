use std::sync::Arc;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::Mutex;

use core::{Bound, Symbol};
use util::BiMap;

lazy_static! {
    static ref FRESH: Atomic<usize> = Atomic::new(0);
    static ref SYMBOLS: Mutex<BiMap<Arc<String>, Symbol>> = Mutex::new(BiMap::new());
}

pub fn fresh_binder() -> Bound {
    let fresh = FRESH.fetch_add(1, Relaxed);
    Bound(fresh)
}

pub fn fresh_symbol() -> Symbol {
    let fresh = FRESH.fetch_add(1, Relaxed);
    let name = Arc::new(format!("_k{}", fresh));
    symbol_for(&name)
}

pub fn symbol_for(name: &Arc<String>) -> Symbol {
    let mut symbols = SYMBOLS.lock();

    if let Some(symbol) = symbols.forward(&name) {
        return *symbol;
    }

    let symbol = Symbol(symbols.len());
    symbols.insert(name, &symbol);
    symbol
}

pub fn symbol_name(symbol: Symbol) -> Arc<String> {
    SYMBOLS.lock().back(&symbol).unwrap().clone()
}
