use std::sync::Arc;
use std::vec::Vec;

use im;

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

    pub fn with_many(&self, formulae: Set<Arc<Formula>>) -> Self {
        Goal::new(self.refute.clone().union(formulae))
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

    pub fn consume(self) -> Set<Arc<Formula>> {
        self.refute
    }
}

pub enum Proof {
    Leaf,
    Branch(Set<Arc<Formula>>, Vec<Box<Proof>>),
}

impl Proof {
    pub fn leaf(goal: &Goal) -> Self {
        assert!(goal.complete(), "proof leaves must have complete goals");
        Proof::Leaf
    }

    pub fn branch(goal: Goal, children: Vec<Box<Proof>>) -> Self {
        Proof::Branch(goal.consume(), children)
    }
}
