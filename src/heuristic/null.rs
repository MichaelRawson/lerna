use unique::Id;

use crate::formula::Formula;
use crate::heuristic::Heuristic;
use crate::score::Score;

pub struct Null;

impl Heuristic for Null {
    fn score(&self, batch: &[Id<Formula>]) -> Vec<Score> {
        let score = 0.5.into();
        batch.iter().map(|_| score).collect()
    }
}
