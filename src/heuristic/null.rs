use unique::Id;

use crate::formula::Formula;
use crate::score::Score;

pub fn score(batch: &[Id<Formula>]) -> Vec<Score> {
    let score = 0.5.into();
    batch.iter().map(|_| score).collect()
}
