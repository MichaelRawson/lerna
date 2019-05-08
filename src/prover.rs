use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use crossbeam::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::yield_now;
use unique::Id;

use crate::formula::Formula;
use crate::heuristic::{receive_from_heuristic, send_to_heuristic};
use crate::options::OPTIONS;
use crate::oracle::consult;
use crate::score::Score;
use crate::search::Search;
use crate::status::Status;
use crate::system::{os_error, within_time};

const MAX_QUEUED: usize = 128;

pub struct Prover {
    pub problem: Id<Formula>,
    pub search: Search,
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
            for _ in 0..OPTIONS.oracle_threads {
                s.spawn(|_| {
                    oracle_task(
                        &running,
                        search2oracle_receive.clone(),
                        oracle2search_send.clone(),
                    )
                });
            }

            if !OPTIONS.heuristic_off {
                s.spawn(|_| {
                    heuristic_in_task(&running, search2heuristic_receive)
                });
                s.spawn(|_| {
                    heuristic_out_task(&running, heuristic2search_send)
                });
            }

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

fn oracle_task(
    running: &AtomicBool,
    oracle_in: Receiver<Id<Formula>>,
    oracle_out: Sender<(Id<Formula>, Status)>,
) {
    while running.load(Ordering::Relaxed) {
        if let Ok(f) = oracle_in.try_recv() {
            let consultation = consult(&f);
            if oracle_out.send((f, consultation)).is_err() {
                return;
            }
        } else {
        }
    }
}

fn heuristic_in_task(
    running: &AtomicBool,
    heuristic_in: Receiver<Id<Formula>>,
) {
    while running.load(Ordering::Relaxed) {
        if let Ok(f) = heuristic_in.try_recv() {
            while !send_to_heuristic(&f) {
                yield_now()
            }
        } else {
            yield_now()
        }
    }
}

fn heuristic_out_task(
    running: &AtomicBool,
    heuristic_out: Sender<(Id<Formula>, Score)>,
) {
    while running.load(Ordering::Relaxed) {
        if let Some(scored) = receive_from_heuristic() {
            if heuristic_out.send(scored).is_err() {
                return;
            }
        } else {
            yield_now();
        }
    }
}

pub fn search_task(
    search: &mut Search,
    heuristic_send: Sender<Id<Formula>>,
    oracle_send: Sender<Id<Formula>>,
    heuristic_recv: Receiver<(Id<Formula>, Score)>,
    oracle_recv: Receiver<(Id<Formula>, Status)>,
) -> Status {
    while !search.status().is_known() && within_time() {
        if let Ok((f, status)) = oracle_recv.try_recv() {
            search.set_status(&f, status);
        } else if let Ok((f, score)) = heuristic_recv.try_recv() {
            search.set_score(&f, score);
        } else {
            let new_formulae = search.do_step();
            for f in new_formulae {
                oracle_send.send(f.clone()).unwrap();
                if !OPTIONS.heuristic_off {
                    heuristic_send.send(f).unwrap();
                }
            }
        }
    }
    log::info!("done");

    search.status()
}
