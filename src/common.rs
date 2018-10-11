use std::sync::Arc;

use core::Formula::*;
use core::Term::*;
use core::{Bound, Formula, Goal, Term, Set, Symbol};
use names::{fresh_symbol, symbol_arity};

fn term_symbols(t: &Arc<Term>) -> Set<Symbol> {
    match **t {
        Var(_) => set![],
        Fun(f, ref args) => {
            let mut arg_symbols = Set::unions(args.iter().map(term_symbols));
            arg_symbols.insert(f);
            arg_symbols
        }
    }
}

pub fn formula_symbols(f: &Arc<Formula>) -> Set<Symbol> {
    match **f {
        T | F => set![],
        Eql(ref left, ref right) => term_symbols(left).union(term_symbols(right)),
        Prd(_, ref args) => Set::unions(args.iter().map(term_symbols)),
        Not(ref p) => formula_symbols(p),
        Imp(ref p, ref q) => formula_symbols(p).union(formula_symbols(q)),
        Eqv(ref p, ref q) => formula_symbols(p).union(formula_symbols(q)),
        And(ref ps) => Set::unions(ps.iter().map(formula_symbols)),
        Or(ref ps) => Set::unions(ps.iter().map(formula_symbols)),
        All(ref p) => formula_symbols(p),
        Ex(ref p) => formula_symbols(p),
    }
}

fn replace_in_term(t: &Arc<Term>, to: &Arc<Term>, from: &Arc<Term>) -> Arc<Term> {
    if t == to {
        from.clone()
    } else {
        match **t {
            Var(_) => panic!("replacing non-ground term"),
            Fun(f, ref args) => Arc::new(Fun(
                f,
                args.iter().map(|t| replace_in_term(t, to, from)).collect(),
            )),
        }
    }
}

pub fn replace_in_formula(f: &Arc<Formula>, to: &Arc<Term>, from: &Arc<Term>) -> Arc<Formula> {
    match **f {
        T | F => f.clone(),
        Eql(ref left, ref right) => Arc::new(Eql(
            replace_in_term(left, to, from),
            replace_in_term(right, to, from),
        )),
        Prd(p, ref args) => Arc::new(Prd(
            p,
            args.iter().map(|t| replace_in_term(t, to, from)).collect(),
        )),
        Not(ref p) => Arc::new(Not(replace_in_formula(p, to, from))),
        Imp(ref p, ref q) => Arc::new(Imp(
            replace_in_formula(p, to, from),
            replace_in_formula(q, to, from),
        )),
        Eqv(ref p, ref q) => Arc::new(Eqv(
            replace_in_formula(p, to, from),
            replace_in_formula(q, to, from),
        )),
        And(ref ps) => Arc::new(And(ps
            .iter()
            .map(|p| replace_in_formula(p, to, from))
            .collect())),
        Or(ref ps) => Arc::new(Or(ps
            .iter()
            .map(|p| replace_in_formula(p, to, from))
            .collect())),
        All(ref p) => Arc::new(All(replace_in_formula(p, to, from))),
        Ex(ref p) => Arc::new(Ex(replace_in_formula(p, to, from))),
    }
}

pub fn replace_in_goal(goal: &Goal, to: &Arc<Term>, from: &Arc<Term>) -> Goal {
    Goal::new(
        goal.formulae()
            .map(|f| replace_in_formula(f, to, from))
            .collect(),
    )
}

fn shift_indices(t: &Arc<Term>, shift: usize) -> Arc<Term> {
    match **t {
        Var(Bound(b)) => Arc::new(Var(Bound(b + shift))),
        Fun(f, ref args) => Arc::new(Fun(f,
            args.iter()
                .map(|t| shift_indices(t, shift))
                .collect()
        ))
    }
}

fn instantiate_in_term(t: &Arc<Term>, i: &Arc<Term>, index: usize) -> Arc<Term> {
    match **t {
        Var(Bound(b)) => if b == index {
            shift_indices(i, index)
        }
        else {
            t.clone()
        },
        Fun(f, ref args) => Arc::new(Fun(f,
            args.iter()
                .map(|t| instantiate_in_term(t, i, index))
                .collect()
        ))
    }
}

pub fn instantiate_in_formula(f: &Arc<Formula>, i: &Arc<Term>, index: usize) -> Arc<Formula> {
    match **f {
        T | F => f.clone(),
        Prd(p, ref args) => Arc::new(Prd(p, args.iter()
            .map(|t| instantiate_in_term(t, i, index))
            .collect()
        )),
        Eql(ref left, ref right) => Arc::new(Eql(
            instantiate_in_term(left, i, index),
            instantiate_in_term(right, i, index)
        )),
        Not(ref p) => Arc::new(Not(instantiate_in_formula(p, i, index))),
        Imp(ref left, ref right) => Arc::new(Imp(
            instantiate_in_formula(left, i, index),
            instantiate_in_formula(right, i, index)
        )),
        Eqv(ref left, ref right) => Arc::new(Eqv(
            instantiate_in_formula(left, i, index),
            instantiate_in_formula(right, i, index)
        )),
        And(ref ps) => Arc::new(And(
            ps.iter()
                .map(|p| instantiate_in_formula(p, i, index))
                .collect()
        )),
        Or(ref ps) => Arc::new(Or(
            ps.iter()
                .map(|p| instantiate_in_formula(p, i, index))
                .collect()
        )),
        All(ref p) => Arc::new(All(instantiate_in_formula(p, i, index + 1))),
        Ex(ref p) => Arc::new(Ex(instantiate_in_formula(p, i, index + 1))),
    }
}

pub fn instantiate_with_constant(f: &Arc<Formula>) -> Arc<Formula> {
    let constant = Arc::new(Fun(fresh_symbol(0), vec![]));
    instantiate_in_formula(&f, &constant, 0)
}

pub fn instantiate_with_symbol(f: &Arc<Formula>, symbol: Symbol) -> Arc<Formula> {
    let arity = symbol_arity(symbol);
    let vars = (0..arity)
        .map(|i| Arc::new(Var(Bound(i))))
        .collect();
    let term = Arc::new(Fun(symbol, vars));
    let mut f = instantiate_in_formula(&f, &term, 0);
    for _ in 0..arity {
        f = Arc::new(All(f));
    }
    f
}
