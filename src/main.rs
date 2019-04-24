#[macro_use]
mod collections;

mod deduction;
mod formula;
mod heuristic;
mod input;
mod logging;
mod options;
mod oracle;
mod output;
mod prover;
mod score;
mod search;
mod simplification;
mod status;
mod symbol;
mod system;
mod term;

use std::collections::HashSet;
use unique::Id;

use crate::formula::Formula;
use crate::input::load;
use crate::options::{Mode, OPTIONS};
use crate::oracle::consult;
use crate::prover::Prover;
use crate::simplification::simplify;
use crate::status::Status;
use crate::system::{check_for_timeout, gave_up, satisfiable, unsatisfiable};

fn run_baseline(simplified: Id<Formula>) {
    log::info!("running oracle in baseline mode...");

    use Status::*;
    match consult(&simplified) {
        Sat => {
            log::info!("...oracle found problem satisfiable");
            satisfiable()
        }
        Unsat => {
            log::info!("...oracle found problem unsatisfiable");
            let mut lemmas = HashSet::new();
            lemmas.insert(simplified.clone());
            unsatisfiable(lemmas)
        }
        Unknown => {
            log::info!("...oracle gave up");
            gave_up()
        }
    }
}

fn run_prover(simplified: Id<Formula>) {
    log::info!("running prover...");
    let mut prover = Prover::new(simplified);

    use Status::*;
    match prover.run() {
        Sat => {
            log::info!("...prover found problem satisfiable");
            satisfiable()
        }
        Unsat => {
            log::info!(
                "...prover found problem unsatisfiable, printing lemmas"
            );
            let lemmas = prover.search.proof();
            unsatisfiable(lemmas)
        }
        _ => unreachable!(),
    }
}

fn main() {
    options::initialize();
    logging::initialize();
    check_for_timeout(true);

    let loaded = load();
    let simplified = simplify(&loaded.goal);
    check_for_timeout(true);

    use Mode::*;
    match OPTIONS.mode {
        Baseline => run_baseline(simplified),
        Prover => run_prover(simplified),
    }
}
