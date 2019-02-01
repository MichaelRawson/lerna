use log::*;
use memmap::Mmap;
use std::io;
use tptp::syntax::*;
use tptp::{parse, resolve_include};
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::set::Set;
use crate::symbol::Symbol;
use crate::system::{input_error, os_error};

#[derive(Default)]
struct Loading {
    axioms: Vec<Id<Formula>>,
    negated_conjecture: Option<Id<Formula>>,
}

pub struct Loaded {
    pub axioms: Id<Set<Formula>>,
    pub negated_conjecture: Id<Formula>,
}

impl Loading {
    fn add_formula(&mut self, _name: &str, role: FormulaRole, formula: Id<Formula>) {
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
                error!("multiple conjectures present");
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
        let axioms = Id::new(Set::new_from_vec(self.axioms));
        let negated_conjecture = self.negated_conjecture.unwrap_or_else(|| {
            error!("no conjecture was found to prove");
            input_error()
        });

        Loaded {
            axioms,
            negated_conjecture,
        }
    }
}

fn open_file(path: &str) -> io::Result<Option<Mmap>> {
    let file = resolve_include(path)?;
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
            NegatedLiteral(f) => Id::new(Not(load_fof_formula(f, loading))),
        })
        .collect();
    Id::new(Or(Id::new(literals)))
}

fn load_fof_formula(formula: FofFormula, loading: &mut Loading) -> Id<Formula> {
    use self::FofFormula::*;
    use self::Formula::*;
    match formula {
        Boolean(true) => Id::new(T),
        Boolean(false) => Id::new(F),
        Infix(_op, _left, _right) => {
            error!("infix operators not yet supported");
            input_error()
        }
        Predicate(name, children) => {
            if children.is_empty() {
                let symbol = loading.symbol(name.as_ref());
                Id::new(Prd(symbol))
            } else {
                error!("non-empty predicates not yet supported");
                input_error()
            }
        }
        Unary(UnaryConnective::Not, f) => Id::new(Not(load_fof_formula(*f, loading))),
        NonAssoc(connective, left, right) => {
            use self::NonAssocConnective::*;
            let left = load_fof_formula(*left, loading);
            let right = load_fof_formula(*right, loading);
            Id::new(match connective {
                LRImplies => Imp(left, right),
                RLImplies => Imp(right, left),
                Equivalent => Eqv(left, right),
                NotEquivalent => Not(Id::new(Eqv(left, right))),
                NotOr => Not(Id::new(Or(Id::new(set![left, right])))),
                NotAnd => Not(Id::new(And(Id::new(set![left, right])))),
            })
        }
        Assoc(connective, children) => {
            let children = Id::new(
                children
                    .into_iter()
                    .map(|f| load_fof_formula(f, loading))
                    .collect(),
            );

            Id::new(match connective {
                AssocConnective::Or => Or(children),
                AssocConnective::And => And(children),
            })
        }
        Quantified(_quantifier, _bound, _formula) => {
            error!("quantified formulae not yet supported");
            input_error()
        }
    }
}

fn load_statement<'a>(statement: Statement<'a>, loading: &'a mut Loading) {
    use self::Statement::*;
    match statement {
        Include(path, None) => load_file(path.to_str().unwrap(), loading),
        Include(_path, _selections) => {
            error!("include() selections are not supported");
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

fn load_file(path: &str, loading: &mut Loading) {
    info!("loading from '{}'...", path);

    let mapped = open_file(path).unwrap_or_else(|e| {
        error!("OS error: {}", e);
        os_error()
    });

    let bytes = &*match mapped {
        Some(bytes) => bytes,
        None => return,
    };

    for result in parse(bytes) {
        OPTIONS.check_time();
        let statement = result.unwrap_or_else(|e| {
            let start = bytes.as_ptr() as usize;
            let position = e.position.as_ptr() as usize;
            let offset = position - start;
            error!("syntax error: in '{}', starting at byte {}", path, offset);
            input_error()
        });
        load_statement(statement, loading);
    }
}

pub fn load() -> Loaded {
    let start = &OPTIONS.file;
    let mut loading = Loading::default();
    load_file(start, &mut loading);
    let loaded = loading.finish();

    info!(
        "...load complete, read {} axiom(s) and 1 conjecture",
        loaded.axioms.len()
    );
    loaded
}
