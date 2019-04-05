use crossbeam::channel::{Receiver, Sender};
use std::ops::{Add, Mul};
use std::time::Duration;
use unique::Id;

use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::system::check_for_timeout;

pub mod z3;

#[derive(Debug)]
pub enum Consulted {
    Sat,
    Unsat,
    Unknown,
}

use Consulted::*;

impl Add for Consulted {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Sat | Unknown => other,
            Unsat => Unsat,
        }
    }
}

impl Mul for Consulted {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match self {
            Sat => Sat,
            Unsat | Unknown => other,
        }
    }
}

pub trait Oracle: Sync + Send {
    fn consult(&self, f: &Id<Formula>) -> Consulted;
}

pub fn consult(f: &Id<Formula>) -> Consulted {
    use Formula::*;
    match **f {
        T => Sat,
        F => Unsat,
        _ => OPTIONS.oracle.consult(f),
    }
}

pub fn consult_task(
    oracle_in: Receiver<Id<Formula>>,
    oracle_out: Sender<(Id<Formula>, Consulted)>,
) {
    loop {
        check_for_timeout(false);
        if let Ok(f) = oracle_in.recv_timeout(Duration::from_millis(10)) {
            let consultation = consult(&f);
            if oracle_out.send((f, consultation)).is_err() {
                log::debug!("outwards channel disconnected, exiting...");
                break;
            }
        }
    }
}
