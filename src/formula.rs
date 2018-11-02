use crate::types::Dag;
use std::vec::Vec;

use crate::symbol::Symbol;
use crate::types::Set;

use crate::term::Term;
use crate::term::Term::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formula {
    T,
    F,
    Eql(Dag<Term>, Dag<Term>),
    Prd(Symbol, Vec<Dag<Term>>),
    Not(Dag<Formula>),
    Imp(Dag<Formula>, Dag<Formula>),
    Eqv(Dag<Formula>, Dag<Formula>),
    And(Set<Dag<Formula>>),
    Or(Set<Dag<Formula>>),
    All(Dag<Formula>),
    Ex(Dag<Formula>),
}
use self::Formula::*;

impl Formula {
    pub fn negated(&self) -> Dag<Formula> {
        dag!(Formula::Not(dag!(self.clone())))
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

    pub fn replace(&self, to: &Dag<Term>, from: &Dag<Term>) -> Dag<Formula> {
        match *self {
            T => dag!(T),
            F => dag!(F),
            Eql(ref left, ref right) => dag!(Eql(left.replace(to, from), right.replace(to, from))),
            Prd(p, ref args) => dag!(Prd(p, args.iter().map(|t| t.replace(to, from)).collect())),
            Not(ref p) => dag!(Not(p.replace(to, from))),
            Imp(ref p, ref q) => dag!(Imp(p.replace(to, from), q.replace(to, from))),
            Eqv(ref p, ref q) => dag!(Eqv(p.replace(to, from), q.replace(to, from))),
            And(ref ps) => dag!(And(ps.iter().map(|p| p.replace(to, from)).collect())),
            Or(ref ps) => dag!(Or(ps.iter().map(|p| p.replace(to, from)).collect())),
            All(ref p) => dag!(All(p.replace(to, from))),
            Ex(ref p) => dag!(Ex(p.replace(to, from))),
        }
    }

    pub fn instantiate(&self, i: &Dag<Term>, index: usize) -> Dag<Formula> {
        match *self {
            T => dag!(T),
            F => dag!(F),
            Prd(p, ref args) => dag!(Prd(
                p,
                args.iter().map(|t| t.instantiate(i, index)).collect(),
            )),
            Eql(ref left, ref right) => {
                dag!(Eql(left.instantiate(i, index), right.instantiate(i, index)))
            }
            Not(ref p) => dag!(Not(p.instantiate(i, index))),
            Imp(ref left, ref right) => {
                dag!(Imp(left.instantiate(i, index), right.instantiate(i, index)))
            }
            Eqv(ref left, ref right) => {
                dag!(Eqv(left.instantiate(i, index), right.instantiate(i, index)))
            }
            And(ref ps) => dag!(And(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            Or(ref ps) => dag!(Or(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            All(ref p) => dag!(All(p.instantiate(i, index + 1))),
            Ex(ref p) => dag!(Ex(p.instantiate(i, index + 1))),
        }
    }

    pub fn instantiate_with_constant(&self) -> Dag<Formula> {
        let constant = dag!(Fun(Symbol::fresh(0), vec![]));
        self.instantiate(&constant, 0)
    }

    pub fn instantiate_with_symbol(&self, symbol: Symbol) -> Dag<Formula> {
        let arity = symbol.arity();
        let vars = (0..arity).map(|i| dag!(Var(i))).collect();
        let term = dag!(Fun(symbol, vars));
        let mut f = self.instantiate(&term, 0);
        for _ in 0..arity {
            f = dag!(All(f));
        }
        f
    }
}
