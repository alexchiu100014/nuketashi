pub mod delegate;

use crate::renderer::vulkano::surface::VulkanoSurface;

use crate::renderer::*;

use winit::event_loop::EventLoop;

pub struct CpuBackend;

impl GraphicBackend for CpuBackend {}

use delegate::CpuDelegate;

pub struct CpuSurface {
    framebuffer: Option<CpuImageBuffer>,
    delegate: CpuDelegate,
}

#[derive(Default)]
pub struct CpuImageBuffer {
    width: usize,
    height: usize,
    rgba_buffer: Vec<u8>,
}

impl CpuSurface {
    pub fn new(event_loop: &EventLoop<()>) -> CpuSurface {
        CpuSurface {
            framebuffer: Some(CpuImageBuffer {
                width: crate::constants::GAME_WINDOW_WIDTH as usize,
                height: crate::constants::GAME_WINDOW_HEIGHT as usize,
                rgba_buffer: vec![],
            }),
            delegate: CpuDelegate::new(event_loop),
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
            .framebuffer
            .take()
            .expect("failed to obtain framebuffer");

        buf.rgba_buffer.clear();
        buf.rgba_buffer.resize(buf.width * buf.height * 4, 0xFF);

        Some(buf)
    }

    fn draw_end(&mut self, target: Self::Target, _: &Ctx) {
        self.delegate.draw(&target);
        self.framebuffer = Some(target);
    }
}

impl RenderingTarget<CpuBackend> for CpuImageBuffer {}

impl RenderingContext<CpuBackend> for () {}
