use std::io::stdout;

use fern::colors::ColoredLevelConfig;
use fern::Dispatch;
use log::LevelFilter::{Info, Trace, Warn};

use options::LoggingOptions;
use options::LoggingOptionsVerbosity::*;

pub fn setup(options: &LoggingOptions) {
    let global_level = match options.verbosity {
        Quiet => Warn,
        Normal => Info,
        Verbose => Trace,
    };
    let colors = ColoredLevelConfig::new();

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "% [{}][{}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        }).level(global_level)
        .chain(stdout())
        .apply()
        .expect("logging configuration failed");

    debug!("logging configured");
}
