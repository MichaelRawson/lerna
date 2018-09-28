use std::sync::Arc;

use tptp::ast;
use tptp::ast::*;
use tptp::prelude::*;

use core;
use core::*;
use util::fail;

fn load_fof_term(core: &Core, bound: Map<ast::Bound, core::Bound>, term: &FofTerm) -> Arc<Term> {
    match term {
        FofTerm::Variable(x) => {
            let x = *bound.get(x).unwrap_or_else(|| {
                error!("unbound variable: {}", x);
                fail()
            });
            Arc::new(Term::Var(x))
        },
        FofTerm::Functor(name, args) => {
            let name = Arc::new(format!("{}", name));
            let arity = args.len();
            let symbol = core.symbol_for(name, arity);
            let args = args.iter()
                .map(|x| load_fof_term(core, bound.clone(), &x))
                .collect();
            Arc::new(Term::Fun(symbol, args))
        }
    }
}

fn load_fof_formula(core: &Core, mut bound: Map<ast::Bound, core::Bound>, formula: &FofFormula) -> Arc<Formula> {
    match formula {
        FofFormula::Boolean(b) => Arc::new(if *b {
            Formula::T
        } else {
            Formula::F
        }),
        FofFormula::Equal(x, y) => Arc::new(Formula::Eql(
            load_fof_term(core, bound.clone(), x),
            load_fof_term(core, bound.clone(), y)
        )),
        FofFormula::Predicate(name, args) => {
            let name = Arc::new(format!("{}", name));
            let arity = args.len();
            let symbol = core.symbol_for(name, arity);
            let args = args.iter()
                .map(|x| load_fof_term(core, bound.clone(), &x))
                .collect();
            Arc::new(Formula::Prd(symbol, args))
        },
        FofFormula::Unary(op, f) => {
            let f = load_fof_formula(core, bound.clone(), f);
            Arc::new(match op {
                FofUnaryOp::Not => Formula::Not(f)
            })
        },
        FofFormula::NonAssoc(op, left, right) => {
            let left = load_fof_formula(core, bound.clone(), left);
            let right = load_fof_formula(core, bound.clone(), right);
            Arc::new(match op {
                FofNonAssocOp::Implies => Formula::Imp(left, right),
                FofNonAssocOp::Equivalent => Formula::Eqv(left, right),
            })
        },
        FofFormula::Assoc(op, args) => {
            let args = args.iter()
                .map(|x| load_fof_formula(core, bound.clone(), &x))
                .collect();

            Arc::new(match op {
                FofAssocOp::And => Formula::And(args),
                FofAssocOp::Or => Formula::Or(args),
            })
        },
        FofFormula::Quantified(quantifier, binders, f) => {
            let quantifier = match quantifier {
                FofQuantifier::Forall => Formula::All,
                FofQuantifier::Exists => Formula::Ex,
            };
            let binders: Vec<core::Bound> = binders.iter().map(|x| {
                let x_bound = core.fresh_binder();
                bound.insert(x.clone(), x_bound);
                x_bound
            }).rev().collect();
            let f = load_fof_formula(core, bound, f);
            binders.iter().fold(f, |f, x| Arc::new(quantifier(*x, f)))
        }
    }
}

fn load_fof(core: &Core, role: FormulaRole, formula: &FofFormula) -> Arc<Formula> {
    let bound = Map::new();
    let formula = load_fof_formula(core, bound, formula);
    match role {
        FormulaRole::Axiom | FormulaRole::Hypothesis | FormulaRole::NegatedConjecture => formula,
        FormulaRole::Conjecture => Formula::negate(formula),
        other => {
            error!("unsupported TPTP formula role: \"{}\"", other);
            fail()
        }
    }
}

fn load_statement(core: &Core, statement: Statement) -> Arc<Formula> {
    match statement {
        Statement::Fof(name, role, formula) => {
            debug!("parsed TPTP input \"{}\"", name);
            let formula = load_fof(core, role, &formula);
            debug!("loaded \"{}\"", name);
            formula
        },
        other => {
            error!("unsupported TPTP input");
            eprintln!("{}", other);
            fail()
        }
    }
}

fn load_or_fail(path: &str, core: &Core) -> Result<Goal, Error> {
    let reader = ReaderBuilder::new().follow_includes().read(path)?;

    let mut formulae = Set::new();
    for statement in reader {
        let statement = load_statement(core, statement?);
        formulae.insert(statement);
    }

    let goal = Goal {formulae};
    Ok(goal)
}

pub fn load(path: &str, core: &Core) -> Goal {
    debug!("parsing TPTP from {:?}", path);

    load_or_fail(path, core).unwrap_or_else(|err| {
        error!("error loading TPTP");
        eprintln!("{:#?}", err);
        fail()
    })
}
