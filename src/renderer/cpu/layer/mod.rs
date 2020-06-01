use crate::renderer::Renderer;
use crate::format::s25::S25Archive;

pub struct LayerRenderer {
    s25_cache: Option<S25Archive>,
}
