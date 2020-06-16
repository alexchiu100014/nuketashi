//! Prefetch pass.
//!
//! Adds prefetch commands.

use super::Pass;
use crate::script::mil::command::{Command, LayerCommand, RuntimeCommand};

#[derive(Clone, Debug, Default)]
pub struct PrefetchPass;

impl PrefetchPass {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Pass for PrefetchPass {
    fn process(self, command: Vec<Command>) -> Vec<Command> {
        let command_chunks = command.split(|v| {
            if let Command::RuntimeCommand(RuntimeCommand::WaitUntilUserEvent) = v {
                true
            } else {
                false
            }
        });

        let mut prefetch = vec![];
        let mut postfetch = vec![];
        let mut output = vec![];

        for ch in command_chunks {
            if !output.is_empty() {
                output.append(&mut prefetch);
                output.push(Command::RuntimeCommand(RuntimeCommand::WaitUntilUserEvent));
                output.append(&mut postfetch);
            } else {
                output.append(&mut postfetch);
            }

            for cmd in ch {
                match cmd {
                    Command::LayerCommand {
                        layer_no,
                        command: LayerCommand::Load(filename, entries),
                    } => {
                        prefetch.push(Command::LayerCommand {
                            layer_no: *layer_no,
                            command: LayerCommand::Prefetch(filename.clone(), entries.clone()),
                        });
                    }
                    _ => {}
                }

                postfetch.push(cmd.clone());
            }
        }

        output.append(&mut prefetch);
        output.push(Command::RuntimeCommand(RuntimeCommand::WaitUntilUserEvent));
        output.append(&mut postfetch);

        output
    }
}
