use std::sync::Arc;

use collections::Set;
use formula::Formula;
use symbol::Symbol;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Goal {
    refute: Set<Arc<Formula>>,
    symbols: Set<Symbol>,
}

impl Goal {
    pub fn new(refute: Set<Arc<Formula>>) -> Self {
        let symbols = Set::unions(refute.iter().map(|f| f.symbols()));
        Goal { refute, symbols }
    }

    pub fn with(&self, f: Arc<Formula>) -> Self {
        if self.refute.contains(&f) {
            self.clone()
        } else {
            let mut refute = self.refute.clone();
            let symbols = self.symbols.clone().union(f.symbols());
            refute.insert(f);
            Goal { refute, symbols }
        }
    }

    pub fn with_many(&self, formulae: Set<Arc<Formula>>) -> Self {
        formulae
            .into_iter()
            .fold(self.clone(), |goal, f| goal.with(f))
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

    pub fn as_refutation(self) -> Set<Arc<Formula>> {
        self.refute
    }
}
