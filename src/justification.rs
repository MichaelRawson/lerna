use std::sync::Arc;

use parking_lot::Mutex;

use collections::Set;
use formula::Formula;

lazy_static! {
    static ref RECORDED: Mutex<Vec<Justification>> = Mutex::new(vec![]);
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Justification {
    derived: Arc<Formula>,
    from: Set<Arc<Formula>>,
    method: &'static str
}
