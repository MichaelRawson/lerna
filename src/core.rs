use std::sync::Arc;
use std::vec::Vec;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use im;
use parking_lot::RwLock;

use options::CoreOptions;
use util::BiMap;

pub type Set<T> = im::OrdSet<T>;
macro_rules! set {
    ($($x:tt)*) => {ordset![$($x)*]}
}
pub type Map<K, V> = im::OrdMap<K, V>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bound(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Var(Bound),
    Fun(Symbol, Vec<Arc<Term>>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formula {
    T,
    F,
    Eql(Arc<Term>, Arc<Term>),
    Prd(Symbol, Vec<Arc<Term>>),
    Not(Arc<Formula>),
    Imp(Arc<Formula>, Arc<Formula>),
    Eqv(Arc<Formula>, Arc<Formula>),
    And(Set<Arc<Formula>>),
    Or(Set<Arc<Formula>>),
    All(Bound, Arc<Formula>),
    Ex(Bound, Arc<Formula>),
}

impl Formula {
    pub fn negate(formula: Arc<Formula>) -> Arc<Formula> {
        Arc::new(Formula::Not(formula))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Goal {
    pub formulae: Set<Arc<Formula>>,
}

impl Goal {
    pub fn new(formulae: Set<Arc<Formula>>) -> Goal {
        Goal { formulae }
    }

    pub fn complete(&self) -> bool {
        self.formulae.contains(&Formula::F)
    }
}

pub struct Core {
    fresh: Atomic<usize>,
    symbols: RwLock<BiMap<(Arc<String>, usize), Symbol>>,
}

impl Core {
    pub fn new(_options: &CoreOptions) -> Self {
        let core = Core {
            fresh: Atomic::new(0),
            symbols: RwLock::new(BiMap::new()),
        };

        debug!("core initialised");
        core
    }

    pub fn fresh_binder(&self) -> Bound {
        let fresh = self.fresh.fetch_add(1, Relaxed);
        Bound(fresh)
    }

    pub fn symbol_for(&self, name: Arc<String>, arity: usize) -> Symbol {
        let mut symbols = self.symbols.write();
        let entry = (name, arity);

        if let Some(symbol) = symbols.forward(&entry) {
            return *symbol;
        }

        let symbol = Symbol(symbols.len());
        symbols.insert(&entry, &symbol);
        symbol
    }

    pub fn name_of_symbol(&self, symbol: Symbol) -> Arc<String> {
        self.symbols.read().back(&symbol).unwrap().0.clone()
    }
}
