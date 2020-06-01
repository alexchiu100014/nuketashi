use crate::game::scene::scenario::script::vm::Vm;
use encoding_rs::SHIFT_JIS;
use std::io::Cursor;

#[test]
fn parse_script() {
    let script = include_bytes!("0X_RT_XX.txt");
    let (script, _, _) = SHIFT_JIS.decode(script);
    let script = Cursor::new(&*script);

    let mut script = Vm::new(script);

    while script.load_command_until_wait().unwrap() {
        // fuck
    }
}
