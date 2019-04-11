use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::status::Status;

pub mod z3;

pub trait Oracle: Sync + Send {
    fn consult(&self, f: &Id<Formula>) -> Status;
}

pub fn consult(f: &Id<Formula>) -> Status {
    OPTIONS.oracle.consult(f)
}
