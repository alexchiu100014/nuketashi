// Game engine name.
pub(crate) static GAME_ENGINE_FULL_NAME: &'static str = "冷泉院桐香 v2.50a";
pub(crate) static GAME_ENGINE_NAME: &'static str = "ReizeiinTouka";

// nukitashi uses 1600x900 as a global resolution
pub(crate) const GAME_WINDOW_WIDTH: u32 = 1600;
pub(crate) const GAME_WINDOW_HEIGHT: u32 = 900;

pub(crate) const TOTAL_LAYERS: i32 = 25;

// font
pub(crate) static FONT_BYTES: &[u8] =
    include_bytes!("../blob/NUKITASHI_D.WAR/ROUNDED-X-MGENPLUS-1M.TTF");

pub(crate) static LRU_CACHE_CAPACITY: usize = 20;
