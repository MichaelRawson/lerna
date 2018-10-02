use std::sync::Arc;
use std::vec::Vec;

use crossbeam::scope;
use time::{get_time, Duration, Timespec};

use core::{Formula, Goal};
use options::SearchOptions;
use simplifications::simplify;
use tree::Tree;

pub enum SearchResult {
    TimeOut,
    ProofFound(Vec<Arc<Formula>>),
}

pub struct Search {
    timeout: Timespec,
    original: Goal,
    tree: Tree,
}

impl Search {
    pub fn new(options: &SearchOptions, start_time: Timespec, start: Goal) -> Self {
        let duration = Duration::seconds(options.timeout as i64);
        let timeout = start_time + duration;
        debug!("timeout is {:?}", timeout);

        debug!("simplifying start goal {:#?}...", start.formulae);
        let simplified = simplify(start.clone());
        debug!("...simplified.");

        let result = Search {
            timeout,
            original: start,
            tree: Tree::new(simplified),
        };
        debug!("search space initialized");
        result
    }

    pub fn work(&self) {
        while !self.tree.complete() && get_time() < self.timeout {
            self.tree.step();
        }
    }

    pub fn run(&self) -> SearchResult {
        scope(|scope| {
            let mut workers = vec![];
            for _ in 1..8 {
                workers.push(scope.spawn(|| self.work()));
            }
            for worker in workers {
                worker.join().unwrap();
            }
        });

        debug!("finished proof search");
        debug!("total steps: {}", self.tree.total_visits());

        if self.tree.complete() {
            debug!("proof found");
            let formulae = self.original.formulae.clone();
            let mut proof = formulae.clone().into_iter().collect();
            self.tree.proof(formulae, &mut proof);
            SearchResult::ProofFound(proof)
        } else {
            debug!("proof failed");
            SearchResult::TimeOut
        }
    }
}
