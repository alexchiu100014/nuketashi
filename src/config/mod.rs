use miniserde::{json, json::Value};

use crate::constants;
use constants::NKTS_CONFIG_DEFAULT_PATH;
use constants::NKTS_CONFIG_ENV;

use lazy_static::*;

use std::collections::BTreeMap;

pub type Config = BTreeMap<String, json::Value>;

lazy_static! {
    pub static ref CONFIG: Config = open_config();
}

pub fn open_config() -> Config {
    use std::env;

    json::from_str(
        &std::fs::read_to_string(
            env::var(NKTS_CONFIG_ENV).unwrap_or(NKTS_CONFIG_DEFAULT_PATH.into()),
        )
        .unwrap(),
    )
    .expect("failed to parse a configuration file")
}

pub fn get_game_title() -> &'static str {
    match CONFIG.get("runtime.title") {
        Some(Value::String(str)) => &str,
        _ => constants::GAME_ENGINE_FULL_NAME,
    }
}

pub fn get_root_path() -> &'static str {
    match CONFIG.get("runtime.rootPath") {
        Some(Value::String(str)) => &str,
        _ => "./blob",
    }
}

use std::path::{Path, PathBuf};

pub fn find_asset<P>(path: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let mut output_path = PathBuf::from(get_root_path());
    output_path.push(&path);

    if !output_path.exists() {
        // try making it uppercase for case-insensitive filesystems
        output_path.set_file_name(
            output_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_ascii_uppercase(),
        );

        if !output_path.exists() {
            return None;
        }
    }

    Some(output_path)
}
