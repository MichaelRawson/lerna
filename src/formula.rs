use std::sync::Arc;
use std::vec::Vec;

use collections::Set;
use symbol::Symbol;

use term::Term;
use term::Term::*;

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
use self::Formula::*;

impl Formula {
    pub fn negated(&self) -> Arc<Formula> {
        Arc::new(Formula::Not(Arc::new(self.clone())))
    }

    pub fn symbols(&self) -> Set<Symbol> {
        match *self {
            T | F => set![],
            Eql(ref left, ref right) => left.symbols().union(right.symbols()),
            Prd(_, ref args) => Set::unions(args.iter().map(|t| t.symbols())),
            Not(ref p) => p.symbols(),
            Imp(ref p, ref q) => p.symbols().union(q.symbols()),
            Eqv(ref p, ref q) => p.symbols().union(q.symbols()),
            And(ref ps) => Set::unions(ps.iter().map(|p| p.symbols())),
            Or(ref ps) => Set::unions(ps.iter().map(|p| p.symbols())),
            All(ref p) => p.symbols(),
            Ex(ref p) => p.symbols(),
        }
    }

    pub fn replace(&self, to: &Arc<Term>, from: &Arc<Term>) -> Arc<Formula> {
        match *self {
            T => Arc::new(T),
            F => Arc::new(F),
            Eql(ref left, ref right) => {
                Arc::new(Eql(left.replace(to, from), right.replace(to, from)))
            }
            Prd(p, ref args) => {
                Arc::new(Prd(p, args.iter().map(|t| t.replace(to, from)).collect()))
            }
            Not(ref p) => Arc::new(Not(p.replace(to, from))),
            Imp(ref p, ref q) => Arc::new(Imp(p.replace(to, from), q.replace(to, from))),
            Eqv(ref p, ref q) => Arc::new(Eqv(p.replace(to, from), q.replace(to, from))),
            And(ref ps) => Arc::new(And(ps.iter().map(|p| p.replace(to, from)).collect())),
            Or(ref ps) => Arc::new(Or(ps.iter().map(|p| p.replace(to, from)).collect())),
            All(ref p) => Arc::new(All(p.replace(to, from))),
            Ex(ref p) => Arc::new(Ex(p.replace(to, from))),
        }
    }

    pub fn instantiate(&self, i: &Arc<Term>, index: usize) -> Arc<Formula> {
        match *self {
            T => Arc::new(T),
            F => Arc::new(F),
            Prd(p, ref args) => Arc::new(Prd(
                p,
                args.iter().map(|t| t.instantiate(i, index)).collect(),
            )),
            Eql(ref left, ref right) => {
                Arc::new(Eql(left.instantiate(i, index), right.instantiate(i, index)))
            }
            Not(ref p) => Arc::new(Not(p.instantiate(i, index))),
            Imp(ref left, ref right) => {
                Arc::new(Imp(left.instantiate(i, index), right.instantiate(i, index)))
            }
            Eqv(ref left, ref right) => {
                Arc::new(Eqv(left.instantiate(i, index), right.instantiate(i, index)))
            }
            And(ref ps) => Arc::new(And(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            Or(ref ps) => Arc::new(Or(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            All(ref p) => Arc::new(All(p.instantiate(i, index + 1))),
            Ex(ref p) => Arc::new(Ex(p.instantiate(i, index + 1))),
        }
    }

    pub fn instantiate_with_constant(&self) -> Arc<Formula> {
        let constant = Arc::new(Fun(Symbol::fresh(0), vec![]));
        self.instantiate(&constant, 0)
    }

    pub fn instantiate_with_symbol(&self, symbol: Symbol) -> Arc<Formula> {
        let arity = symbol.arity();
        let vars = (0..arity).map(|i| Arc::new(Var(i))).collect();
        let term = Arc::new(Fun(symbol, vars));
        let mut f = self.instantiate(&term, 0);
        for _ in 0..arity {
            f = Arc::new(All(f));
        }
        f
    }
}
