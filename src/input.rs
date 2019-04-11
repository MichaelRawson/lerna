use memmap::Mmap;
use std::io;
use std::vec::Vec;
use tptp::syntax::*;
use tptp::{parse, resolve_include};
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::symbol::Symbol;
use crate::system::{check_for_timeout, input_error, os_error};
use crate::term::Term;

#[derive(Default)]
struct Loading {
    axioms: Vec<Id<Formula>>,
}

pub struct Loaded {
    pub goal: Id<Formula>,
}

impl Loading {
    fn add_formula(&mut self, role: FormulaRole, mut formula: Id<Formula>) {
        use self::FormulaRole::*;

        formula = match role {
            Conjecture => Formula::negate(&formula),
            _ => formula,
        };
        self.axioms.push(formula);
    }

    fn finish(self) -> Loaded {
        let goal = Id::new(Formula::And(self.axioms.into_iter().collect()));
        Loaded { goal }
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

fn load_symbol(symbol: &str) -> Id<Symbol> {
    Id::new(Symbol::Original(symbol.into()))
}

fn load_term(term: FofTerm, bound: &mut Vec<Id<Symbol>>) -> Id<Term> {
    use self::FofTerm::*;
    use self::Term::*;
    match term {
        Variable(x) => {
            let symbol = load_symbol(x.as_ref());
            let index = bound.len()
                - (bound.iter().rposition(|x| x == &symbol).unwrap_or_else(
                    || {
                        log::error!("unbound variable: {}", x);
                        input_error()
                    },
                ) + 1);
            Id::new(Var(index))
        }
        Functor(f, ts) => {
            let f = load_symbol(f.as_ref());
            let ts = ts.into_iter().map(|t| load_term(t, bound)).collect();
            Id::new(Fn(f, ts))
        }
    }
}

fn load_fof_formula(
    formula: FofFormula,
    bound: &mut Vec<Id<Symbol>>,
) -> Id<Formula> {
    use self::FofFormula::*;
    use self::Formula::*;
    match formula {
        Boolean(true) => Id::new(T),
        Boolean(false) => Id::new(F),
        Infix(op, left, right) => {
            use self::InfixEquality::*;
            let left = load_term(left, bound);
            let right = load_term(right, bound);
            match op {
                Equal => Id::new(Eq(idset![left, right])),
                NotEqual => Formula::negate(&Id::new(Eq(idset![left, right]))),
            }
        }
        Predicate(name, children) => {
            let name = load_symbol(name.as_ref());
            let children =
                children.into_iter().map(|t| load_term(t, bound)).collect();
            Id::new(Prd(name, children))
        }
        Unary(UnaryConnective::Not, f) => {
            Formula::negate(&load_fof_formula(*f, bound))
        }
        NonAssoc(connective, left, right) => {
            use self::NonAssocConnective::*;
            let left = load_fof_formula(*left, bound);
            let right = load_fof_formula(*right, bound);
            match connective {
                LRImplies => Id::new(Imp(left, right)),
                RLImplies => Id::new(Imp(right, left)),
                Equivalent => Id::new(Eqv(idset![left, right])),
                NotEquivalent => {
                    Formula::negate(&Id::new(Eqv(idset![left, right])))
                }
                NotOr => Formula::negate(&Id::new(Or(idset![left, right]))),
                NotAnd => Formula::negate(&Id::new(And(idset![left, right]))),
            }
        }
        Assoc(connective, children) => {
            let children = children
                .into_iter()
                .map(|f| load_fof_formula(f, bound))
                .collect();

            Id::new(match connective {
                AssocConnective::Or => Or(children),
                AssocConnective::And => And(children),
            })
        }
        Quantified(quantifier, vars, f) => {
            use self::FofQuantifier::*;
            let original_size = bound.len();
            bound.extend(vars.iter().map(|x| load_symbol(x.as_ref())));
            let mut f = load_fof_formula(*f, bound);
            bound.resize_with(original_size, || unreachable!());

            for _ in 0..vars.len() {
                f = match quantifier {
                    Forall => Id::new(All(f)),
                    Exists => Id::new(Ex(f)),
                }
            }

            f
        }
    }
}

fn load_statement<'a>(statement: Statement<'a>, loading: &'a mut Loading) {
    use self::Statement::*;
    match statement {
        Include(included, None) => load_file(&included, loading),
        Include(_path, _selections) => {
            log::error!(
                "include() statements with selections are not supported"
            );
            input_error()
        }
        Cnf(_name, _role, _formula, _annotations) => {
            log::error!("CNF statements are not supported");
            input_error()
        }
        Fof(_name, role, formula, _annotations) => {
            let formula = load_fof_formula(formula, &mut vec![]);
            loading.add_formula(role, formula);
        }
    }
}

fn load_file(path: &Included, loading: &mut Loading) {
    log::info!("loading from '{}'...", path);

    let mapped = open_file(&path).unwrap_or_else(|e| {
        log::error!("OS error: {}", e);
        os_error()
    });
    check_for_timeout(true);

    let bytes = &*match mapped {
        Some(bytes) => bytes,
        None => return,
    };

    for result in parse(bytes) {
        check_for_timeout(true);
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
    let num_axioms = loading.axioms.len();
    let loaded = loading.finish();

    log::info!("...load complete, read {} axiom(s)", num_axioms);
    loaded
}
