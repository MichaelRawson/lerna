use std::iter::{Product, Sum};
use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Sat,
    Unsat,
    Unknown,
}

use Status::*;

impl Default for Status {
    fn default() -> Self {
        Unknown
    }
}

impl Add for Status {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Unsat, _) | (_, Unsat) => Unsat,
            (Unknown, _) | (_, Unknown) => Unknown,
            (Sat, Sat) => Sat,
        }
    }
}

impl Sum for Status {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Sat, |x, y| x + y)
    }
}

impl Mul for Status {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Sat, _) | (_, Sat) => Sat,
            (Unknown, _) | (_, Unknown) => Unknown,
            (Unsat, Unsat) => Unsat,
        }
    }
}

impl Product for Status {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Unsat, |x, y| x * y)
    }
}
