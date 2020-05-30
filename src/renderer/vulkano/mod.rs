pub mod instance;
pub mod surface;
pub mod pipeline;
pub mod text;
pub mod layer;
pub mod shaders;
pub mod texture_loader;

use crate::renderer::*;
use ::vulkano::framebuffer::RenderPassAbstract;
use std::sync::Arc;

pub struct VulkanoBackend;

impl GraphicBackend for VulkanoBackend {}

pub trait VulkanoRenderingContext {
    fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Sync + Send>;
}
