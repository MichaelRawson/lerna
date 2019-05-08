use clap::{App, Arg, ArgMatches};
use lazy_static::lazy_static;
use std::str::FromStr;
use std::time::Duration;

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
    // global options
    pub exploration: f32,
    pub file: String,
    pub mode: Mode,
    pub time: Duration,
    pub quiet: bool,
    // oracle options
    pub oracle: Oracle,
    pub oracle_threads: u16,
    pub oracle_iterations: u64,
    pub oracle_timeout: u16,
    //heuristic options
    pub heuristic_address: String,
    pub heuristic_off: bool,
    // record options
    pub record_file: Option<String>,
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
                Arg::with_name("exploration")
                    .help("Exploration constant: higher values give broader search")
                    .long("exploration")
                    .takes_value(true)
                    .value_name("C")
                    .validator(|x| {
                        validate::<f32>(&x, "should be a floating-point number")
                    })
                    .default_value("1.4142"),
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
                Arg::with_name("quiet")
                    .short("q")
                    .long("quiet")
                    .help("Turns off logging, except errors"),
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
                Arg::with_name("oracle threads")
                    .help("Number of threads used for oracle")
                    .long("oracle_threads")
                    .takes_value(true)
                    .value_name("THREADS")
                    .validator(|x| {
                        validate::<u16>(
                            &x,
                            "should be a positive number of threads",
                        )
                    })
                    .default_value("8"),
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
                    .default_value("100"),
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
                    .default_value("20"),
            )
            .arg(
                Arg::with_name("heuristic address")
                    .help("heuristic IP address/port number")
                    .short("a")
                    .long("heuristic_address")
                    .takes_value(true)
                    .value_name("ADDR:PORT")
                    .default_value("localhost:1337"),
            )
            .arg(
                Arg::with_name("heuristic off")
                    .help("Turns off heuristic")
                    .long("heuristic_off"),
            )
            .arg(
                Arg::with_name("record file")
                    .help("Record subgoal outcomes to RECORD_FILE")
                    .long("record_file")
                    .takes_value(true)
                    .value_name("RECORD_FILE"),
            )
            .get_matches();

        let file = get_validated_arg(&matches, "FILE");
        let mode = get_validated_arg(&matches, "mode");
        let time = Duration::from_secs(get_validated_arg(&matches, "time"));
        let exploration = get_validated_arg(&matches, "exploration");
        let quiet = matches.is_present("quiet");

        let oracle = get_validated_arg(&matches, "oracle");
        let oracle_threads = get_validated_arg(&matches, "oracle threads");
        let oracle_iterations =
            get_validated_arg(&matches, "oracle iterations");
        let oracle_timeout = get_validated_arg(&matches, "oracle timeout");

        let heuristic_address =
            get_validated_arg(&matches, "heuristic address");
        let heuristic_off = matches.is_present("heuristic off");

        let record_file = matches.value_of("record file").map(|x| x.into());

        Options {
            exploration,
            file,
            mode,
            time,
            quiet,
            oracle,
            oracle_threads,
            oracle_iterations,
            oracle_timeout,
            heuristic_address,
            heuristic_off,
            record_file,
        }
    }
}

lazy_static! {
    pub static ref OPTIONS: Options = Options::new();
}

pub fn initialize() {
    lazy_static::initialize(&OPTIONS);
}
