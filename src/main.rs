#[macro_use]
mod collections;

mod deduction;
mod formula;
mod heuristic;
mod input;
mod logging;
mod options;
mod oracle;
mod prover;
mod score;
mod search;
mod simplification;
mod status;
mod symbol;
mod system;
mod term;

use crate::input::load;
use crate::prover::Prover;
use crate::simplification::simplify;
use crate::status::Status;
use crate::system::check_for_timeout;

fn main() {
    options::initialize();
    logging::initialize();
    check_for_timeout(true);

    let loaded = load();
    let simplified = simplify(&loaded.goal);
    check_for_timeout(true);

    let mut prover = Prover::new(simplified);
    match prover.run() {
        Status::Sat => {
            println!("% SZS Status CounterSatisfiable");
        }
        Status::Unsat => {
            println!("% SZS Status Theorem");
        }
        _ => unreachable!(),
    }
}
