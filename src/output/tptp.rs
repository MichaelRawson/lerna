use std::sync::Arc;
use std::vec::Vec;

use tptp::ast;
use tptp::ast::FofFormula::*;
use tptp::ast::FofTerm::*;

use core::Formula::*;
use core::Term::*;
use core::{Bound, Core, Formula, Term};

fn to_tptp_bound(b: Bound) -> ast::Bound {
    let Bound(b) = b;
    ast::Bound(Arc::new(format!("X{}", b)))
}

fn to_tptp_term(core: &Core, t: Arc<Term>) -> Box<ast::FofTerm> {
    Box::new(match &*t {
        Var(x) => Variable(to_tptp_bound(*x)),
        Fun(f, ts) => {
            let ts = ts.iter().map(|t| to_tptp_term(core, t.clone())).collect();
            let name = ast::Name::Word(core.name_of_symbol(f));
            Functor(name, ts)
        }
    })
}

fn to_tptp_formula(core: &Core, f: Arc<Formula>) -> Box<ast::FofFormula> {
    Box::new(match &*f {
        T => Boolean(true),
        F => Boolean(false),
        Eql(left, right) => Equal(
            to_tptp_term(core, left.clone()),
            to_tptp_term(core, right.clone()),
        ),
        Prd(p, ts) => {
            let ts = ts.iter().map(|x| to_tptp_term(core, x.clone())).collect();
            let name = ast::Name::Word(core.name_of_symbol(p));
            Predicate(name, ts)
        }
        Not(p) => Unary(ast::FofUnaryOp::Not, to_tptp_formula(core, p.clone())),
        And(ps) => {
            let ps = ps
                .iter()
                .map(|p| to_tptp_formula(core, p.clone()))
                .collect();
            Assoc(ast::FofAssocOp::And, ps)
        }
        Or(ps) => {
            let ps = ps
                .iter()
                .map(|p| to_tptp_formula(core, p.clone()))
                .collect();
            Assoc(ast::FofAssocOp::Or, ps)
        }
        Imp(p, q) => NonAssoc(
            ast::FofNonAssocOp::Implies,
            to_tptp_formula(core, p.clone()),
            to_tptp_formula(core, q.clone()),
        ),
        Eqv(p, q) => NonAssoc(
            ast::FofNonAssocOp::Equivalent,
            to_tptp_formula(core, p.clone()),
            to_tptp_formula(core, q.clone()),
        ),
        All(x, p) => Quantified(
            ast::FofQuantifier::Forall,
            vec![to_tptp_bound(*x)],
            to_tptp_formula(core, p.clone()),
        ),
        Ex(x, p) => Quantified(
            ast::FofQuantifier::Exists,
            vec![to_tptp_bound(*x)],
            to_tptp_formula(core, p.clone()),
        ),
    })
}

fn to_tptp_statement(core: &Core, index: usize, f: Arc<Formula>) -> ast::Statement {
    let name = ast::Name::Integer(Arc::new(format!("{}", index)));
    let role = ast::FormulaRole::Plain;
    let formula = to_tptp_formula(core, f);
    ast::Statement::Fof(name, role, formula)
}

pub fn szs_refutation(core: &Core, proof: Vec<Arc<Formula>>) {
    debug!("printing {} TPTP statments...", proof.len());
    println!("% SZS status Refutation");
    println!("% SZS output start");
    for (i, f) in proof.into_iter().enumerate() {
        let statement = to_tptp_statement(core, i, f);
        println!("{}", statement);
    }
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
