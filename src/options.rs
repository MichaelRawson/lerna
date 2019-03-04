use clap::{App, Arg, ArgMatches};
use lazy_static::lazy_static;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use crate::deduction;
use crate::deduction::Deduction;
use crate::simplification;
use crate::simplification::Simplification;

pub struct Options {
    pub file: String,
    pub start_time: SystemTime,
    pub time: Duration,
    pub skepticism: f32,
    pub goal_queue_size: usize,
    pub deductions: Vec<Box<dyn Deduction>>,
    pub simplifications: Vec<Box<dyn Simplification>>,
    pub quiet: bool,
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
                Arg::with_name("goal_queue_size")
                    .help("Max goals queued")
                    .long("goal_queue_size")
                    .takes_value(true)
                    .value_name("SIZE")
                    .validator(|x| {
                        validate::<usize>(&x, "should be a positive number")
                    })
                    .default_value("1024"),
            )
            .arg(
                Arg::with_name("quiet")
                    .short("q")
                    .long("quiet")
                    .help("Turns off logging, except errors"),
            )
            .get_matches();

        let file = get_validated_arg(&matches, "FILE");
        let time = Duration::from_secs(get_validated_arg(&matches, "time"));
        let skepticism = get_validated_arg(&matches, "skepticism");
        let goal_queue_size = get_validated_arg(&matches, "goal_queue_size");
        let quiet = matches.is_present("quiet");

        let deductions: Vec<Box<dyn Deduction>> =
            vec![Box::new(deduction::axiom::Axiom)];
        let simplifications: Vec<Box<dyn Simplification>> =
            vec![Box::new(simplification::contradiction::Contradiction)];

        Options {
            file,
            start_time,
            time,
            skepticism,
            goal_queue_size,
            deductions,
            simplifications,
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
