use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::process::exit;
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::output::tptp;

fn logical_data_id() -> &'static str {
    Path::new(&OPTIONS.file)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("unknown")
}

pub fn os_error<T>() -> T {
    println!();
    println!("% SZS status OSError for {}", logical_data_id());
    exit(1);
}

pub fn input_error<T>() -> T {
    println!();
    println!("% SZS status InputError for {}", logical_data_id());
    exit(1);
}

pub fn time_out<T>(print: bool) -> T {
    if print {
        println!();
        println!("% SZS status TimeOut for {}", logical_data_id());
    }
    exit(1);
}

pub fn satisfiable<T>() -> T {
    println!();
    println!("% SZS status CounterSatisfiable for {}", logical_data_id());
    println!("% SZS output start Assurance for {}", logical_data_id());
    println!("% SZS output end Assurance for {}", logical_data_id());
    exit(0);
}

pub fn unsatisfiable<T>(lemmas: HashSet<Id<Formula>>) -> T {
    println!();
    println!("% SZS status Theorem for {}", logical_data_id());
    println!("% SZS output start Solution for {}", logical_data_id());
    for lemma in &lemmas {
        tptp::write_lemma(&mut std::io::stdout(), lemma)
            .expect("writing lemma to stdout failed");
    }
    println!("% SZS output end Solution for {}", logical_data_id());
    exit(0);
}

pub fn gave_up<T>() -> T {
    println!();
    println!("% SZS status GaveUp for {}", logical_data_id());
    exit(1);
}

pub fn check_for_timeout(print: bool) {
    if !within_time() {
        time_out(print)
    }
}

pub fn within_time() -> bool {
    let elapsed = OPTIONS.start_time.elapsed().unwrap_or_default();
    elapsed < OPTIONS.time
}
