pub mod tptp;

use core::{Core, Goal};
use options::InputOptions;

pub enum LoadError {
    OSError,
    InputError,
    Unsupported,
}

pub fn load(options: &InputOptions, core: &Core) -> Result<Goal, LoadError> {
    tptp::load(&options.file, core)
}
