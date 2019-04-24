use clap::{App, Arg, ArgMatches};
use lazy_static::lazy_static;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use crate::heuristic::Heuristic;
use crate::oracle::Oracle;

pub enum Mode {
    Baseline,
    Prover,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        use Mode::*;
        match x {
            "baseline" => Ok(Baseline),
            "prover" => Ok(Prover),
            _ => Err(()),
        }
    }
}

pub struct Options {
    pub mode: Mode,
    pub file: String,
    pub start_time: SystemTime,
    pub time: Duration,
    pub heuristic: Heuristic,
    pub oracle: Oracle,
    pub oracle_iterations: u64,
    pub oracle_timeout: u16,
    pub quiet: bool,
    pub skepticism: f32,
}

fn validate<T: FromStr>(arg: &str, error: &str) -> Result<(), String> {
    arg.parse::<T>().map(|_| ()).map_err(|_| error.into())
}

fn get_validated_arg<T: FromStr>(matches: &ArgMatches, name: &str) -> T {
    matches
        .value_of(name)
        .unwrap()
        .parse()
        .unwrap_or_else(|_| panic!("bad argument validation, check your code"))
}

impl Options {
    fn new() -> Self {
        let start_time = SystemTime::now();

        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name("FILE")
                    .help("the input problem")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("mode")
                    .help("Mode of operation")
                    .long("mode")
                    .takes_value(true)
                    .value_name("MODE")
                    .possible_values(&["baseline", "prover"])
                    .default_value("prover"),
            )
            .arg(
                Arg::with_name("time")
                    .help("Prover timeout")
                    .long("time")
                    .short("t")
                    .takes_value(true)
                    .value_name("SECS")
                    .validator(|x| {
                        validate::<u32>(
                            &x,
                            "should be a positive number of seconds",
                        )
                    })
                    .default_value("30"),
            )
            .arg(
                Arg::with_name("heuristic")
                    .help("Heuristic to use")
                    .long("heuristic")
                    .takes_value(true)
                    .value_name("HEURISTIC")
                    .possible_values(&["null"])
                    .default_value("null"),
            )
            .arg(
                Arg::with_name("oracle")
                    .help("Oracle to use")
                    .long("oracle")
                    .takes_value(true)
                    .value_name("ORACLE")
                    .possible_values(&["null", "z3"])
                    .default_value("z3"),
            )
            .arg(
                Arg::with_name("oracle iterations")
                    .help("Oracle maximum iterations")
                    .long("oracle_iterations")
                    .takes_value(true)
                    .value_name("ITERATIONS")
                    .validator(|x| {
                        validate::<u64>(
                            &x,
                            "should be a positive number of iterations",
                        )
                    })
                    .default_value("1000000"),
            )
            .arg(
                Arg::with_name("oracle timeout")
                    .help("Oracle time limit")
                    .long("oracle_timeout")
                    .takes_value(true)
                    .value_name("MILLIS")
                    .validator(|x| {
                        validate::<u16>(
                            &x,
                            "should be a positive number of milliseconds",
                        )
                    })
                    .default_value("10"),
            )
            .arg(
                Arg::with_name("skepticism")
                    .help("Exploitation constant")
                    .long("skepticism")
                    .takes_value(true)
                    .value_name("C")
                    .validator(|x| {
                        validate::<f32>(&x, "should be a floating-point number")
                    })
                    .default_value("2"),
            )
            .arg(
                Arg::with_name("quiet")
                    .short("q")
                    .long("quiet")
                    .help("Turns off logging, except errors"),
            )
            .get_matches();

        let file = get_validated_arg(&matches, "FILE");
        let mode = get_validated_arg(&matches, "mode");
        let time = Duration::from_secs(get_validated_arg(&matches, "time"));
        let heuristic = get_validated_arg(&matches, "heuristic");
        let oracle = get_validated_arg(&matches, "oracle");
        let oracle_iterations =
            get_validated_arg(&matches, "oracle iterations");
        let oracle_timeout = get_validated_arg(&matches, "oracle timeout");
        let skepticism = get_validated_arg(&matches, "skepticism");
        let quiet = matches.is_present("quiet");

        Options {
            file,
            mode,
            start_time,
            time,
            heuristic,
            oracle,
            oracle_iterations,
            oracle_timeout,
            skepticism,
            quiet,
        }
    }
}

lazy_static! {
    pub static ref OPTIONS: Options = Options::new();
}

pub fn initialize() {
    lazy_static::initialize(&OPTIONS);
}
