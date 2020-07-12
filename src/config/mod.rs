use miniserde::{json, json::Value};

use crate::constants;
use constants::NKTS_CONFIG_DEFAULT_PATH;
use constants::NKTS_CONFIG_ENV;

use lazy_static::*;

use std::collections::BTreeMap;

pub type Config = BTreeMap<String, json::Value>;

lazy_static! {
    pub static ref CONFIG: Config = {
        open_config()
    };
}

pub fn open_config() -> Config {
    use std::env;

    json::from_str(
        &std::fs::read_to_string(
            env::var(NKTS_CONFIG_ENV).unwrap_or(NKTS_CONFIG_DEFAULT_PATH.into()),
        )
        .unwrap(),
    ).expect("failed to parse a configuration file")
}

pub fn get_game_title() -> &'static str {
    match CONFIG.get("runtime.title") {
        Some(Value::String(str)) => &str,
        _ => constants::GAME_ENGINE_FULL_NAME,
    }
}
