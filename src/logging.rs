use fern::Dispatch;
use log::LevelFilter;
use std::io::stderr;

use crate::options::OPTIONS;

pub fn initialize() {
    let level = if OPTIONS.quiet {
        LevelFilter::Error
    } else {
        LevelFilter::Debug
    };

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}]\t[{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(stderr())
        .apply()
        .expect("failed to start application logging");
}
