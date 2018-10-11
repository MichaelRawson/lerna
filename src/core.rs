use std::sync::Arc;
use std::vec::Vec;

use im;
use common::{formula_symbols};

pub type Set<T> = im::OrdSet<T>;
macro_rules! set {
    ($($x:tt)*) => {ordset![$($x)*]}
}
pub type Map<K, V> = im::OrdMap<K, V>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bound(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(pub usize);

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
    All(Arc<Formula>),
    Ex(Arc<Formula>),
}

impl Formula {
    pub fn negate(formula: Arc<Formula>) -> Arc<Formula> {
        Arc::new(Formula::Not(formula))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Goal {
    refute: Set<Arc<Formula>>,
    symbols: Set<Symbol>
}

impl Goal {
    pub fn new(refute: Set<Arc<Formula>>) -> Self {
        let symbols = Set::unions(refute.iter().map(formula_symbols));
        Goal { refute, symbols }
    }

    pub fn with(&self, f: Arc<Formula>) -> Self {
        if self.refute.contains(&f) {
            self.clone()
        }
        else {
            let mut refute = self.refute.clone();
            let symbols = self.symbols.clone().union(formula_symbols(&f));
            refute.insert(f);
            Goal { refute, symbols }
        }
    }

    pub fn with_many(&self, formulae: Set<Arc<Formula>>) -> Self {
        formulae.into_iter().fold(self.clone(), |goal, f| goal.with(f))
    }

    pub fn contains(&self, f: &Formula) -> bool {
        self.refute.contains(f)
    }

    pub fn complete(&self) -> bool {
        self.refute.contains(&Formula::F)
    }

    pub fn formulae(&self) -> impl Iterator<Item = &Arc<Formula>> {
        self.refute.iter()
    }

    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }

    pub fn consume(self) -> Set<Arc<Formula>> {
        self.refute
    }
}

pub enum Proof {
    Leaf,
    Branch(Goal, Vec<Box<Proof>>),
}

impl Proof {
    pub fn leaf(goal: &Goal) -> Self {
        assert!(goal.complete(), "proof leaves must have complete goals");
        Proof::Leaf
    }

    pub fn branch(goal: Goal, children: Vec<Box<Proof>>) -> Self {
        Proof::Branch(goal, children)
    }
}
