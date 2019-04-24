use std::io;
use std::io::Write;
use std::process::{Command, Stdio};
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::output::smtlib2;
use crate::status::Status;

fn write_stdin<W: Write>(w: &mut W, f: &Id<Formula>) -> io::Result<()> {
    writeln!(w, "(set-option :smt.auto-config false)")?;
    writeln!(w, "(set-option :smt.ematching false)")?;
    writeln!(w, "(set-option :smt.mbqi true)")?;
    writeln!(
        w,
        "(set-option :smt.mbqi.max_iterations {})",
        OPTIONS.oracle_iterations
    )?;
    writeln!(w)?;
    smtlib2::write_problem(w, f)
}

pub fn run(f: &Id<Formula>) -> Status {
    let mut z3 = Command::new("z3")
        .arg("-in")
        .arg(format!("-t:{}", OPTIONS.oracle_timeout))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to launch `z3`");

    let stdin = z3.stdin.as_mut().expect("failed to get z3 stdin");
    write_stdin(stdin, f).expect("failed to write z3 stdin");
    let run = z3.wait_with_output().expect("z3 failed");

    if !run.status.success() {
        log::error!(
            "z3 crashed: {}",
            std::str::from_utf8(&run.stdout).unwrap()
        );
        panic!("z3 crashed, what do?");
    }
    assert!(run.status.success(), "z3 returned non-zero exit status");
    let stdout: &[u8] = &run.stdout;
    match stdout {
        b"sat\n" => Status::Sat,
        b"unsat\n" => Status::Unsat,
        b"unknown\n" => Status::Unknown,
        _ => panic!("z3 produced unknown output: {:?}", stdout),
    }
}
