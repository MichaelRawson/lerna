mod tptp;

use std::vec::Vec;
use std::sync::Arc;

use core::{Core, Formula};
use options::OutputOptions;

pub fn print(_options: &OutputOptions, core: &Core, proof: Vec<Arc<Formula>>) {
    tptp::print(core, proof)
}
