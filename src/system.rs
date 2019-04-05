use std::process::exit;

use crate::options::OPTIONS;

pub fn os_error<T>() -> T {
    println!("% SZS Status OSError");
    exit(1);
}

pub fn input_error<T>() -> T {
    println!("% SZS Status InputError");
    exit(1);
}

pub fn time_out<T>(print: bool) -> T {
    if print {
        println!("% SZS Status TimeOut");
    }
    exit(1);
}

pub fn check_for_timeout(print: bool) {
    if !within_time() {
        time_out(print)
    }
}

pub fn within_time() -> bool {
    let elapsed = OPTIONS.start_time.elapsed().unwrap_or_default();
    elapsed < OPTIONS.time
}
