use crate::formula::Formula;
use crate::symbol::Symbol;
use crate::types::Set;

use unique::Uniq;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Goal {
    refute: Set<Uniq<Formula>>,
    symbols: Set<Symbol>,
}

impl Goal {
    pub fn new(refute: Set<Uniq<Formula>>) -> Self {
        let symbols = Set::unions(refute.iter().map(|f| f.symbols()));
        Goal { refute, symbols }
    }

    pub fn with(&self, f: Uniq<Formula>) -> Self {
        if self.refute.contains(&f) {
            self.clone()
        } else {
            let mut refute = self.refute.clone();
            let symbols = self.symbols.clone().union(f.symbols());
            refute.insert(f);
            Goal { refute, symbols }
        }
    }

    pub fn with_many(&self, formulae: Set<Uniq<Formula>>) -> Self {
        formulae
            .into_iter()
            .fold(self.clone(), |goal, f| goal.with(f))
    }

    pub fn contains(&self, f: Uniq<Formula>) -> bool {
        self.refute.contains(&f)
    }

    pub fn complete(&self) -> bool {
        self.refute.contains(&Formula::F)
    }

    pub fn formulae(&self) -> impl Iterator<Item = &Uniq<Formula>> {
        self.refute.iter()
    }

    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }

    pub fn refutation(self) -> Set<Uniq<Formula>> {
        self.refute
    }
}
