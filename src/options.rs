use clap::{App, Arg, ArgMatches};

pub struct CoreOptions {}

impl CoreOptions {
    pub fn new(_matches: &ArgMatches) -> Self {
        CoreOptions {}
    }
}

pub enum LoggingOptionsVerbosity {
    Quiet,
    Normal,
    Verbose,
}

pub struct LoggingOptions {
    pub verbosity: LoggingOptionsVerbosity,
}

impl LoggingOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let verbosity = if matches.is_present("quiet") {
            LoggingOptionsVerbosity::Quiet
        } else if matches.is_present("verbose") {
            LoggingOptionsVerbosity::Verbose
        } else {
            LoggingOptionsVerbosity::Normal
        };
        LoggingOptions { verbosity }
    }
}

pub struct InputOptions {
    pub file: String,
}

impl InputOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let file = matches.value_of("FILE").unwrap().into();
        InputOptions { file }
    }
}

pub struct OutputOptions {}

impl OutputOptions {
    pub fn new(_matches: &ArgMatches) -> Self {
        OutputOptions {}
    }
}

pub struct SearchOptions {
    pub timeout: usize,
}

impl SearchOptions {
    pub fn new(matches: &ArgMatches) -> Self {
        let timeout = matches.value_of("timeout").unwrap().parse().unwrap();
        SearchOptions { timeout }
    }
}

pub struct Options {
    pub core: CoreOptions,
    pub logging: LoggingOptions,
    pub input: InputOptions,
    pub output: OutputOptions,
    pub search: SearchOptions,
}

fn validate_seconds(secs: String) -> Result<(), String> {
    match secs.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("should be a time (in seconds)".to_string()),
    }
}

impl Options {
    pub fn parse() -> Self {
        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name("FILE")
                    .help("load problem from this file")
                    .required(true)
                    .index(1),
            ).arg(
                Arg::with_name("timeout")
                    .long("timeout")
                    .short("t")
                    .takes_value(true)
                    .value_name("SECS")
                    .validator(validate_seconds)
                    .default_value("60"),
            ).arg(
                Arg::with_name("quiet")
                    .long("quiet")
                    .short("q")
                    .help("suppress all non-essential output (overrides -v)"),
            ).arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .help("debugging output, extremely verbose"),
            ).get_matches();

        let core = CoreOptions::new(&matches);
        let logging = LoggingOptions::new(&matches);
        let input = InputOptions::new(&matches);
        let output = OutputOptions::new(&matches);
        let search = SearchOptions::new(&matches);
        Options {
            core,
            logging,
            input,
            output,
            search,
        }
    }
}
