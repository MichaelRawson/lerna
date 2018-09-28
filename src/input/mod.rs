pub mod tptp;

use core::{Core, Goal};
use options::InputOptions;

pub fn load(options: &InputOptions, core: &Core) -> Goal {
    tptp::load(&options.file, core)
}
