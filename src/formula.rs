use std::vec::Vec;
use types::Dag;

use symbol::Symbol;
use types::Set;

use term::Term;
use term::Term::*;

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
        Dag::new(Formula::Not(Dag::new(self.clone())))
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
            T => Dag::new(T),
            F => Dag::new(F),
            Eql(ref left, ref right) => {
                Dag::new(Eql(left.replace(to, from), right.replace(to, from)))
            }
            Prd(p, ref args) => {
                Dag::new(Prd(p, args.iter().map(|t| t.replace(to, from)).collect()))
            }
            Not(ref p) => Dag::new(Not(p.replace(to, from))),
            Imp(ref p, ref q) => Dag::new(Imp(p.replace(to, from), q.replace(to, from))),
            Eqv(ref p, ref q) => Dag::new(Eqv(p.replace(to, from), q.replace(to, from))),
            And(ref ps) => Dag::new(And(ps.iter().map(|p| p.replace(to, from)).collect())),
            Or(ref ps) => Dag::new(Or(ps.iter().map(|p| p.replace(to, from)).collect())),
            All(ref p) => Dag::new(All(p.replace(to, from))),
            Ex(ref p) => Dag::new(Ex(p.replace(to, from))),
        }
    }

    pub fn instantiate(&self, i: &Dag<Term>, index: usize) -> Dag<Formula> {
        match *self {
            T => Dag::new(T),
            F => Dag::new(F),
            Prd(p, ref args) => Dag::new(Prd(
                p,
                args.iter().map(|t| t.instantiate(i, index)).collect(),
            )),
            Eql(ref left, ref right) => {
                Dag::new(Eql(left.instantiate(i, index), right.instantiate(i, index)))
            }
            Not(ref p) => Dag::new(Not(p.instantiate(i, index))),
            Imp(ref left, ref right) => {
                Dag::new(Imp(left.instantiate(i, index), right.instantiate(i, index)))
            }
            Eqv(ref left, ref right) => {
                Dag::new(Eqv(left.instantiate(i, index), right.instantiate(i, index)))
            }
            And(ref ps) => Dag::new(And(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            Or(ref ps) => Dag::new(Or(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            All(ref p) => Dag::new(All(p.instantiate(i, index + 1))),
            Ex(ref p) => Dag::new(Ex(p.instantiate(i, index + 1))),
        }
    }

    pub fn instantiate_with_constant(&self) -> Dag<Formula> {
        let constant = Dag::new(Fun(Symbol::fresh(0), vec![]));
        self.instantiate(&constant, 0)
    }

    pub fn instantiate_with_symbol(&self, symbol: Symbol) -> Dag<Formula> {
        let arity = symbol.arity();
        let vars = (0..arity).map(|i| Dag::new(Var(i))).collect();
        let term = Dag::new(Fun(symbol, vars));
        let mut f = self.instantiate(&term, 0);
        for _ in 0..arity {
            f = Dag::new(All(f));
        }
        f
    }
}
