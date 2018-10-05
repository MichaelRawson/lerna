use std::sync::Arc;

use tptp::ast;
use tptp::ast::FofFormula::*;
use tptp::ast::FofTerm::*;

use core::Formula::*;
use core::Term::*;
use core::{Bound, Names, Formula, Term, Set, Proof};

fn to_tptp_bound(b: Bound) -> ast::Bound {
    let Bound(b) = b;
    ast::Bound(Arc::new(format!("X{}", b)))
}

fn to_tptp_term(names: &Names, t: &Arc<Term>) -> Box<ast::FofTerm> {
    Box::new(match **t {
        Var(x) => Variable(to_tptp_bound(x)),
        Fun(f, ref ts) => {
            let ts = ts.iter().map(|t| to_tptp_term(names, t)).collect();
            let name = ast::Name::Word(names.symbol_name(f));
            Functor(name, ts)
        }
    })
}

fn to_tptp_formula(names: &Names, f: &Arc<Formula>) -> Box<ast::FofFormula> {
    Box::new(match **f {
        T => Boolean(true),
        F => Boolean(false),
        Eql(ref left, ref right) => Equal(to_tptp_term(names, left), to_tptp_term(names, right)),
        Prd(p, ref ts) => {
            let ts = ts.iter().map(|x| to_tptp_term(names, x)).collect();
            let name = ast::Name::Word(names.symbol_name(p));
            Predicate(name, ts)
        }
        Not(ref p) => Unary(ast::FofUnaryOp::Not, to_tptp_formula(names, p)),
        And(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(names, p)).collect();
            Assoc(ast::FofAssocOp::And, ps)
        }
        Or(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(names, p)).collect();
            Assoc(ast::FofAssocOp::Or, ps)
        }
        Imp(ref p, ref q) => NonAssoc(
            ast::FofNonAssocOp::Implies,
            to_tptp_formula(names, p),
            to_tptp_formula(names, q),
        ),
        Eqv(ref p, ref q) => NonAssoc(
            ast::FofNonAssocOp::Equivalent,
            to_tptp_formula(names, p),
            to_tptp_formula(names, q),
        ),
        All(x, ref p) => Quantified(
            ast::FofQuantifier::Forall,
            vec![to_tptp_bound(x)],
            to_tptp_formula(names, p),
        ),
        Ex(x, ref p) => Quantified(
            ast::FofQuantifier::Exists,
            vec![to_tptp_bound(x)],
            to_tptp_formula(names, p),
        ),
    })
}

fn to_tptp_statement(names: &Names, index: usize, f: &Arc<Formula>) -> ast::Statement {
    let name = ast::Name::Integer(Arc::new(format!("{}", index)));
    let role = ast::FormulaRole::Plain;
    let formula = to_tptp_formula(names, f);
    ast::Statement::Fof(name, role, formula)
}

fn print_refutation(names: &Names, done: Set<Arc<Formula>>, mut index: usize, proof: &Box<Proof>) -> usize {
    match **proof {
        Proof::Leaf => {
            let statement = to_tptp_statement(names, index, &Arc::new(Formula::F));
            println!("{}", statement);
            index + 1
        },
        Proof::Branch(ref formulae, ref children) => {
            for f in formulae {
                if !done.contains(f) {
                    let statement = to_tptp_statement(names, index, f);
                    println!("{}", statement);
                    index += 1;
                }
            }
            let done = done.union(formulae.clone());
            for child in children {
                index = print_refutation(names, done.clone(), index, child);
            }
            index
        }
    }
}

pub fn szs_refutation(names: &Names, done: Set<Arc<Formula>>, proof: &Box<Proof>) {
    println!("% SZS status Refutation");
    println!("% SZS output start");
    print_refutation(names, done, 0, proof);
    println!("% SZS output end");
    debug!("...statements printed");
}

pub fn szs_timeout() {
    println!("% SZS status TimeOut");
}

pub fn szs_os_error() {
    println!("% SZS status OSError");
}

pub fn szs_input_error() {
    println!("% SZS status InputError");
}

pub fn szs_inappropriate() {
    println!("% SZS status Inappropriate");
}
