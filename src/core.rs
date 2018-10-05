use std::sync::Arc;
use std::vec::Vec;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use im;
use parking_lot::RwLock;

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
    refute: Set<Arc<Formula>>,
}

impl Goal {
    pub fn new(refute: Set<Arc<Formula>>) -> Self {
        Goal { refute }
    }

    pub fn with(&self, f: Arc<Formula>) -> Self {
        Goal::new(self.refute.update(f))
    }

    pub fn contains(&self, f: &Formula) -> bool {
        self.refute.contains(f)
    }

    pub fn complete(&self) -> bool {
        self.refute.contains(&Formula::F)
    }

    pub fn formulae(&self) -> impl Iterator<Item=&Arc<Formula>> {
        self.refute.iter()
    }

    pub fn consume(self) -> Set<Arc<Formula>> {
        self.refute
    }
}

pub enum Proof {
    Leaf,
    Branch(Set<Arc<Formula>>, Vec<Box<Proof>>)
}

impl Proof {
    pub fn leaf(goal: Goal) -> Self {
        assert!(goal.complete(), "proof leaves must have complete goals");
        Proof::Leaf
    }

    pub fn branch(goal: Goal, children: Vec<Box<Proof>>) -> Self {
        Proof::Branch(goal.consume(), children)
    }
}

pub struct Names {
    fresh: Atomic<usize>,
    symbols: RwLock<BiMap<(Arc<String>, usize), Symbol>>,
}

impl Names {
    pub fn new() -> Self {
        Names {
            fresh: Atomic::new(0),
            symbols: RwLock::new(BiMap::new()),
        }
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

    pub fn symbol_name(&self, symbol: Symbol) -> Arc<String> {
        self.symbols.read().back(&symbol).unwrap().0.clone()
    }
}
