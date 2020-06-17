use crate::format::s25::{S25Archive, S25Image};
use crate::renderer::Renderer;

use std::sync::Arc;

use lru::LruCache;

pub struct LayerRenderer {
    pub s25: Option<S25Archive>,
    pub filename: Option<String>,
    pub entries: Vec<(i32, Arc<S25Image>)>,
    pub cache: LruCache<(String, i32), Arc<S25Image>>,
    pub framebuffer: Vec<f32>,
}
