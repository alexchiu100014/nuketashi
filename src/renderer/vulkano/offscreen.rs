//! Offscreen target.

use ::vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use ::vulkano::device::Queue;
use ::vulkano::format::Format;
use ::vulkano::framebuffer::{Framebuffer, RenderPassAbstract};
use ::vulkano::image::{Dimensions, ImageUsage, StorageImage};
use ::vulkano::sync::GpuFuture;

use std::sync::Arc;

use crate::renderer::vulkano::{VulkanoBackend, VulkanoRenderingContext, VulkanoRenderingTarget};
use crate::renderer::{RenderingContext, RenderingSurface};

pub struct OffscreenTexture {
    pub texture: Arc<StorageImage<Format>>,
    pub queue: Arc<Queue>,
}

pub struct OffscreenFramebuffer {
    pub framebuffer: Arc<
        Framebuffer<Arc<dyn RenderPassAbstract + Sync + Send>, ((), Arc<StorageImage<Format>>)>,
    >,
    pub command_buffer: AutoCommandBufferBuilder,
    pub future: Box<dyn GpuFuture>,
    pub dynamic_state: DynamicState,
}

impl OffscreenTexture {
    pub fn new(viewport: (u32, u32), queue: Arc<Queue>, format: Format) -> Self {
        let texture = StorageImage::with_usage(
            queue.device().clone(),
            Dimensions::Dim2d {
                width: viewport.0,
                height: viewport.1,
            },
            format,
            ImageUsage {
                sampled: true,
                transfer_source: true,
                transfer_destination: true,
                input_attachment: true,
                color_attachment: true,
                ..ImageUsage::none()
            },
            Some(queue.family()),
        )
        .unwrap();

        Self { texture, queue }
    }
}

impl<'a, Ctx> RenderingSurface<VulkanoBackend, Ctx> for OffscreenTexture
where
    Ctx: RenderingContext<VulkanoBackend> + VulkanoRenderingContext,
{
    type Target = OffscreenFramebuffer;
    type Future = Box<dyn GpuFuture>;

    fn draw_begin(&mut self, context: &Ctx) -> Option<Self::Target> {
        // create a framebuffer
        let framebuffer = Arc::new(
            Framebuffer::start(context.render_pass().clone())
                .add(self.texture.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.queue.device().clone(),
            self.queue.family(),
        )
        .ok()?;

        let future = Box::new(::vulkano::sync::now(self.queue.device().clone()));
        let dynamic_state = DynamicState::none();

        Some(Self::Target {
            framebuffer,
            command_buffer,
            future,
            dynamic_state,
        })
    }

    fn draw_end(&mut self, target: Self::Target, _: &Ctx) -> Self::Future {
        let command_buffer = target
            .command_buffer
            .build()
            .expect("failed to build command buffer");

        Box::new(
            target
                .future
                .then_execute(self.queue.clone(), command_buffer)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap(),
        )
    }
}

impl VulkanoRenderingTarget for OffscreenFramebuffer {
    fn command_buffer(&mut self) -> &mut AutoCommandBufferBuilder {
        &mut self.command_buffer
    }

    fn dynamic_state(&mut self) -> &mut DynamicState {
        &mut self.dynamic_state
    }

    fn future(&mut self) -> &mut Box<dyn GpuFuture> {
        &mut self.future
    }
}
