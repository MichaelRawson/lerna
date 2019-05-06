#[macro_use]
mod collections;

mod deduction;
mod formula;
mod graph;
mod heuristic;
mod input;
mod logging;
mod options;
mod oracle;
mod output;
mod prover;
mod record;
mod score;
mod search;
mod simplification;
mod status;
mod symbol;
mod system;
mod term;

use unique::Id;

use crate::formula::Formula;
use crate::input::load;
use crate::options::{Mode, OPTIONS};
use crate::oracle::consult;
use crate::prover::Prover;
use crate::simplification::simplify;
use crate::status::Status;
use crate::system::{check_for_timeout, satisfiable, time_out, unsatisfiable};

fn run_baseline(simplified: Id<Formula>) {
    log::info!("running oracle...");

    use Status::*;
    match consult(&simplified) {
        Sat => {
            log::info!("...problem satisfiable");
            satisfiable()
        }
        Unsat => {
            log::info!("...problem unsatisfiable");
            unsatisfiable(vec![simplified.clone()])
        }
        Unknown => {
            log::info!("...time out");
            time_out()
        }
    }
}

fn run_prover(simplified: Id<Formula>) {
    log::info!("running prover...");
    let mut prover = Prover::new(simplified);

    use Status::*;
    match prover.run() {
        Sat => {
            log::info!("...problem satisfiable");
            satisfiable()
        }
        Unsat => {
            log::info!("...problem unsatisfiable");
            let lemmas = prover.search.proof();
            unsatisfiable(lemmas)
        }
        Unknown => {
            log::info!("...time out");
            time_out()
        }
    }
}

fn main() {
    options::initialize();
    logging::initialize();
    check_for_timeout();

    let loaded = load();
    let simplified = simplify(&loaded.goal);
    check_for_timeout();

    use Mode::*;
    match OPTIONS.mode {
        Baseline => run_baseline(simplified),
        Prover => run_prover(simplified),
    }
}
