pub mod tptp;

use core::{Names, Goal};
use options::InputOptions;

pub enum LoadError {
    OSError,
    InputError,
    Unsupported,
}

pub fn load(options: &InputOptions, core: &Names) -> Result<Goal, LoadError> {
    tptp::load(&options.file, core)
}
