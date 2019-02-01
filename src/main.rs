#[macro_use]
mod set;

mod deduction;
mod formula;
mod goal;
mod heuristic;
mod inference;
mod input;
mod logging;
mod options;
mod prover;
mod score;
mod search;
mod simplification;
mod symbol;
mod system;

fn main() {
    logging::initialise();
    options::parse();
    options::OPTIONS.check_time();

    let loaded = input::load();
    let prover = prover::Prover::new(loaded.axioms, loaded.negated_conjecture);
    if prover.run() {
        println!("% SZS Status THM");
    }
    else {
        system::give_up()
    };
}
