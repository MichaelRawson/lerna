use std::str::FromStr;
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::status::Status;

mod null;
mod z3;

pub enum Oracle {
    Null,
    Z3,
}

impl FromStr for Oracle {
    type Err = ();

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        use Oracle::*;
        match x {
            "null" => Ok(Null),
            "z3" => Ok(Z3),
            _ => Err(()),
        }
    }
}

pub fn consult(f: &Id<Formula>) -> Status {
    use Oracle::*;
    match OPTIONS.oracle {
        Null => null::run(f),
        Z3 => z3::run(f),
    }
}
