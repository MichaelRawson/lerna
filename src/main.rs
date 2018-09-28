extern crate atomic;
extern crate clap;
extern crate fern;
#[macro_use]
extern crate im;
#[macro_use]
extern crate log;
extern crate parking_lot;
extern crate tptp;
extern crate rand;

#[macro_use]
mod core;
mod inferences;
mod input;
mod logging;
mod options;
mod output;
mod search;
mod simplifications;
mod tree;
mod uct;
mod util;

use core::Core;
use options::Options;
use search::{Search, SearchResult};

fn main() {
    let options = Options::parse();
    logging::setup(&options.logging);
    let core = Core::new(&options.core);

    info!("loading from {:?}...", options.input.file);
    let goal = input::load(&options.input, &core);
    info!("loading complete");

    let search = Search::new(&options.search, goal);
    info!("initialisation complete, begin proving");

    match search.go(100) {
        SearchResult::Failed => {
            info!("failed");
        }
        SearchResult::Proof(proof) => {
            output::print(&options.output, &core, proof);
        }
    }
}
