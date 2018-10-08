pub mod tptp;

use core::Goal;
use options::{InputOptions, InputOptionsLanguage};

pub enum LoadError {
    OSError,
    InputError,
    Unsupported,
}

pub fn load(options: &InputOptions) -> Result<Goal, LoadError> {
    match options.language {
        InputOptionsLanguage::TPTP => tptp::load(&options.file),
    }
}
