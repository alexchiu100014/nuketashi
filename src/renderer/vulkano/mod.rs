pub mod instance;
pub mod surface;

use crate::renderer::*;
use ::vulkano::framebuffer::RenderPassAbstract;
use std::sync::Arc;

pub struct VulkanoBackend;

impl GraphicBackend for VulkanoBackend {}

pub trait VulkanoRenderingContext {
    fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Sync + Send>;
}
