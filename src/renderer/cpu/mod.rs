use crate::renderer::*;

pub struct CpuBackend;

impl GraphicBackend for CpuBackend {}

pub struct CpuSurface {
    frontbuffer: CpuImageBuffer,
    backbuffer: Option<CpuImageBuffer>,
}

#[derive(Default)]
pub struct CpuImageBuffer {
    width: usize,
    height: usize,
    rgba_buffer: Vec<u8>,
}

impl<Ctx> RenderingSurface<CpuBackend, Ctx> for CpuSurface
where
    Ctx: RenderingContext<CpuBackend>,
{
    type Target = CpuImageBuffer;

    fn draw_begin(&mut self, _: &Ctx) -> Option<Self::Target> {
        let mut buf = self.backbuffer.take().expect("failed to obtain backbuffer");

        buf.rgba_buffer.clear();
        buf.rgba_buffer.resize(buf.width * buf.height * 4, 0);

        Some(buf)
    }

    fn draw_end(&mut self, target: Self::Target) {
        // swap buffer
        self.backbuffer = Some(std::mem::replace(&mut self.frontbuffer, target));
    }
}

impl RenderingTarget<CpuBackend> for CpuImageBuffer {}
