use crate::types::Dag;
use std::fmt::Arguments;

use fern::FormatCallback;
use log::Record;
use tptp::syntax;
use tptp::syntax::FofFormula::*;
use tptp::syntax::FofTerm::*;

use crate::formula::Formula;
use crate::formula::Formula::*;
use crate::proof::Proof;
use crate::symbol::Flavour;
use crate::term::Term;
use crate::term::Term::*;
use crate::types::Set;

fn to_tptp_bound(bound: usize) -> String {
    format!("X{}", bound)
}

fn to_tptp_term(t: &Dag<Term>, bound_depth: usize) -> Box<syntax::FofTerm> {
    Box::new(match **t {
        Var(x) => Variable(to_tptp_bound(bound_depth - 1 - x)),
        Fun(f, ref ts) => match f.flavour() {
            Flavour::Functor => {
                let ts = ts.iter().map(|t| to_tptp_term(t, bound_depth)).collect();
                let name = syntax::Name::Atomic(f.name());
                Functor(name, ts)
            }
            Flavour::Distinct => DistinctObject(f.name()),
        },
    })
}

fn to_tptp_formula(f: &Dag<Formula>, bound_depth: usize) -> Box<syntax::FofFormula> {
    Box::new(match **f {
        T => Boolean(true),
        F => Boolean(false),
        Eql(ref left, ref right) => Infix(
            syntax::InfixEquality::Equal,
            to_tptp_term(left, bound_depth),
            to_tptp_term(right, bound_depth),
        ),
        Prd(p, ref ts) => {
            let ts = ts.iter().map(|x| to_tptp_term(x, bound_depth)).collect();
            let name = syntax::Name::Atomic(p.name());
            Predicate(name, ts)
        }
        Not(ref p) => Unary(
            syntax::FofUnaryConnective::Not,
            to_tptp_formula(p, bound_depth),
        ),
        And(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(p, bound_depth)).collect();
            Assoc(syntax::FofAssocConnective::And, ps)
        }
        Or(ref ps) => {
            let ps = ps.iter().map(|p| to_tptp_formula(p, bound_depth)).collect();
            Assoc(syntax::FofAssocConnective::Or, ps)
        }
        Imp(ref p, ref q) => NonAssoc(
            syntax::FofNonAssocConnective::LRImplies,
            to_tptp_formula(p, bound_depth),
            to_tptp_formula(q, bound_depth),
        ),
        Eqv(ref p, ref q) => NonAssoc(
            syntax::FofNonAssocConnective::Equivalent,
            to_tptp_formula(p, bound_depth),
            to_tptp_formula(q, bound_depth),
        ),
        All(ref p) => Quantified(
            syntax::FofQuantifier::Forall,
            vec![to_tptp_bound(bound_depth)],
            to_tptp_formula(p, bound_depth + 1),
        ),
        Ex(ref p) => Quantified(
            syntax::FofQuantifier::Exists,
            vec![to_tptp_bound(bound_depth)],
            to_tptp_formula(p, bound_depth + 1),
        ),
    })
}

fn to_tptp_statement(index: usize, f: &Dag<Formula>) -> syntax::Statement {
    let name = syntax::Name::Integer(format!("{}", index));
    let role = syntax::FormulaRole::Plain;
    let formula = to_tptp_formula(f, 0);
    syntax::Statement::Fof(name, role, formula, None)
}

fn print_refutation(start: &Set<Dag<Formula>>, proof: Proof, mut index: usize) -> usize {
    let formulae = proof.goal.refutation();
    let fresh: Set<_> = formulae
        .iter()
        .filter(|f| !start.contains(*f))
        .cloned()
        .collect();

    if !fresh.is_empty() {
        let conjunction = dag!(Formula::And(fresh));
        let statement = to_tptp_statement(index, &conjunction);
        println!("{}", statement);
        index += 1;
    }

    for child in proof.children {
        index = print_refutation(&formulae, child, index);
    }

    index
}

pub fn szs_refutation(name: &str, start: &Set<Dag<Formula>>, proof: Proof) {
    println!("% SZS status Theorem for {}", name);
    println!("% SZS output start Refutation for {}", name);
    print_refutation(start, proof, 0);
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
