use std::vec::Vec;

use unique::{Backed, Uniq};
use unique::backing::HashBacking;

use crate::symbol::Symbol;
use crate::types::Set;
use crate::term::Term;
use crate::term::Term::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Formula {
    T,
    F,
    Eql(Uniq<Term>, Uniq<Term>),
    Prd(Symbol, Vec<Uniq<Term>>),
    Not(Uniq<Formula>),
    Imp(Uniq<Formula>, Uniq<Formula>),
    Eqv(Uniq<Formula>, Uniq<Formula>),
    And(Set<Uniq<Formula>>),
    Or(Set<Uniq<Formula>>),
    All(Uniq<Formula>),
    Ex(Uniq<Formula>),
}
use self::Formula::*;

lazy_static! {
    static ref FORMULA_BACKING: HashBacking<Formula> = HashBacking::new(0x1000);
}

impl Backed for Formula {
    fn unique(self) -> Uniq<Self> {
        FORMULA_BACKING.unique(self)
    }
}

impl Formula {
    pub fn negate(formula: Uniq<Formula>) -> Uniq<Formula> {
        Uniq::new(Formula::Not(formula))
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

    pub fn replace(&self, to: Uniq<Term>, from: Uniq<Term>) -> Uniq<Formula> {
        match *self {
            T => Uniq::new(T),
            F => Uniq::new(F),
            Eql(left, right) => Uniq::new(Eql(Term::replace(left, to, from), Term::replace(right, to, from))),
            Prd(p, ref args) => Uniq::new(Prd(p, args.iter().map(|t| Term::replace(*t, to, from)).collect())),
            Not(ref p) => Uniq::new(Not(p.replace(to, from))),
            Imp(ref p, ref q) => Uniq::new(Imp(p.replace(to, from), q.replace(to, from))),
            Eqv(ref p, ref q) => Uniq::new(Eqv(p.replace(to, from), q.replace(to, from))),
            And(ref ps) => Uniq::new(And(ps.iter().map(|p| p.replace(to, from)).collect())),
            Or(ref ps) => Uniq::new(Or(ps.iter().map(|p| p.replace(to, from)).collect())),
            All(ref p) => Uniq::new(All(p.replace(to, from))),
            Ex(ref p) => Uniq::new(Ex(p.replace(to, from))),
        }
    }

    pub fn instantiate(&self, i: Uniq<Term>, index: usize) -> Uniq<Formula> {
        match *self {
            T => Uniq::new(T),
            F => Uniq::new(F),
            Prd(p, ref args) => Uniq::new(Prd(
                p,
                args.iter().map(|t| Term::instantiate(*t, i, index)).collect(),
            )),
            Eql(left, right) => {
                Uniq::new(Eql(Term::instantiate(left, i, index), Term::instantiate(right, i, index)))
            }
            Not(ref p) => Uniq::new(Not(p.instantiate(i, index))),
            Imp(ref left, ref right) => {
                Uniq::new(Imp(left.instantiate(i, index), right.instantiate(i, index)))
            }
            Eqv(ref left, ref right) => {
                Uniq::new(Eqv(left.instantiate(i, index), right.instantiate(i, index)))
            }
            And(ref ps) => Uniq::new(And(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            Or(ref ps) => Uniq::new(Or(ps.iter().map(|p| p.instantiate(i, index)).collect())),
            All(ref p) => Uniq::new(All(p.instantiate(i, index + 1))),
            Ex(ref p) => Uniq::new(Ex(p.instantiate(i, index + 1))),
        }
    }

    pub fn instantiate_with_constant(&self) -> Uniq<Formula> {
        let constant = Uniq::new(Fun(Symbol::fresh(0), vec![]));
        self.instantiate(constant, 0)
    }

    pub fn instantiate_with_symbol(&self, symbol: Symbol) -> Uniq<Formula> {
        let arity = symbol.arity();
        let vars = (0..arity).map(|i| Uniq::new(Var(i))).collect();
        let term = Uniq::new(Fun(symbol, vars));
        let mut f = self.instantiate(term, 0);
        for _ in 0..arity {
            f = Uniq::new(All(f));
        }
        f
    }
}
