use std::sync::Arc;

use core::Formula::*;
use core::Term::*;
use core::{Bound, Formula, Goal, Set, Term};

use names::{fresh_binder, fresh_symbol};

fn term_binders(t: &Arc<Term>) -> Set<Bound> {
    match **t {
        Var(x) => set![x],
        Fun(_, ref args) => Set::unions(args.iter().map(term_binders)),
    }
}

fn term_subterms(t: &Arc<Term>) -> Set<Arc<Term>> {
    match **t {
        Var(_) => set![],
        Fun(_, ref args) => {
            let mut arg_terms = Set::unions(args.iter().map(term_subterms));
            arg_terms.insert(t.clone());
            arg_terms
        }
    }
}

fn formula_subterms(f: &Arc<Formula>) -> Set<Arc<Term>> {
    match **f {
        T | F => set![],
        Eql(ref left, ref right) => term_subterms(left).union(term_subterms(right)),
        Prd(_, ref args) => Set::unions(args.iter().map(term_subterms)),
        Not(ref p) => formula_subterms(p),
        Imp(ref p, ref q) => formula_subterms(p).union(formula_subterms(q)),
        Eqv(ref p, ref q) => formula_subterms(p).union(formula_subterms(q)),
        And(ref ps) => Set::unions(ps.iter().map(formula_subterms)),
        Or(ref ps) => Set::unions(ps.iter().map(formula_subterms)),
        All(_, ref p) => formula_subterms(p),
        Ex(_, ref p) => formula_subterms(p),
    }
}

pub fn goal_subterms(goal: &Goal) -> Set<Arc<Term>> {
    Set::unions(goal.formulae().map(formula_subterms))
}

fn replace_in_term(t: &Arc<Term>, to: &Arc<Term>, from: &Arc<Term>) -> Arc<Term> {
    if t == to {
        from.clone()
    } else {
        match **t {
            Var(_) => t.clone(),
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
        All(x, ref p) => Arc::new(All(x, replace_in_formula(p, to, from))),
        Ex(x, ref p) => Arc::new(Ex(x, replace_in_formula(p, to, from))),
    }
}

pub fn replace_in_goal(goal: &Goal, to: &Arc<Term>, from: &Arc<Term>) -> Goal {
    Goal::new(
        goal.formulae()
            .map(|f| replace_in_formula(f, to, from))
            .collect(),
    )
}

fn rebind_term(t: &Arc<Term>) -> (Set<Bound>, Arc<Term>) {
    let xs = term_binders(t);
    let mut t = t.clone();
    let fresh = xs
        .into_iter()
        .map(|x| {
            let old = Arc::new(Var(x));
            let fresh = fresh_binder();
            let new = Arc::new(Var(fresh));
            t = replace_in_term(&t, &old, &new);
            fresh
        }).collect();
    (fresh, t)
}

pub fn instantiate_ex(x: Bound, f: &Arc<Formula>) -> Arc<Formula> {
    let v = Arc::new(Var(x));
    let k = Arc::new(Fun(fresh_symbol(), vec![]));
    replace_in_formula(f, &v, &k)
}

pub fn instantiate_all(x: Bound, goal: &Goal, f: &Arc<Formula>) -> Set<Arc<Formula>> {
    let v = Arc::new(Var(x));
    goal_subterms(goal)
        .into_iter()
        .map(|t| {
            let (xs, t) = rebind_term(&t);
            let mut acc = replace_in_formula(f, &v, &t);
            for x in xs {
                acc = Arc::new(All(x, acc));
            }
            acc
        }).collect::<Set<_>>()
        .update(instantiate_ex(x, f))
}
