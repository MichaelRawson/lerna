use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use crossbeam::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use unique::Id;

use crate::formula::Formula;
use crate::heuristic::heuristic;
use crate::oracle::consult;
use crate::score::Score;
use crate::search::Search;
use crate::status::Status;
use crate::system::{check_for_timeout, os_error};

const BATCH_SIZE: usize = 128;
const MAX_QUEUED: usize = 65536;

pub struct Prover {
    problem: Id<Formula>,
    search: Search,
}

impl Prover {
    pub fn new(problem: Id<Formula>) -> Self {
        let search = Search::new(problem.clone());
        Self { problem, search }
    }

    pub fn run(&mut self) -> Status {
        let (search2heuristic_send, search2heuristic_receive) =
            bounded(MAX_QUEUED);
        let (search2oracle_send, search2oracle_receive) = bounded(MAX_QUEUED);
        let (heuristic2search_send, heuristic2search_receive) = unbounded();
        let (oracle2search_send, oracle2search_receive) = unbounded();

        search2heuristic_send.send(self.problem.clone()).unwrap();
        search2oracle_send.send(self.problem.clone()).unwrap();

        let running = AtomicBool::new(true);
        thread::scope(|s| {
            s.spawn(|_| {
                consult_task(&running, search2oracle_receive, oracle2search_send)
            });
            s.spawn(|_| {
                heuristic_task(&running, search2heuristic_receive, heuristic2search_send)
            });

            let result = search_task(
                &mut self.search,
                search2heuristic_send,
                search2oracle_send,
                heuristic2search_receive,
                oracle2search_receive,
            );
            running.store(false, Ordering::Relaxed);
            result
        })
        .unwrap_or_else(|e| {
            log::error!("failed to run worker threads: {:?}", e);
            os_error()
        })
    }
}

pub fn consult_task(
    running: &AtomicBool,
    oracle_in: Receiver<Id<Formula>>,
    oracle_out: Sender<(Id<Formula>, Status)>,
) {
    while running.load(Ordering::Relaxed) {
        check_for_timeout(false);
        if let Ok(f) = oracle_in.try_recv() {
            let consultation = consult(&f);
            if oracle_out.send((f, consultation)).is_err() {
                log::debug!("outwards channel disconnected, exiting...");
                break;
            }
        }
    }
}

pub fn heuristic_task(
    running: &AtomicBool,
    heuristic_in: Receiver<Id<Formula>>,
    heuristic_out: Sender<(Id<Formula>, Score)>,
) {
    while running.load(Ordering::Relaxed) {
        check_for_timeout(true);

        let mut batch = vec![];
        while let Ok(f) = heuristic_in.try_recv() {
            batch.push(f);
            if batch.len() > BATCH_SIZE {
                break;
            }
        }

        let scores = heuristic(&batch);
        assert!(scores.len() == batch.len());

        for scored in batch.into_iter().zip(scores.into_iter()) {
            if heuristic_out.send(scored).is_err() {
                log::debug!("outwards channel disconnected, exiting...");
                return;
            }
        }
    }
}

pub fn search_task(
    search: &mut Search,
    _heuristic_send: Sender<Id<Formula>>,
    _oracle_send: Sender<Id<Formula>>,
    heuristic_recv: Receiver<(Id<Formula>, Score)>,
    oracle_recv: Receiver<(Id<Formula>, Status)>,
) -> Status {
    while search.status() == Status::Unknown {
        check_for_timeout(true);

        if let Ok((f, status)) = oracle_recv.try_recv() {
            search.set_status(f, status);
        }
        else if let Ok((f, score)) = heuristic_recv.try_recv() {
            search.set_score(f, score);
        }
    }

    search.status()
}
