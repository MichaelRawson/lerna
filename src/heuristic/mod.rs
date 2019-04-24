use std::str::FromStr;
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::score::Score;

mod null;

pub enum Heuristic {
    Null,
}

impl FromStr for Heuristic {
    type Err = ();

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        use Heuristic::*;
        match x {
            "null" => Ok(Null),
            _ => Err(()),
        }
    }
}

pub fn heuristic(batch: &[Id<Formula>]) -> Vec<Score> {
    use Heuristic::*;
    match &OPTIONS.heuristic {
        Null => null::score(batch),
    }
}
