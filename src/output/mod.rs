mod tptp;

use std::io::stdout;
use std::sync::Arc;

use fern::Dispatch;
use log::LevelFilter::{Info, Trace, Warn};

use collections::Set;
use formula::Formula;
use options::{OutputOptions, OutputOptionsLanguage, OutputOptionsLoggingLevel};
use proof::RawProof;

pub fn os_error(options: &OutputOptions) {
    match options.language {
        OutputOptionsLanguage::TPTP => tptp::szs_os_error(&options.name),
    }
}

pub fn input_error(options: &OutputOptions) {
    match options.language {
        OutputOptionsLanguage::TPTP => tptp::szs_input_error(&options.name),
    }
}

pub fn unsupported(options: &OutputOptions) {
    match options.language {
        OutputOptionsLanguage::TPTP => tptp::szs_inappropriate(&options.name),
    }
}

pub fn proof_found(options: &OutputOptions, original: Set<Arc<Formula>>, proof: &RawProof) {
    match options.language {
        OutputOptionsLanguage::TPTP => tptp::szs_refutation(&options.name, original, proof),
    }
}

pub fn time_out(options: &OutputOptions) {
    match options.language {
        OutputOptionsLanguage::TPTP => tptp::szs_timeout(&options.name),
    }
}

pub fn setup_logging(options: &OutputOptions) {
    let global_level = match options.logging_level {
        OutputOptionsLoggingLevel::Quiet => Warn,
        OutputOptionsLoggingLevel::Normal => Info,
        OutputOptionsLoggingLevel::Verbose => Trace,
    };

    Dispatch::new()
        .format(match options.language {
            OutputOptionsLanguage::TPTP => tptp::format_log,
        }).level(global_level)
        .chain(stdout())
        .apply()
        .expect("logging configuration failed");

    debug!("logging configured");
}
