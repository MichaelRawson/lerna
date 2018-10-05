mod tptp;

use std::sync::Arc;

use core::{Formula, Proof, Set};
use options::OutputOptions;

pub fn os_error(_options: &OutputOptions) {
    tptp::szs_os_error();
}

pub fn input_error(_options: &OutputOptions) {
    tptp::szs_input_error();
}

pub fn unsupported(_options: &OutputOptions) {
    tptp::szs_inappropriate();
}

pub fn proof_found(_options: &OutputOptions, original: Set<Arc<Formula>>, proof: &Proof) {
    tptp::szs_refutation(original, proof);
}

pub fn time_out(_options: &OutputOptions) {
    tptp::szs_timeout();
}
