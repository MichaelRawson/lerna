use std::io;
use std::io::Write;
use std::iter;
use unique::Id;

use crate::collections::IdList;
use crate::formula::Formula;
use crate::symbol::Symbol;
use crate::term::Term;

fn symbol_name(s: &Symbol, arity: usize) -> String {
    format!("{:?}_{}", s, arity)
}

fn write_signature<W: Write>(w: &mut W, f: &Id<Formula>) -> io::Result<()> {
    writeln!(w, "(declare-sort object)")?;
    for (symbol, arity) in Formula::predicate_symbols(f) {
        let name = symbol_name(&symbol, arity);
        let args = iter::repeat("object")
            .take(arity)
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(w, "(declare-fun {} ({}) Bool)", name, args)?;
    }

    for (symbol, arity) in Formula::function_symbols(f) {
        let name = symbol_name(&symbol, arity);
        let args = iter::repeat("object")
            .take(arity)
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(w, "(declare-fun {} ({}) object)", name, args)?;
    }

    Ok(())
}

fn write_term_list<W: Write>(
    w: &mut W,
    fs: &IdList<Term>,
    bound: usize,
) -> io::Result<()> {
    for f in fs {
        write!(w, " ")?;
        write_term(w, f, bound)?;
    }
    Ok(())
}

fn write_term<W: Write>(
    w: &mut W,
    t: &Id<Term>,
    bound: usize,
) -> io::Result<()> {
    use Term::*;
    match **t {
        Var(n) => write!(w, "X{}", (bound - 1) - n),
        Fn(ref f, ref ts) => {
            if ts.is_empty() {
                write!(w, "{}", symbol_name(&f, 0))
            } else {
                write!(w, "({}", symbol_name(&f, ts.len()))?;
                write_term_list(w, ts, bound)?;
                write!(w, ")")
            }
        }
    }
}

fn write_formula_list<W: Write>(
    w: &mut W,
    fs: &IdList<Formula>,
    bound: usize,
) -> io::Result<()> {
    for f in fs {
        write!(w, " ")?;
        write_formula(w, f, bound)?;
    }
    Ok(())
}

fn write_formula<W: Write>(
    w: &mut W,
    f: &Id<Formula>,
    bound: usize,
) -> io::Result<()> {
    use Formula::*;
    match **f {
        T => write!(w, "true"),
        F => write!(w, "false"),
        Eq(ref ts) => {
            if ts.len() == 1 {
                write!(w, "true")
            } else if ts.len() > 1 {
                write!(w, "(=")?;
                write_term_list(w, ts.as_ref(), bound)?;
                write!(w, ")")
            } else {
                unreachable!()
            }
        }
        Prd(ref p, ref ts) => {
            if ts.is_empty() {
                write!(w, "{}", symbol_name(p, 0))
            } else {
                write!(w, "({}", symbol_name(p, ts.len()))?;
                write_term_list(w, ts, bound)?;
                write!(w, ")")
            }
        }
        Not(ref p) => {
            write!(w, "(not ")?;
            write_formula(w, p, bound)?;
            write!(w, ")")
        }
        Imp(ref p, ref q) => {
            write!(w, "(=> ")?;
            write_formula(w, p, bound)?;
            write!(w, " ")?;
            write_formula(w, q, bound)?;
            write!(w, ")")
        }
        And(ref ps) => {
            write!(w, "(and")?;
            write_formula_list(w, ps.as_ref(), bound)?;
            write!(w, ")")
        }
        Or(ref ps) => {
            write!(w, "(or")?;
            write_formula_list(w, ps.as_ref(), bound)?;
            write!(w, ")")
        }
        Eqv(ref ps) => {
            if ps.len() == 1 {
                write!(w, "true")
            } else if ps.len() > 1 {
                write!(w, "(=")?;
                write_formula_list(w, ps.as_ref(), bound)?;
                write!(w, ")")
            } else {
                unreachable!()
            }
        }
        All(ref p) => {
            write!(w, "(forall ((X{} object)) ", bound)?;
            write_formula(w, p, bound + 1)?;
            write!(w, ")")
        }
        Ex(ref p) => {
            write!(w, "(exists ((X{} object)) ", bound)?;
            write_formula(w, p, bound + 1)?;
            write!(w, ")")
        }
    }
}

pub fn write_problem<W: Write>(w: &mut W, f: &Id<Formula>) -> io::Result<()> {
    write_signature(w, f)?;
    write!(w, "(assert ")?;
    write_formula(w, f, 0)?;
    writeln!(w, ")")?;
    writeln!(w, "(check-sat)")
}
