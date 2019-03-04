use crossbeam::channel::{bounded, unbounded};
use crossbeam::scope;
use std::sync::atomic::{AtomicBool, Ordering};
use unique::Id;

use crate::collections::Set;
use crate::formula::Formula;
use crate::heuristic::Heuristic;
use crate::options::OPTIONS;
use crate::search::Search;
use crate::system;

/*
pub struct Prover {
    search: Search,
    heuristic: Heuristic,
}

pub struct Result {
    pub success: bool,
}

impl Prover {
    pub fn new(axioms: Set<Formula>, negated_conjecture: Id<Formula>) -> Self {
        let (goal_send, goal_recv) = bounded(OPTIONS.goal_queue_size);
        let (score_send, score_recv) = unbounded();

        let search =
            Search::new(axioms, negated_conjecture, goal_send, score_recv);
        let heuristic = Heuristic::new(goal_recv, score_send);

        Self { search, heuristic }
    }

    pub fn run(self) -> Result {
        let search = self.search;
        let heuristic = self.heuristic;

        log::info!("searching...");
        scope(|threads| {
            threads
                .builder()
                .name("search".into())
                .spawn(|_| search_task(&mut search, &done))
                .unwrap();
            threads
                .builder()
                .name("heuristic".into())
                .spawn(|_| heuristic_task(&mut heuristic, &done))
                .unwrap();
        })
        .unwrap();

        let success = search.is_refuted();
        if success {
            log::info!("...proof found.");
        } else {
            log::info!("..timed out.");
        }

        Result { success }
    }
}

fn should_continue(done: &AtomicBool) -> bool {
    system::within_time() && !done.load(Ordering::Relaxed)
}

fn search_task(search: &mut Search, done: &AtomicBool) {
    while should_continue(done) {
        search.work();
        done.fetch_or(search.is_refuted(), Ordering::Relaxed);
    }
}

fn heuristic_task(heuristic: &mut Heuristic, done: &AtomicBool) {
    while should_continue(done) {
        heuristic.work();
    }
    heuristic.cleanup();
}
*/
