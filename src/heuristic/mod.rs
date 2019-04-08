use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::score::Score;

pub mod null;

pub trait Heuristic: Sync + Send {
    fn score(&self, batch: &[Id<Formula>]) -> Vec<Score>;
}

pub fn heuristic(batch: &[Id<Formula>]) -> Vec<Score> {
    OPTIONS.heuristic.score(batch)
}
