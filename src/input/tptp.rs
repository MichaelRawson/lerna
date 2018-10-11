use std::sync::Arc;

use tptp::ast;
use tptp::ast::*;
use tptp::prelude::*;

use collections::{Map, Set};
use formula::Formula;
use goal::Goal;
use input::LoadError;
use symbol::Symbol;
use term::Term;
use term::Term::*;

fn load_fof_term(
    bound: &Map<ast::Bound, usize>,
    bound_depth: usize,
    term: &FofTerm,
) -> Result<Arc<Term>, LoadError> {
    match term {
        FofTerm::Variable(x) => match bound.get(x) {
            Some(x) => Ok(Arc::new(Var(bound_depth - 1 - x))),
            None => {
                error!("unbound variable: {}", x);
                Err(LoadError::InputError)
            }
        },
        FofTerm::Functor(name, fof_args) => {
            let name = Arc::new(format!("{}", name));
            let arity = fof_args.len();
            let symbol = Symbol::get(&name, arity);

            let mut args = vec![];
            for arg in fof_args {
                args.push(load_fof_term(&bound, bound_depth, arg)?);
            }
            Ok(Arc::new(Fun(symbol, args)))
        }
    }
}

fn load_fof_formula(
    mut bound: Map<ast::Bound, usize>,
    bound_depth: usize,
    formula: &FofFormula,
) -> Result<Arc<Formula>, LoadError> {
    Ok(match formula {
        FofFormula::Boolean(b) => Arc::new(if *b { Formula::T } else { Formula::F }),
        FofFormula::Equal(x, y) => Arc::new(Formula::Eql(
            load_fof_term(&bound, bound_depth, x)?,
            load_fof_term(&bound, bound_depth, y)?,
        )),
        FofFormula::Predicate(name, fof_args) => {
            let name = Arc::new(format!("{}", name));
            let arity = fof_args.len();
            let symbol = Symbol::get(&name, arity);

            let mut args = vec![];
            for arg in fof_args {
                args.push(load_fof_term(&bound, bound_depth, arg)?);
            }
            Arc::new(Formula::Prd(symbol, args))
        }
        FofFormula::Unary(op, f) => {
            let f = load_fof_formula(bound.clone(), bound_depth, f)?;
            Arc::new(match op {
                FofUnaryOp::Not => Formula::Not(f),
            })
        }
        FofFormula::NonAssoc(op, left, right) => {
            let left = load_fof_formula(bound.clone(), bound_depth, left)?;
            let right = load_fof_formula(bound.clone(), bound_depth, right)?;
            Arc::new(match op {
                FofNonAssocOp::Implies => Formula::Imp(left, right),
                FofNonAssocOp::Equivalent => Formula::Eqv(left, right),
            })
        }
        FofFormula::Assoc(op, fof_args) => {
            let mut args = set![];
            for arg in fof_args {
                args.insert(load_fof_formula(bound.clone(), bound_depth, arg)?);
            }
            Arc::new(match op {
                FofAssocOp::And => Formula::And(args),
                FofAssocOp::Or => Formula::Or(args),
            })
        }
        FofFormula::Quantified(quantifier, binders, f) => {
            let quantifier = match quantifier {
                FofQuantifier::Forall => Formula::All,
                FofQuantifier::Exists => Formula::Ex,
            };
            let num_bound = binders.len();

            let mut depth = bound_depth;
            for binder in binders {
                bound.insert(binder.clone(), depth);
                depth += 1;
            }

            let mut f = load_fof_formula(bound, depth, f)?;
            for _ in 0..num_bound {
                f = Arc::new(quantifier(f));
            }
            f
        }
    })
}

fn load_fof(role: FormulaRole, formula: &FofFormula) -> Result<Arc<Formula>, LoadError> {
    let bound = Map::new();
    let formula = load_fof_formula(bound, 0, formula)?;
    match role {
        FormulaRole::Axiom
        | FormulaRole::Hypothesis
        | FormulaRole::Definition
        | FormulaRole::Lemma
        | FormulaRole::Theorem
        | FormulaRole::Corollary
        | FormulaRole::NegatedConjecture => Ok(formula),
        FormulaRole::Conjecture => Ok(formula.negated()),
        other => {
            error!("unsupported TPTP formula role: \"{}\"", other);
            Err(LoadError::Unsupported)
        }
    }
}

fn load_statement(statement: Statement) -> Result<Arc<Formula>, LoadError> {
    match statement {
        Statement::Fof(name, role, formula) => {
            debug!("encountered TPTP input \"{}\"", name);
            let formula = load_fof(role, &formula)?;
            debug!("loaded \"{}\"", name);
            Ok(formula)
        }
        other => {
            error!("unsupported TPTP input \"{}\"", other);
            Err(LoadError::Unsupported)
        }
    }
}

fn convert_error(e: Error) -> LoadError {
    error!("error loading TPTP from {:?}", e.includes.last().unwrap());
    match e.reported {
        Reported::IO(e) => {
            error!("IO error: {:?}", e);
            LoadError::OSError
        }
        Reported::Lexical(e) => {
            error!("lexical error: {:?}", e);
            LoadError::InputError
        }
        Reported::Syntactic(Syntactic::UnsupportedDialect(p, dialect)) => {
            error!("unsupported TPTP dialect \"{}\" at {}", dialect, p);
            LoadError::Unsupported
        }
        Reported::Syntactic(e) => {
            error!("syntax error: {:?}", e);
            LoadError::InputError
        }
        Reported::Include(e) => {
            error!("include error: {:?}", e);
            LoadError::InputError
        }
    }
}

pub fn load(path: &str) -> Result<Goal, LoadError> {
    debug!("parsing TPTP from {:?}...", path);
    let reader = ReaderBuilder::new()
        .follow_includes()
        .read(path)
        .map_err(convert_error)?;

    let mut formulae = Set::new();
    for input in reader {
        let (_file, _position, statement) = input.map_err(convert_error)?;
        let statement = load_statement(statement)?;
        formulae.insert(statement);
    }
    debug!("...parsing done");

    Ok(Goal::new(formulae))
}
