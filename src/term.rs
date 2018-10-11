use std::sync::Arc;
use std::vec::Vec;

use collections::Set;
use symbol::Symbol;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    Var(usize),
    Fun(Symbol, Vec<Arc<Term>>),
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

    pub fn replace(&self, to: &Arc<Term>, from: &Arc<Term>) -> Arc<Term> {
        if *self == **to {
            from.clone()
        } else {
            match *self {
                Var(x) => Arc::new(Var(x)),
                Fun(f, ref args) => {
                    Arc::new(Fun(f, args.iter().map(|t| t.replace(to, from)).collect()))
                }
            }
        }
    }

    fn shift_indices(&self, shift: usize) -> Arc<Term> {
        match *self {
            Var(x) => Arc::new(Var(x + shift)),
            Fun(f, ref args) => Arc::new(Fun(
                f,
                args.iter().map(|t| t.shift_indices(shift)).collect(),
            )),
        }
    }

    pub fn instantiate(&self, i: &Arc<Term>, index: usize) -> Arc<Term> {
        match *self {
            Var(x) => if x == index {
                i.shift_indices(index)
            } else {
                Arc::new(Var(x))
            },
            Fun(f, ref args) => Arc::new(Fun(
                f,
                args.iter().map(|t| t.instantiate(i, index)).collect(),
            )),
        }
    }
}
