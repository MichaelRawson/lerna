extern crate atomic;
extern crate clap;
extern crate crossbeam;
extern crate fern;
#[macro_use]
extern crate im;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate parking_lot;
extern crate rand;
extern crate time;
extern crate tptp;

#[macro_use]
mod core;
mod common;
mod inferences;
mod input;
mod names;
mod options;
mod output;
mod search;
mod simplifications;
mod tree;
mod uct;
mod util;

use std::process::exit;

use time::get_time;

use input::LoadError;
use options::Options;
use search::{Search, SearchResult};

fn main() {
    let start_time = get_time();

    let options = Options::parse();
    output::setup_logging(&options.output);

    debug!("program started, start time was {:?}", start_time);
    info!("OK, running for {}s", &options.search.timeout);
    info!("loading from {:?}...", options.input.file);
    let goal = input::load(&options.input).unwrap_or_else(|err| {
        match err {
            LoadError::OSError => output::os_error(&options.output),
            LoadError::InputError => output::input_error(&options.output),
            LoadError::Unsupported => output::unsupported(&options.output),
        };
        debug!("load error, exit(1)");
        exit(1);
    });
    info!("loading complete");

    let search = Search::new(&options.search, start_time, goal);
    info!("begin proving...");

    match search.run() {
        SearchResult::TimeOut => {
            info!("...timed out");
            debug!("time out, reporting...");
            output::time_out(&options.output);
            debug!("...proof failed, exit(1)");
            exit(1);
        }
        SearchResult::ProofFound(original, proof) => {
            info!("...proof found");
            debug!("proof found, reporting...");
            let done = original.consume();
            output::proof_found(&options.output, done, &proof);
            info!("bye!");
            debug!("...proof succeeded, exit(0)");
            exit(0);
        }
    }
}
