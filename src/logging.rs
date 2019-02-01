use fern::Dispatch;
use std::io::stderr;

pub fn initialise() {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}]\t[{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(stderr())
        .apply()
        .expect("failed to start application logging");
}
