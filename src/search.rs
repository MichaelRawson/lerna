use crossbeam::scope;
use time::{get_time, Duration, Timespec};

use core::{Goal, Proof};
use options::SearchOptions;
use simplifications::simplify;
use tree::Tree;

pub enum SearchResult {
    TimeOut,
    ProofFound(Goal, Box<Proof>),
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

        debug!("simplifying start goal...");
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

    pub fn run(self) -> SearchResult {
        scope(|scope| {
            let mut workers = vec![];
            for index in 1..8 {
                debug!("spawning worker {}", index);
                workers.push(scope.spawn(|| self.work()));
            }
            for worker in workers {
                debug!("waiting for worker {:?}", worker);
                worker.join().unwrap();
            }
        });

        debug!("search finished");
        debug!("total steps: {}", self.tree.total_visits());

        if self.tree.complete() {
            debug!("proof found");
            SearchResult::ProofFound(self.original, self.tree.proof())
        } else {
            debug!("proof failed");
            SearchResult::TimeOut
        }
    }
}
