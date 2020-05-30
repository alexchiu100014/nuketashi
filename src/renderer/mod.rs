pub mod cpu;
pub mod vulkano;

use winit::event::Event;
use winit::event_loop::ControlFlow;

/// Grahpic backend.
pub trait GraphicBackend {}

/// Surface.
pub trait RenderingSurface<B: GraphicBackend, Ctx: RenderingContext<B>>: Sized {
    type UserEvent;
    type Target: RenderingTarget<B>;

    fn handle_event(&mut self, event: &Event<Self::UserEvent>, control_flow: &mut ControlFlow);

    fn draw(&mut self, context: &Ctx) -> Option<Self::Target>;
}

/// Resources for a renderer.
///
/// Usually contains a render pass, pipelines.
pub trait RenderingContext<B: GraphicBackend> {}

/// Rendering target. Usually a swapchain image.
pub trait RenderingTarget<B: GraphicBackend> {}

/// Renderer
pub trait Renderer<B: GraphicBackend, Ctx: RenderingContext<B>, T: RenderingTarget<B>> {
    type Error: std::error::Error;

    fn render(context: &mut Ctx, target: &mut T) -> Result<(), Self::Error>;
}
