pub mod tptp;

use crate::goal::Goal;
use crate::options::{InputOptions, InputOptionsLanguage};

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
