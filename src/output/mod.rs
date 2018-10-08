mod tptp;

use std::sync::Arc;

use core::{Formula, Proof, Set};
use options::OutputOptions;

pub fn os_error(options: &OutputOptions) {
    tptp::szs_os_error(&options.name);
}

pub fn input_error(options: &OutputOptions) {
    tptp::szs_input_error(&options.name);
}

pub fn unsupported(options: &OutputOptions) {
    tptp::szs_inappropriate(&options.name);
}

pub fn proof_found(options: &OutputOptions, original: Set<Arc<Formula>>, proof: &Proof) {
    tptp::szs_refutation(&options.name, original, proof);
}

pub fn time_out(options: &OutputOptions) {
    tptp::szs_timeout(&options.name);
}
