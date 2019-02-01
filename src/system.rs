use std::process::exit;

pub fn os_error<T>() -> T {
    println!("% SZS Status OSError");
    exit(1);
}

pub fn input_error<T>() -> T {
    println!("% SZS Status InputError");
    exit(1);
}

pub fn give_up<T>() -> T {
    println!("% SZS Status GaveUp");
    exit(1);
}

pub fn time_out<T>() -> T {
    println!("% SZS Status TimeOut");
    exit(1);
}
