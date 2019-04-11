use std::io;
use std::io::Write;
use std::iter;
use std::process::{Command, Stdio};
use unique::Id;

use crate::collections::IdList;
use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::oracle::Oracle;
use crate::status::Status;
use crate::symbol::Symbol;
use crate::term::Term;

pub struct Z3;

fn symbol_name(s: &Symbol) -> String {
    format!("{:?}", s)
}

fn write_header<W: Write>(w: &mut W, f: &Id<Formula>) -> io::Result<()> {
    writeln!(w, "(set-option :smt.auto-config false)")?;
    writeln!(w, "(set-option :smt.ematching false)")?;
    writeln!(w, "(set-option :smt.mbqi true)")?;
    writeln!(w, "(set-option :smt.mbqi.max_iterations 0)")?;
    writeln!(w, "(declare-sort object)")?;

    for (symbol, arity) in Formula::predicate_symbols(f) {
        let name = symbol_name(&symbol);
        let args = iter::repeat("object")
            .take(arity)
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(w, "(declare-fun {} ({}) Bool)", name, args)?;
    }

    for (symbol, arity) in Formula::function_symbols(f) {
        let name = symbol_name(&symbol);
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
                write!(w, "{}", symbol_name(&f))
            } else {
                write!(w, "({}", symbol_name(&f))?;
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
                write!(w, "{}", symbol_name(p))
            } else {
                write!(w, "({}", symbol_name(p))?;
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

fn write_problem<W: Write>(w: &mut W, f: &Id<Formula>) -> io::Result<()> {
    write!(w, "(assert ")?;
    write_formula(w, f, 0)?;
    writeln!(w, ")")
}

fn write_stdin<W: Write>(w: &mut W, f: &Id<Formula>) -> io::Result<()> {
    write_header(w, f)?;
    write_problem(w, f)?;
    writeln!(w, "(check-sat)")
}

fn run(f: &Id<Formula>) -> Status {
    let mut z3 = Command::new("z3")
        .arg("-in")
        .arg(format!("-t:{}", OPTIONS.oracle_time))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to launch `z3`");

    let stdin = z3.stdin.as_mut().expect("failed to get z3 stdin");
    write_stdin(stdin, f).expect("failed to write z3 stdin");
    let run = z3.wait_with_output().expect("z3 failed");

    if !run.status.success() {
        log::error!("z3 failed: {}", std::str::from_utf8(&run.stdout).unwrap());
        panic!("z3 failed, what do?");
    }
    assert!(run.status.success(), "z3 returned non-zero exit status");
    let stdout: &[u8] = &run.stdout;
    return Status::Unknown;
    match stdout {
        b"sat\n" => Status::Sat,
        b"unsat\n" => Status::Unsat,
        b"unknown\n" => Status::Unknown,
        _ => panic!("z3 produced unknown output: {:?}", stdout),
    }
}

impl Oracle for Z3 {
    fn consult(&self, f: &Id<Formula>) -> Status {
        run(f)
    }
}
