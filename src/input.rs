use memmap::Mmap;
use smallvec::smallvec;
use std::io;
use tptp::syntax::*;
use tptp::{parse, resolve_include};
use unique::Id;

use crate::collections::{list, List, Set};
use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::symbol::Symbol;
use crate::system::{check_for_timeout, input_error, os_error};

#[derive(Default)]
struct Loading {
    axioms: List<Id<Formula>>,
    negated_conjecture: Option<Id<Formula>>,
}

pub struct Loaded {
    pub axioms: Set<Formula>,
    pub negated_conjecture: Id<Formula>,
}

impl Loading {
    fn add_formula(
        &mut self,
        _name: &str,
        role: FormulaRole,
        formula: Id<Formula>,
    ) {
        use self::FormulaRole::*;
        let formula = match role {
            Conjecture => Id::new(Formula::Not(formula)),
            _ => formula,
        };

        let is_conjecture = match role {
            Conjecture | NegatedConjecture => true,
            _ => false,
        };

        if is_conjecture {
            if self.negated_conjecture.is_some() {
                log::error!("multiple conjectures present");
                return input_error();
            }

            self.negated_conjecture = Some(formula);
        } else {
            self.axioms.push(formula);
        }
    }

    fn symbol(&mut self, name: &str) -> Id<Symbol> {
        Id::new(Symbol::Original(name.into()))
    }

    fn finish(self) -> Loaded {
        let axioms = self.axioms.into();
        let negated_conjecture = self.negated_conjecture.unwrap_or_else(|| {
            log::error!("no conjecture was found to prove");
            input_error()
        });

        Loaded {
            axioms,
            negated_conjecture,
        }
    }
}

fn open_file(included: &Included) -> io::Result<Option<Mmap>> {
    let file = resolve_include(included.clone())?;
    let size = file.metadata()?.len();

    // can't mmap() an empty file
    if size == 0 {
        Ok(None)
    } else {
        let mapped = unsafe { Mmap::map(&file) }?;
        Ok(Some(mapped))
    }
}

fn load_cnf_formula(formula: CnfFormula, loading: &mut Loading) -> Id<Formula> {
    use self::CnfLiteral::*;
    use self::Formula::*;

    let literals = formula
        .0
        .into_iter()
        .map(|l| match l {
            Literal(f) => load_fof_formula(f, loading),
            NegatedLiteral(f) => Formula::negate(&load_fof_formula(f, loading)),
        })
        .collect();
    Id::new(Or(literals))
}

fn load_fof_formula(formula: FofFormula, loading: &mut Loading) -> Id<Formula> {
    use self::FofFormula::*;
    use self::Formula::*;
    match formula {
        Boolean(true) => Id::new(T),
        Boolean(false) => Id::new(F),
        Infix(_op, _left, _right) => {
            log::error!("infix operators not yet supported");
            input_error()
        }
        Predicate(name, children) => {
            if children.is_empty() {
                let symbol = loading.symbol(name.as_ref());
                Id::new(Prd(symbol))
            } else {
                log::error!("non-empty predicates not yet supported");
                input_error()
            }
        }
        Unary(UnaryConnective::Not, f) => {
            Id::new(Not(load_fof_formula(*f, loading)))
        }
        NonAssoc(connective, left, right) => {
            use self::NonAssocConnective::*;
            let left = load_fof_formula(*left, loading);
            let right = load_fof_formula(*right, loading);
            match connective {
                LRImplies => Id::new(Imp(left, right)),
                RLImplies => Id::new(Imp(right, left)),
                Equivalent => Id::new(Eqv(left, right)),
                NotEquivalent => Formula::negate(&Id::new(Eqv(left, right))),
                NotOr => Formula::negate(&Id::new(Or(set![left, right]))),
                NotAnd => Formula::negate(&Id::new(And(set![left, right]))),
            }
        }
        Assoc(connective, children) => {
            let children = children
                .into_iter()
                .map(|f| load_fof_formula(f, loading))
                .collect();

            Id::new(match connective {
                AssocConnective::Or => Or(children),
                AssocConnective::And => And(children),
            })
        }
        Quantified(_quantifier, _bound, _formula) => {
            log::error!("quantified formulae not yet supported");
            input_error()
        }
    }
}

fn load_statement<'a>(statement: Statement<'a>, loading: &'a mut Loading) {
    use self::Statement::*;
    match statement {
        Include(included, None) => load_file(&included, loading),
        Include(_path, _selections) => {
            log::error!("include() selections are not supported");
            input_error()
        }
        Cnf(name, role, formula, _annotations) => {
            let formula = load_cnf_formula(formula, loading);
            loading.add_formula(name.as_ref(), role, formula);
        }
        Fof(name, role, formula, _annotations) => {
            let formula = load_fof_formula(formula, loading);
            loading.add_formula(name.as_ref(), role, formula);
        }
    }
}

fn load_file(path: &Included, loading: &mut Loading) {
    log::info!("loading from '{}'...", path);

    let mapped = open_file(&path).unwrap_or_else(|e| {
        log::error!("OS error: {}", e);
        os_error()
    });
    check_for_timeout();

    let bytes = &*match mapped {
        Some(bytes) => bytes,
        None => return,
    };

    for result in parse(bytes) {
        check_for_timeout();
        let statement = result.unwrap_or_else(|e| {
            let start = bytes.as_ptr() as usize;
            let position = e.position.as_ptr() as usize;
            let offset = position - start;
            log::error!(
                "syntax error: in '{}', starting at byte {}",
                path,
                offset
            );
            input_error()
        });
        load_statement(statement, loading);
    }
}

pub fn load() -> Loaded {
    let start = &OPTIONS.file;
    let mut loading = Loading::default();
    load_file(&Included(&start), &mut loading);
    let loaded = loading.finish();

    log::info!(
        "...load complete, read {} axiom(s) and 1 conjecture",
        loaded.axioms.len()
    );
    loaded
}
