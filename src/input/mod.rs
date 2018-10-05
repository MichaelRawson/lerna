pub mod tptp;

use core::Goal;
use options::InputOptions;

pub enum LoadError {
    OSError,
    InputError,
    Unsupported,
}

pub fn load(options: &InputOptions) -> Result<Goal, LoadError> {
    tptp::load(&options.file)
}
