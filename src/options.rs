use std::path::Path;

use clap::{App, Arg, ArgMatches};

pub enum LoggingOptionsVerbosity {
    Quiet,
    Normal,
    Verbose,
}

impl LoggingOptionsVerbosity {
    fn new(flag: &str) -> Self {
        match flag {
            "quiet" => LoggingOptionsVerbosity::Quiet,
            "normal" => LoggingOptionsVerbosity::Normal,
            "verbose" => LoggingOptionsVerbosity::Verbose,
            _ => unreachable!(),
        }
    }
}

pub struct LoggingOptions {
    pub verbosity: LoggingOptionsVerbosity,
}

impl LoggingOptions {
    fn new(matches: &ArgMatches) -> Self {
        let logging_level = matches.value_of("logging").unwrap();
        let verbosity = LoggingOptionsVerbosity::new(logging_level);
        LoggingOptions { verbosity }
    }
}

pub struct InputOptions {
    pub file: String,
}

impl InputOptions {
    fn new(matches: &ArgMatches) -> Self {
        let file = matches.value_of("FILE").unwrap().into();
        InputOptions { file }
    }
}

pub struct OutputOptions {
    pub name: String,
}

impl OutputOptions {
    fn get_name(file: &str) -> Option<&str> {
        let path = Path::new(file);
        path.file_stem()?.to_str()
    }

    fn new(matches: &ArgMatches) -> Self {
        let name = Self::get_name(matches.value_of("FILE").unwrap())
            .unwrap_or("<unknown>")
            .into();
        OutputOptions { name }
    }
}

pub struct SearchOptions {
    pub timeout: usize,
}

impl SearchOptions {
    fn new(matches: &ArgMatches) -> Self {
        let timeout = matches.value_of("timeout").unwrap().parse().unwrap();
        SearchOptions { timeout }
    }
}

pub struct Options {
    pub logging: LoggingOptions,
    pub input: InputOptions,
    pub output: OutputOptions,
    pub search: SearchOptions,
}

fn validate_seconds(secs: &str) -> Result<(), String> {
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
                    .validator(|x| validate_seconds(&x))
                    .default_value("30"),
            ).arg(
                Arg::with_name("logging")
                    .long("logging")
                    .takes_value(true)
                    .value_name("LEVEL")
                    .possible_value("quiet")
                    .possible_value("normal")
                    .possible_value("verbose")
                    .default_value("normal"),
            ).get_matches();

        let logging = LoggingOptions::new(&matches);
        let input = InputOptions::new(&matches);
        let output = OutputOptions::new(&matches);
        let search = SearchOptions::new(&matches);
        Options {
            logging,
            input,
            output,
            search,
        }
    }
}
