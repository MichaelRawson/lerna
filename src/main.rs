#[macro_use]
mod collections;

mod deduction;
mod formula;
mod goal;
//mod heuristic;
mod inference;
mod input;
mod justification;
mod logging;
mod options;
//mod prover;
mod score;
mod search;
mod simplification;
mod symbol;
mod system;

//use crate::prover::Prover;

fn main() {
    options::initialize();
    logging::initialize();
    system::check_for_timeout();

    let loaded = input::load();
    system::check_for_timeout();

    /*
    let result = Prover::new(loaded.axioms, loaded.negated_conjecture).run();
    
    if result.success {
        println!("% SZS Status THM");
    } else {
        system::time_out()
    };
    */
}
