use clap::{App, Arg, ArgMatches};
use lazy_static::{initialize, lazy_static};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use crate::deduction;
use crate::deduction::Deduction;
use crate::simplification;
use crate::simplification::Simplification;
use crate::system::time_out;

pub struct Options {
    pub file: String,
    pub start_time: SystemTime,
    pub time: Duration,
    pub skepticism: f32,
    pub goal_queue_size: usize,
    pub score_queue_size: usize,
    pub deductions: Vec<Box<dyn Deduction>>,
    pub simplifications: Vec<Box<dyn Simplification>>,
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
    pub fn within_time(&self) -> bool {
        let elapsed = self.start_time.elapsed().unwrap_or_default();
        elapsed < self.time
    }

    pub fn check_time(&self) {
        if !self.within_time() {
            time_out()
        }
    }

    fn new() -> Self {
        let start_time = SystemTime::now();
        log::info!("parsing options...");

        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name("FILE")
                    .help("load problem from this file")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("time")
                    .help("set the prover timeout")
                    .long("time")
                    .short("t")
                    .takes_value(true)
                    .value_name("SECS")
                    .validator(|x| validate::<u32>(&x, "should be a positive number of seconds"))
                    .default_value("30"),
            )
            .arg(
                Arg::with_name("skepticism")
                    .help("balance between exploitation and exploration")
                    .long("skepticism")
                    .takes_value(true)
                    .value_name("C")
                    .validator(|x| validate::<f32>(&x, "should be a floating-point number"))
                    .default_value("2"),
            )
            .arg(
                Arg::with_name("goal_queue_size")
                    .help("the maximum number of goals queued for evaluation")
                    .long("goal_queue_size")
                    .takes_value(true)
                    .value_name("SIZE")
                    .validator(|x| validate::<usize>(&x, "should be a positive number"))
                    .default_value("1024"),
            )
            .arg(
                Arg::with_name("score_queue_size")
                    .help("the maximum number of scores queued for processing")
                    .long("score_queue_size")
                    .takes_value(true)
                    .value_name("SIZE")
                    .validator(|x| validate::<usize>(&x, "should be a positive number"))
                    .default_value("1024"),
            )
            .get_matches();

        let file = get_validated_arg(&matches, "FILE");
        let time = Duration::from_secs(get_validated_arg(&matches, "time"));
        let skepticism = get_validated_arg(&matches, "skepticism");
        let goal_queue_size = get_validated_arg(&matches, "goal_queue_size");
        let score_queue_size = get_validated_arg(&matches, "score_queue_size");

        let deductions: Vec<Box<dyn Deduction>> = vec![Box::new(deduction::axiom::Axiom)];
        let simplifications: Vec<Box<dyn Simplification>> =
            vec![Box::new(simplification::contradiction::Contradiction)];

        log::info!("...options OK");
        Options {
            file,
            start_time,
            time,
            skepticism,
            goal_queue_size,
            score_queue_size,
            deductions,
            simplifications,
        }
    }
}

lazy_static! {
    pub static ref OPTIONS: Options = Options::new();
}

pub fn parse() {
    initialize(&OPTIONS);
}
