use unique::Id;

use crate::formula::Formula;
use crate::status::Status;

pub fn run(f: &Id<Formula>) -> Status {
    if **f == Formula::F {
        Status::Unsat
    } else {
        Status::Unknown
    }
}
