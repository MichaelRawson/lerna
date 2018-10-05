use crossbeam::scope;
use num_cpus;
use time::{get_time, Duration, Timespec};

use core::{Goal, Proof};
use options::SearchOptions;
use simplifications::simplify;
use tree::Tree;

const STACK_SIZE: usize = 10 * 1024 * 1024;

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
        let parallelism = num_cpus::get();
        info!("running on {} logical core(s)", parallelism);

        scope(|scope| {
            let mut workers = vec![];
            for i in 0..parallelism {
                debug!("spawning worker {}", i);
                let builder = scope
                    .builder()
                    .name(format!("lerna-worker-{}", i))
                    .stack_size(STACK_SIZE);
                workers.push(builder.spawn(|| self.work()).expect("spawn failed"));
            }
            for (i, worker) in workers.into_iter().enumerate() {
                worker.join().unwrap();
                debug!("worker {} exited", i);
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
