mod complete;
mod weakening;

use std::collections::HashSet;
use unique::Id;

use crate::collections::IdSet;
use crate::formula::Formula;
use crate::simplification::simplify;

pub fn deductions(f: &Id<Formula>) -> HashSet<IdSet<Formula>> {
    let mut deduced = HashSet::new();
    complete::complete_deductions(&mut deduced, f);
    weakening::weakening_deductions(&mut deduced, f);

    deduced
        .into_iter()
        .map(|inference| inference.into_iter().map(|f| simplify(&f)).collect())
        .collect()
}
