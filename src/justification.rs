use std::sync::Arc;

use collections::Set;
use formula::Formula;

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Justification {
    derived: Arc<Formula>,
    from: Set<Arc<Formula>>,
    method: &'static str
}
