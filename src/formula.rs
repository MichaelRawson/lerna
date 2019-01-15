use std::ops::Deref;

use unique::{Backed, Uniq};
use unique::backing::HashBacking;

use crate::symbol::Symbol;
use crate::types::Set;
use crate::term::{Term, TermList};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormulaSet(pub Set<Uniq<Formula>>);

impl Deref for FormulaSet {
    type Target = Set<Uniq<Formula>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Formula {
    T,
    F,
    Eql(Uniq<Term>, Uniq<Term>),
    Prd(Symbol, Uniq<TermList>),
    Not(Uniq<Formula>),
    Imp(Uniq<Formula>, Uniq<Formula>),
    Eqv(Uniq<Formula>, Uniq<Formula>),
    And(Uniq<FormulaSet>),
    Or(Uniq<FormulaSet>),
    All(Uniq<Formula>),
    Ex(Uniq<Formula>),
}
use self::Formula::*;

lazy_static! {
    static ref FORMULA_BACKING: HashBacking<Formula> = HashBacking::new(0x1000);
    static ref FORMULA_SET_BACKING: HashBacking<FormulaSet> = HashBacking::new(0x1000);
}

impl Backed for Formula {
    fn unique(formula: Self) -> Uniq<Self> {
        FORMULA_BACKING.unique(formula)
    }
}

impl Backed for FormulaSet {
    fn unique(set: Self) -> Uniq<Self> {
        FORMULA_SET_BACKING.unique(set)
    }
}

impl Formula {
    pub fn t() -> Uniq<Self> {
        uniq!(Formula::T)
    }

    pub fn f() -> Uniq<Self> {
        uniq!(Formula::F)
    }

    pub fn eql(left: Uniq<Term>, right: Uniq<Term>) -> Uniq<Self> {
        uniq!(Formula::Eql(left, right))
    }

    pub fn prd<T: Iterator<Item=Uniq<Term>>>(name: Symbol, args: T) -> Uniq<Self> {
        uniq!(Formula::Prd(name, uniq!(TermList(args.collect()))))
    }

    pub fn not(formula: Uniq<Self>) -> Uniq<Self> {
        uniq!(Formula::Not(formula))
    }


    pub fn symbols(&self) -> Set<Symbol> {
        match *self {
            T | F => set![],
            Eql(left, right) => left.symbols().union(right.symbols()),
            Prd(_, args) => Set::unions(args.iter().map(|t| t.symbols())),
            Not(p) => p.symbols(),
            Imp(p, q) => p.symbols().union(q.symbols()),
            Eqv(p, q) => p.symbols().union(q.symbols()),
            And(ps) => Set::unions(ps.iter().map(|p| p.symbols())),
            Or(ps) => Set::unions(ps.iter().map(|p| p.symbols())),
            All(p) => p.symbols(),
            Ex(p) => p.symbols(),
        }
    }

    pub fn replace(formula: Uniq<Self>, to: Uniq<Term>, from: Uniq<Term>) -> Uniq<Self> {
        match *formula {
            T => formula,
            F => formula,
            Eql(left, right) => uniq!(Eql(Term::replace(left, to, from), Term::replace(right, to, from))),
            Prd(p, args) => uniq!(Prd(p, uniq!(TermList(args.iter().map(|t| Term::replace(*t, to, from)).collect())))),
            Not(p) => uniq!(Not(Self::replace(p, to, from))),
            Imp(p, q) => uniq!(Imp(Self::replace(p, to, from), Self::replace(q, to, from))),
            Eqv(p, q) => uniq!(Eqv(Self::replace(p, to, from), Self::replace(q, to, from))),
            And(ps) => uniq!(And(uniq!(FormulaSet(ps.iter().map(|p| Self::replace(*p, to, from)).collect())))),
            Or(ps) => uniq!(Or(uniq!(FormulaSet(ps.iter().map(|p| Self::replace(*p, to, from)).collect())))),
            All(p) => uniq!(All(Self::replace(p, to, from))),
            Ex(p) => uniq!(Ex(Self::replace(p, to, from))),
        }
    }

    pub fn instantiate(formula: Uniq<Self>, i: Uniq<Term>, index: usize) -> Uniq<Self> {
        match *formula {
            T => formula,
            F => formula,
            Prd(p, args) => uniq!(Prd(
                p,
                uniq!(TermList(args.iter().map(|t| Term::instantiate(*t, i, index)).collect())),
            )),
            Eql(left, right) => {
                uniq!(Eql(Term::instantiate(left, i, index), Term::instantiate(right, i, index)))
            }
            Not(p) => uniq!(Not(Self::instantiate(p, i, index))),
            Imp(left, right) => {
                uniq!(Imp(Self::instantiate(left, i, index), Self::instantiate(right, i, index)))
            }
            Eqv(left, right) => {
                uniq!(Eqv(Self::instantiate(left, i, index), Self::instantiate(right, i, index)))
            }
            And(ps) => uniq!(And(uniq!(FormulaSet(ps.iter().map(|p| Self::instantiate(*p, i, index)).collect())))),
            Or(ps) => uniq!(Or(uniq!(FormulaSet(ps.iter().map(|p| Self::instantiate(*p, i, index)).collect())))),
            All(p) => uniq!(All(Self::instantiate(p, i, index + 1))),
            Ex(p) => uniq!(Ex(Self::instantiate(p, i, index + 1))),
        }
    }

    pub fn instantiate_with_constant(formula: Uniq<Self>) -> Uniq<Self> {
        let constant = Term::fun(Symbol::fresh(0), std::iter::empty());
        Self::instantiate(formula, constant, 0)
    }

    pub fn instantiate_with_symbol(formula: Uniq<Self>, symbol: Symbol) -> Uniq<Self> {
        let arity = symbol.arity();
        let vars = (0..arity).map(Term::var);
        let term = Term::fun(symbol, vars);
        let mut f = Self::instantiate(formula, term, 0);
        for _ in 0..arity {
            f = uniq!(All(f));
        }
        f
    }
}
