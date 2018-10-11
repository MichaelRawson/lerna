use std::fmt::Arguments;
use std::sync::Arc;

use fern::FormatCallback;
use log::Record;
use tptp::ast;
use tptp::ast::FofFormula::*;
use tptp::ast::FofTerm::*;

use collections::Set;
use formula::Formula;
use formula::Formula::*;
use proof::RawProof;
use term::Term;
use term::Term::*;

fn to_tptp_bound(bound: usize) -> ast::Bound {
    ast::Bound(Arc::new(format!("X{}", bound)))
}

fn to_tptp_term(t: &Arc<Term>, bound_depth: usize) -> Box<ast::FofTerm> {
    Box::new(match **t {
        Var(x) => Variable(to_tptp_bound(bound_depth - 1 - x)),
        Fun(f, ref ts) => {
            let ts = ts.iter().map(|t| to_tptp_term(t, bound_depth)).collect();
            let name = ast::Name::Word(f.name());
            Functor(name, ts)
        }
    })
}

fn to_tptp_formula(f: &Arc<Formula>, bound_depth: usize) -> Box<ast::FofFormula> {
    Box::new(match **f {
        T => Boolean(true),
        F => Boolean(false),
        Eql(ref left, ref right) => Equal(
            to_tptp_term(left, bound_depth),
            to_tptp_term(right, bound_depth),
        ),
        Prd(p, ref ts) => {
            let ts = ts.iter().map(|x| to_tptp_term(x, bound_depth)).collect();
            let name = ast::Name::Word(p.name());
            Predicate(name, ts)
        }
        Not(ref p) => Unary(ast::FofUnaryOp::Not, to_tptp_formula(p, bound_depth)),
        And(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(p, bound_depth)).collect();
            Assoc(ast::FofAssocOp::And, ps)
        }
        Or(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(p, bound_depth)).collect();
            Assoc(ast::FofAssocOp::Or, ps)
        }
        Imp(ref p, ref q) => NonAssoc(
            ast::FofNonAssocOp::Implies,
            to_tptp_formula(p, bound_depth),
            to_tptp_formula(q, bound_depth),
        ),
        Eqv(ref p, ref q) => NonAssoc(
            ast::FofNonAssocOp::Equivalent,
            to_tptp_formula(p, bound_depth),
            to_tptp_formula(q, bound_depth),
        ),
        All(ref p) => Quantified(
            ast::FofQuantifier::Forall,
            vec![to_tptp_bound(bound_depth)],
            to_tptp_formula(p, bound_depth + 1),
        ),
        Ex(ref p) => Quantified(
            ast::FofQuantifier::Exists,
            vec![to_tptp_bound(bound_depth)],
            to_tptp_formula(p, bound_depth + 1),
        ),
    })
}

fn to_tptp_statement(index: usize, f: &Arc<Formula>) -> ast::Statement {
    let name = ast::Name::Integer(Arc::new(format!("{}", index)));
    let role = ast::FormulaRole::Plain;
    let formula = to_tptp_formula(f, 0);
    ast::Statement::Fof(name, role, formula)
}

fn print_refutation(done: Set<Arc<Formula>>, mut index: usize, proof: &RawProof) -> usize {
    match *proof {
        RawProof::Leaf => {
            let statement = to_tptp_statement(index, &Arc::new(Formula::F));
            println!("{}", statement);
            index + 1
        }
        RawProof::Branch(ref goal, ref children) => {
            let formulae: Set<_> = goal
                .formulae()
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

pub fn szs_refutation(name: &str, done: Set<Arc<Formula>>, proof: &RawProof) {
    println!("% SZS status Theorem for {}", name);
    println!("% SZS output start Refutation for {}", name);
    print_refutation(done, 0, proof);
    println!("% SZS output end Refutation for {}", name);
    debug!("...statements printed");
}

pub fn szs_timeout(name: &str) {
    println!("% SZS status TimeOut for {}", name);
}

pub fn szs_os_error(name: &str) {
    println!("% SZS status OSError for {}", name);
}

pub fn szs_input_error(name: &str) {
    println!("% SZS status InputError for {}", name);
}

pub fn szs_inappropriate(name: &str) {
    println!("% SZS status Inappropriate for {}", name);
}

pub fn format_log(out: FormatCallback, message: &Arguments, record: &Record) {
    out.finish(format_args!(
        "% [{}][{}] {}",
        record.level(),
        record.target(),
        message
    ))
}
