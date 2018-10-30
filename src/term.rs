use std::vec::Vec;
use types::Dag;

use symbol::Symbol;
use types::Set;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Var(usize),
    Fun(Symbol, Vec<Dag<Term>>),
}
use self::Term::*;

impl Term {
    pub fn symbols(&self) -> Set<Symbol> {
        match *self {
            Var(_) => set![],
            Fun(f, ref args) => {
                let mut arg_symbols = Set::unions(args.iter().map(|t| t.symbols()));
                arg_symbols.insert(f);
                arg_symbols
            }
        }
    }

    pub fn replace(&self, to: &Dag<Term>, from: &Dag<Term>) -> Dag<Term> {
        if *self == **to {
            from.clone()
        } else {
            match *self {
                Var(x) => Dag::new(Var(x)),
                Fun(f, ref args) => {
                    Dag::new(Fun(f, args.iter().map(|t| t.replace(to, from)).collect()))
                }
            }
        }
    }

    fn shift_indices(&self, shift: usize) -> Dag<Term> {
        match *self {
            Var(x) => Dag::new(Var(x + shift)),
            Fun(f, ref args) => Dag::new(Fun(
                f,
                args.iter().map(|t| t.shift_indices(shift)).collect(),
            )),
        }
    }

    pub fn instantiate(&self, i: &Dag<Term>, index: usize) -> Dag<Term> {
        match *self {
            Var(x) => if x == index {
                i.shift_indices(index)
            } else {
                Dag::new(Var(x))
            },
            Fun(f, ref args) => Dag::new(Fun(
                f,
                args.iter().map(|t| t.instantiate(i, index)).collect(),
            )),
        }
    }
}
