//! Passes and optimizers.

pub mod autoface;
pub mod log_entry;
pub mod prefetch;

use crate::script::mil::command::Command;

pub trait Pass {
    fn process(self, commands: Vec<Command>) -> Vec<Command>;
}
