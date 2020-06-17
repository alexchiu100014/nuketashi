//! auto-face

use super::Pass;
use crate::script::mil::command::Command;

// use crate::format::fautotbl;
use std::collections::HashMap;
// use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct AutofacePass {
    facemap: HashMap<String, String>,
}

impl AutofacePass {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Pass for AutofacePass {
    fn process(self, command: Vec<Command>) -> Vec<Command> {
        command
    }
}
