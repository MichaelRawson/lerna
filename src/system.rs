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

pub fn time_out<T>() -> T {
    println!("% SZS Status TimeOut");
    exit(1);
}

pub fn check_for_timeout() {
    if !within_time() {
        time_out()
    }
}

pub fn within_time() -> bool {
    let elapsed = OPTIONS.start_time.elapsed().unwrap_or_default();
    elapsed < OPTIONS.time
}
