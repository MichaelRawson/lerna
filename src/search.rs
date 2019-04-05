use crossbeam::channel::{Receiver, Sender};
use unique::Id;

use crate::formula::Formula;
use crate::heuristic::Score;
use crate::oracle::Consulted;
use crate::system::check_for_timeout;

struct Node {}

pub struct Search {
    oracle_in: Receiver<(Id<Formula>, Consulted)>,
    oracle_out: Sender<Id<Formula>>,
    heuristic_in: Receiver<(Id<Formula>, Score)>,
    heuristic_out: Sender<Id<Formula>>,
}

impl Search {
    pub fn new(
        _problem: Id<Formula>,
        oracle_in: Receiver<(Id<Formula>, Consulted)>,
        oracle_out: Sender<Id<Formula>>,
        heuristic_in: Receiver<(Id<Formula>, Score)>,
        heuristic_out: Sender<Id<Formula>>,
    ) -> Self {
        Search {
            oracle_in,
            oracle_out,
            heuristic_in,
            heuristic_out,
        }
    }

    pub fn task(&mut self) {
        loop {
            check_for_timeout(true);
        }
    }
}
