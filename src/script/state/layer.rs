use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
pub struct LayerModel {
    // 
    pub filename: Option<PathBuf>,
    pub entries: Vec<i32>,
    pub origin: (i32, i32),
    pub opacity: f32,
    pub blur_radius: (i32, i32),
    // inner state
    delay_state: Option<Instant>,
}

pub enum LayerCommand {
    LayerDelay(Duration),
}
