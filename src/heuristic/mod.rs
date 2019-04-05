use crossbeam::channel::{Receiver, Sender};
use std::cmp::Ordering;
use std::fmt;
use std::thread::sleep;
use std::time::Duration;
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::system::check_for_timeout;

pub mod null;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Score(pub f32);

impl Score {
    pub fn new(score: f32) -> Self {
        assert!(score.is_finite());
        Score(score)
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.partial_cmp(&other.0).unwrap())
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Debug for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub trait Heuristic: Sync + Send {
    fn score(&self, batch: &[Id<Formula>]) -> Vec<Score>;
}

pub fn heuristic(batch: &[Id<Formula>]) -> Vec<Score> {
    OPTIONS.heuristic.score(batch)
}

pub fn heuristic_task(
    heuristic_in: Receiver<Id<Formula>>,
    heuristic_out: Sender<(Id<Formula>, Score)>,
) {
    loop {
        check_for_timeout(true);

        let mut batch = vec![];
        while let Ok(f) = heuristic_in.try_recv() {
            batch.push(f);
        }

        let scores = heuristic(&batch);
        assert!(scores.len() == batch.len());

        for scored in batch.into_iter().zip(scores.into_iter()) {
            if heuristic_out.send(scored).is_err() {
                log::debug!("outwards channel disconnected, exiting...");
                break;
            }
        }

        if heuristic_in.is_empty() {
            sleep(Duration::from_millis(10));
        }
    }
}
