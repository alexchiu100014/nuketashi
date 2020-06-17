//! generates log entry

use super::Pass;
use crate::script::mil::command::Command;

#[derive(Clone, Debug, Default)]
pub struct LogEntryPass;

impl LogEntryPass {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Pass for LogEntryPass {
    fn process(self, command: Vec<Command>) -> Vec<Command> {
        command
    }
}
