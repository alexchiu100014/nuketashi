pub mod delegate;
pub mod image;
pub mod layer;
pub mod utils;

use crate::renderer::vulkano::surface::VulkanoSurface;
use crate::renderer::*;

use winit::event_loop::EventLoop;

use std::collections::VecDeque;

pub struct CpuBackend;

impl GraphicBackend for CpuBackend {}

use delegate::{CpuDelegate, CpuImageBuffer};

pub struct CpuSurface {
    framebuffers: VecDeque<CpuImageBuffer>,
    delegate: CpuDelegate,
}

impl CpuSurface {
    pub fn new(event_loop: &EventLoop<()>) -> CpuSurface {
        let delegate = CpuDelegate::new(event_loop);

        let mut framebuffers = VecDeque::new();
        framebuffers.resize_with(2, || delegate.create_framebuffer(1600, 900));

        CpuSurface {
            framebuffers,
            delegate,
        }
    }

    pub fn request_redraw(&mut self) {
        self.delegate.surface.surface.window().request_redraw();
    }
}

impl EventDelegate for CpuSurface {
    type UserEvent = <VulkanoSurface<'static> as EventDelegate>::UserEvent;

    fn handle_event(&mut self, event: &Event<Self::UserEvent>, control_flow: &mut ControlFlow) {
        self.delegate.surface.handle_event(event, control_flow)
    }
}

impl<Ctx> RenderingSurface<CpuBackend, Ctx> for CpuSurface
where
    Ctx: RenderingContext<CpuBackend>,
{
    type Target = CpuImageBuffer;

    fn draw_begin(&mut self, _: &Ctx) -> Option<Self::Target> {
        let mut buf = self
            .framebuffers
            .pop_front()
            .expect("failed to obtain framebuffer");

        for v in &mut buf.rgba_buffer {
            *v = 0x00;
        }

        Some(buf)
    }

    fn draw_end(&mut self, target: Self::Target, _: &Ctx) {
        self.delegate.draw(&target);
        self.framebuffers.push_back(target);
    }
}

impl RenderingTarget<CpuBackend> for CpuImageBuffer {}

impl RenderingContext<CpuBackend> for () {}
