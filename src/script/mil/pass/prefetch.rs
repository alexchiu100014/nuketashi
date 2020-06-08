//! Prefetch pass.
use crate::script::mil::command::Command;

pub struct PrefetchPass {
    buf: Vec<Command>,
    chunk: Vec<Command>,
    load_commands: Vec<Command>,
}

