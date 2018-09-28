use clap::{App, Arg, ArgMatches};

pub struct CoreOptions {}

impl CoreOptions {
    pub fn new(_matches: &ArgMatches) -> Self {
        CoreOptions {}
    }
}

pub struct LoggingOptions {
    pub debug: bool
}

impl LoggingOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let debug = matches.is_present("debug");
        LoggingOptions {
            debug
        }
    }
}

pub struct InputOptions {
    pub file: String,
}

impl InputOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let file = matches.value_of("FILE").unwrap().into();
        InputOptions {
            file
        }
    }
}


pub struct OutputOptions {}

impl OutputOptions {
    pub fn new(_matches: &ArgMatches) -> Self {
        OutputOptions {}
    }
}

pub struct SearchOptions {}

impl SearchOptions {
    pub fn new(_matches: &ArgMatches) -> Self {
        SearchOptions {}
    }
}

pub struct Options {
    pub core: CoreOptions,
    pub logging: LoggingOptions,
    pub input: InputOptions,
    pub output: OutputOptions,
    pub search: SearchOptions
}

impl Options {
    pub fn parse() -> Self {
        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(Arg::with_name("FILE")
                 .help("load problem from this file")
                 .required(true)
                 .index(1)
            )
            .arg(Arg::with_name("debug")
                 .long("debug")
                 .help("log debugging information")
            )
            .get_matches();

        let core = CoreOptions::new(&matches);
        let logging = LoggingOptions::new(&matches);
        let input = InputOptions::new(&matches);
        let output = OutputOptions::new(&matches);
        let search = SearchOptions::new(&matches);
        Options {core, logging, input, output, search}
    }
}
