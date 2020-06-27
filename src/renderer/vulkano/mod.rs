pub mod instance;
pub mod layer;
pub mod pipeline;
pub mod shaders;
pub mod surface;
pub mod text;
pub mod texture_loader;

use crate::renderer::*;
use ::vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use ::vulkano::framebuffer::RenderPassAbstract;
use std::sync::Arc;

pub struct VulkanoBackend;

impl GraphicBackend for VulkanoBackend {}

pub trait VulkanoRenderingContext {
    fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Sync + Send>;
}

pub trait VulkanoRenderingTarget {
    fn command_buffer(&mut self) -> &mut AutoCommandBufferBuilder;
    fn dynamic_state(&mut self) -> &mut DynamicState;
}

impl<T> RenderingContext<VulkanoBackend> for T where T: VulkanoRenderingContext {}

impl<T> RenderingTarget<VulkanoBackend> for T where T: VulkanoRenderingTarget {}
