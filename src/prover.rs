use crossbeam::channel::{bounded, unbounded};
use crossbeam::thread;
use unique::Id;

use crate::formula::Formula;
use crate::heuristic::heuristic_task;
use crate::oracle::consult_task;
use crate::search::Search;
use crate::system::os_error;

const MAX_QUEUE: usize = 128;

pub struct Prover {
    search: Search,
}

impl Prover {
    pub fn new(problem: Id<Formula>) -> Self {
        let (search2heuristic_send, search2heuristic_receive) =
            bounded(MAX_QUEUE);
        let (search2oracle_send, search2oracle_receive) = bounded(MAX_QUEUE);
        let (heuristic2search_send, heuristic2search_receive) = unbounded();
        let (oracle2search_send, oracle2search_receive) = unbounded();

        let search = Search::new(
            problem,
            oracle2search_receive,
            search2oracle_send,
            heuristic2search_receive,
            search2heuristic_send,
        );

        thread::scope(|s| {
            s.spawn(move |_| {
                log::debug!("consulting...");
                consult_task(search2oracle_receive, oracle2search_send)
            });
            s.spawn(move |_| {
                log::debug!("scoring...");
                heuristic_task(search2heuristic_receive, heuristic2search_send)
            });
        }).unwrap_or_else(|e| {
            log::error!("failed to spawn worker threads: {:?}", e);
            os_error()
        });

        Self { search }
    }

    pub fn run(&mut self) {
        self.search.task();
    }
}
