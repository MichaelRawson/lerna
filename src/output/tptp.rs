use std::sync::Arc;

use tptp::ast;
use tptp::ast::FofFormula::*;
use tptp::ast::FofTerm::*;

use core::Formula::*;
use core::Term::*;
use core::{Bound, Formula, Proof, Set, Term};
use names::symbol_name;

fn to_tptp_bound(b: Bound) -> ast::Bound {
    let Bound(b) = b;
    ast::Bound(Arc::new(format!("X{}", b)))
}

fn to_tptp_term(t: &Arc<Term>) -> Box<ast::FofTerm> {
    Box::new(match **t {
        Var(x) => Variable(to_tptp_bound(x)),
        Fun(f, ref ts) => {
            let ts = ts.iter().map(|t| to_tptp_term(t)).collect();
            let name = ast::Name::Word(symbol_name(f));
            Functor(name, ts)
        }
    })
}

fn to_tptp_formula(f: &Arc<Formula>) -> Box<ast::FofFormula> {
    Box::new(match **f {
        T => Boolean(true),
        F => Boolean(false),
        Eql(ref left, ref right) => Equal(to_tptp_term(left), to_tptp_term(right)),
        Prd(p, ref ts) => {
            let ts = ts.iter().map(|x| to_tptp_term(x)).collect();
            let name = ast::Name::Word(symbol_name(p));
            Predicate(name, ts)
        }
        Not(ref p) => Unary(ast::FofUnaryOp::Not, to_tptp_formula(p)),
        And(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(p)).collect();
            Assoc(ast::FofAssocOp::And, ps)
        }
        Or(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(p)).collect();
            Assoc(ast::FofAssocOp::Or, ps)
        }
        Imp(ref p, ref q) => NonAssoc(
            ast::FofNonAssocOp::Implies,
            to_tptp_formula(p),
            to_tptp_formula(q),
        ),
        Eqv(ref p, ref q) => NonAssoc(
            ast::FofNonAssocOp::Equivalent,
            to_tptp_formula(p),
            to_tptp_formula(q),
        ),
        All(x, ref p) => Quantified(
            ast::FofQuantifier::Forall,
            vec![to_tptp_bound(x)],
            to_tptp_formula(p),
        ),
        Ex(x, ref p) => Quantified(
            ast::FofQuantifier::Exists,
            vec![to_tptp_bound(x)],
            to_tptp_formula(p),
        ),
    })
}

fn to_tptp_statement(index: usize, f: &Arc<Formula>) -> ast::Statement {
    let name = ast::Name::Integer(Arc::new(format!("{}", index)));
    let role = ast::FormulaRole::Plain;
    let formula = to_tptp_formula(f);
    ast::Statement::Fof(name, role, formula)
}

fn print_refutation(done: Set<Arc<Formula>>, mut index: usize, proof: &Proof) -> usize {
    match *proof {
        Proof::Leaf => {
            let statement = to_tptp_statement(index, &Arc::new(Formula::F));
            println!("{}", statement);
            index + 1
        }
        Proof::Branch(ref formulae, ref children) => {
            let formulae: Set<_> = formulae
                .iter()
                .filter(|f| !done.contains(*f))
                .cloned()
                .collect();
            if !formulae.is_empty() {
                let conjunction = Arc::new(Formula::And(formulae.clone()));
                let statement = to_tptp_statement(index, &conjunction);
                println!("{}", statement);
                index += 1;
            }

            let done = done.union(formulae.clone());
            for child in children {
                index = print_refutation(done.clone(), index, child);
            }
            index
        }
    }
}

pub fn szs_refutation(done: Set<Arc<Formula>>, proof: &Proof) {
    println!("% SZS status Refutation");
    println!("% SZS output start");
    print_refutation(done, 0, proof);
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
