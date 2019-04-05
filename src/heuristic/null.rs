use unique::Id;

use crate::formula::Formula;
use crate::heuristic::{Heuristic, Score};

pub struct Null;

impl Heuristic for Null {
    fn score(&self, batch: &[Id<Formula>]) -> Vec<Score> {
        batch.iter().map(|_| Score::new(0.5)).collect()
    }
}
