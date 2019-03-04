use crate::collections::Set;
use crate::formula::Formula;

pub struct Justification {
    pub rule: &'static str,
    pub parents: Set<Formula>,
}
