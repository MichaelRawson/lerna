use std::io::stderr;

use fern::Dispatch;
use fern::colors::ColoredLevelConfig;
use log::LevelFilter::{Debug, Info};

use options::LoggingOptions;

pub fn setup(options: &LoggingOptions) {
    let global_level = if options.debug { Debug } else { Info };
    let colors = ColoredLevelConfig::new();

    Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "% [{}][{}] {}",
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(global_level)
        .chain(stderr())
        .apply()
        .expect("logging configuration failed");
}
