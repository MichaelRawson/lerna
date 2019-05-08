use lazy_static::lazy_static;
use std::ffi::OsStr;
use std::path::Path;
use std::process::exit;
use std::time::SystemTime;
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::output::tptp;

lazy_static! {
    static ref START_TIME: SystemTime = SystemTime::now();
}

fn logical_data_id() -> &'static str {
    Path::new(&OPTIONS.file)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("unknown")
}

pub fn os_error() -> ! {
    println!();
    println!("% SZS status OSError for {}", logical_data_id());
    exit(1);
}

pub fn input_error() -> ! {
    println!();
    println!("% SZS status InputError for {}", logical_data_id());
    exit(1)
}

pub fn time_out() -> ! {
    println!();
    println!("% SZS status TimeOut for {}", logical_data_id());
    exit(1)
}

pub fn satisfiable() -> ! {
    let id = logical_data_id();
    println!();
    println!("% SZS status Satisfiable for {}", id);
    println!("% SZS output start Assurance for {}", id);
    println!("% SZS output end Assurance for {}", id);
    exit(0)
}

pub fn unsatisfiable(lemmas: Vec<Id<Formula>>) -> ! {
    let id = logical_data_id();
    println!();
    println!("% SZS status Unsatisfiable for {}", id);
    println!("% SZS output start Refutation for {}", id);
    for lemma in &lemmas {
        tptp::write_statement(&mut std::io::stdout(), lemma)
            .expect("writing statement to stdout failed");
    }
    println!("% SZS output end Refutation for {}", id);
    exit(0)
}

pub fn check_for_timeout() {
    if !within_time() {
        time_out()
    }
}

pub fn within_time() -> bool {
    let elapsed = START_TIME.elapsed().unwrap_or_default();
    elapsed < OPTIONS.time
}

pub fn initialize() {
    lazy_static::initialize(&START_TIME);
}
