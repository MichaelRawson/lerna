use crate::types::Dag;

use tptp;
use tptp::error::*;
use tptp::syntax::*;

use crate::formula::Formula;
use crate::goal::Goal;
use crate::input::LoadError;
use crate::symbol::{Flavour, Symbol};
use crate::term::Term;
use crate::term::Term::*;
use crate::types::{Map, Set};

fn load_fof_name(name: Name) -> String {
    match name {
        Name::Atomic(x) => x,
        Name::Integer(x) => x,
    }
}

fn load_fof_term(
    bound: &Map<String, usize>,
    bound_depth: usize,
    term: FofTerm,
) -> Result<Dag<Term>, LoadError> {
    match term {
        FofTerm::Variable(x) => match bound.get(&x) {
            Some(x) => Ok(dag!(Var(bound_depth - 1 - x))),
            None => {
                error!("unbound variable: {}", x);
                Err(LoadError::InputError)
            }
        },
        FofTerm::Functor(name, fof_args) => {
            let name = load_fof_name(name);
            let arity = fof_args.len();
            let symbol = Symbol::get(name, arity, Flavour::Functor);

            let mut args = vec![];
            for arg in fof_args {
                args.push(load_fof_term(&bound, bound_depth, *arg)?);
            }
            Ok(dag!(Fun(symbol, args)))
        }
        FofTerm::DistinctObject(name) => {
            let symbol = Symbol::get(name, 0, Flavour::Distinct);
            Ok(dag!(Fun(symbol, vec![])))
        }
    }
}

fn load_fof_formula(
    mut bound: Map<String, usize>,
    bound_depth: usize,
    formula: FofFormula,
) -> Result<Dag<Formula>, LoadError> {
    Ok(match formula {
        FofFormula::Boolean(b) => dag!(if b { Formula::T } else { Formula::F }),
        FofFormula::Infix(op, x, y) => {
            let equality = dag!(Formula::Eql(
                load_fof_term(&bound, bound_depth, *x)?,
                load_fof_term(&bound, bound_depth, *y)?,
            ));
            use self::InfixEquality::*;
            match op {
                Equal => equality,
                NotEqual => equality.negated(),
            }
        }
        FofFormula::Predicate(name, fof_args) => {
            let name = load_fof_name(name);
            let arity = fof_args.len();
            let symbol = Symbol::get(name, arity, Flavour::Functor);

            let mut args = vec![];
            for arg in fof_args {
                args.push(load_fof_term(&bound, bound_depth, *arg)?);
            }
            dag!(Formula::Prd(symbol, args))
        }
        FofFormula::Unary(op, f) => {
            let f = load_fof_formula(bound.clone(), bound_depth, *f)?;
            dag!(match op {
                FofUnaryConnective::Not => Formula::Not(f),
            })
        }
        FofFormula::NonAssoc(op, left, right) => {
            let left = load_fof_formula(bound.clone(), bound_depth, *left)?;
            let right = load_fof_formula(bound.clone(), bound_depth, *right)?;
            dag!(match op {
                FofNonAssocConnective::LRImplies => Formula::Imp(left, right),
                FofNonAssocConnective::RLImplies => Formula::Imp(right, left),
                FofNonAssocConnective::Equivalent => Formula::Eqv(left, right),
                FofNonAssocConnective::NotEquivalent => {
                    Formula::Not(dag!(Formula::Eqv(left, right)))
                }
                FofNonAssocConnective::NotOr => {
                    Formula::Not(dag!(Formula::Or(set![left, right])))
                }
                FofNonAssocConnective::NotAnd => {
                    Formula::Not(dag!(Formula::And(set![left, right])))
                }
            })
        }
        FofFormula::Assoc(op, fof_args) => {
            let mut args = set![];
            for arg in fof_args {
                args.insert(load_fof_formula(bound.clone(), bound_depth, *arg)?);
            }
            dag!(match op {
                FofAssocConnective::And => Formula::And(args),
                FofAssocConnective::Or => Formula::Or(args),
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

            let mut f = load_fof_formula(bound, depth, *f)?;
            for _ in 0..num_bound {
                f = dag!(quantifier(f));
            }
            f
        }
    })
}

fn should_negate(role: FormulaRole) -> Result<bool, LoadError> {
    match role {
        FormulaRole::Axiom
        | FormulaRole::Hypothesis
        | FormulaRole::Definition
        | FormulaRole::Lemma
        | FormulaRole::Theorem
        | FormulaRole::Corollary
        | FormulaRole::NegatedConjecture => Ok(false),
        FormulaRole::Conjecture => Ok(true),
        other => {
            error!("unsupported TPTP formula role: \"{}\"", other);
            Err(LoadError::Unsupported)
        }
    }
}

fn load_fof(role: FormulaRole, formula: FofFormula) -> Result<Dag<Formula>, LoadError> {
    let bound = map![];
    let formula = load_fof_formula(bound, 0, formula)?;
    if should_negate(role)? {
        Ok(formula.negated())
    } else {
        Ok(formula)
    }
}

fn fof_term_bound(term: &FofTerm) -> Set<String> {
    match *term {
        FofTerm::Variable(ref x) => set![x.clone()],
        FofTerm::Functor(_, ref args) => Set::unions(args.iter().map(|arg| fof_term_bound(arg))),
        FofTerm::DistinctObject(_) => set![],
    }
}

fn cnf_literal_bound(literal: &CnfLiteral) -> Set<String> {
    let literal = match literal {
        CnfLiteral::Literal(f) => f,
        CnfLiteral::NegatedLiteral(f) => f,
    };

    match **literal {
        FofFormula::Boolean(_) => set![],
        FofFormula::Infix(_, ref left, ref right) => {
            fof_term_bound(left).union(fof_term_bound(right))
        }
        FofFormula::Predicate(_, ref args) => {
            Set::unions(args.iter().map(|arg| fof_term_bound(arg)))
        }
        _ => unreachable!(),
    }
}

fn cnf_bound(formula: &CnfFormula) -> Set<String> {
    Set::unions(formula.0.iter().map(cnf_literal_bound))
}

fn cnf_literal_to_fof(literal: CnfLiteral) -> Box<FofFormula> {
    match literal {
        CnfLiteral::Literal(f) => f,
        CnfLiteral::NegatedLiteral(f) => Box::new(FofFormula::Unary(FofUnaryConnective::Not, f)),
    }
}

fn load_cnf(role: FormulaRole, formula: CnfFormula) -> Result<Dag<Formula>, LoadError> {
    let bound = cnf_bound(&formula).into_iter().collect();
    let mut disjunction = FofFormula::Assoc(
        FofAssocConnective::Or,
        formula.0.into_iter().map(cnf_literal_to_fof).collect(),
    );

    if should_negate(role)? {
        disjunction = FofFormula::Unary(FofUnaryConnective::Not, Box::new(disjunction));
    }

    let quantified = FofFormula::Quantified(FofQuantifier::Forall, bound, Box::new(disjunction));
    load_fof_formula(map![], 0, quantified)
}

fn load_statement(statement: Statement) -> Result<Dag<Formula>, LoadError> {
    match statement {
        Statement::Fof(name, role, formula, _) => {
            debug!("encountered FOF input \"{}\"", name);
            let formula = load_fof(role, *formula)?;
            debug!("loaded \"{}\"", name);
            Ok(formula)
        }
        Statement::Cnf(name, role, formula, _) => {
            debug!("encountered CNF input \"{}\"", name);
            let formula = load_cnf(role, formula)?;
            debug!("loaded \"{}\"", name);
            Ok(formula)
        }
        other => {
            error!("unsupported TPTP input \"{}\"", other);
            Err(LoadError::Unsupported)
        }
    }
}

fn convert_error(e: ErrorInfo) -> LoadError {
    error!(
        "error loading TPTP from {:?} at {}",
        e.includes.last().unwrap(),
        e.position
    );
    use tptp::error::Error::*;
    match e.error {
        System(e) => {
            error!("IO error: {:?}", e);
            LoadError::OSError
        }
        Lexical(e) => {
            error!("lexical error: {:?}", e);
            LoadError::InputError
        }
        Syntactic(SyntacticError::UnsupportedDialect(dialect)) => {
            error!("unsupported TPTP dialect \"{}\"", dialect);
            LoadError::Unsupported
        }
        Syntactic(e) => {
            error!("syntax error: {:?}", e);
            LoadError::InputError
        }
        Include(e) => {
            error!("include error: {:?}", e);
            LoadError::InputError
        }
    }
}

pub fn load(path: &str) -> Result<Goal, LoadError> {
    debug!("parsing TPTP from {:?}...", path);
    let mut formulae = Set::new();
    for input in tptp::stream(path).map_err(convert_error)? {
        let statement = load_statement(input.map_err(convert_error)?)?;
        formulae.insert(statement);
    }
    debug!("...parsing done");

    Ok(Goal::new(formulae))
}
