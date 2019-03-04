use crate::collections::Set;
use crate::formula::Formula;

pub struct Inference {
    pub add: Set<Formula>,
    pub remove: Set<Formula>,
}
