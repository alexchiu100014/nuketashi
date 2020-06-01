pub mod script;

use script::vm::Vm;

/// Scenario player.
pub struct Scenario {
    pub vm: Vm<std::io::Cursor<String>>,
}

impl Scenario {
    pub fn new(scenario: String) -> Self {
        use std::io::Cursor;
        let scenario = Cursor::new(scenario);

        Scenario {
            vm: Vm::new(scenario)
        }
    }
}
