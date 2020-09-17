pub mod common;
pub mod cpu;
pub mod vulkano;

use winit::event::Event;
use winit::event_loop::ControlFlow;

/// Grahpic backend.
pub trait GraphicBackend {}

pub trait EventDelegate {
    type UserEvent;

    fn handle_event(&mut self, event: &Event<Self::UserEvent>, control_flow: &mut ControlFlow);
}

/// Surface.
pub trait RenderingSurface<B: GraphicBackend, Ctx: RenderingContext<B>> {
    type Target: RenderingTarget<B>;
    type Future;

    /// Begins a draw command.
    fn draw_begin(&mut self, context: &Ctx) -> Option<Self::Target>;

    /// Finalizes a draw command.
    fn draw_end(&mut self, target: Self::Target, context: &Ctx) -> Self::Future;
}

/// Resources for a renderer.
///
/// Usually contains a render pass, pipelines, and shared between renderers.
pub trait RenderingContext<B: GraphicBackend> {}

/// Rendering target. Usually a swapchain image.
pub trait RenderingTarget<B: GraphicBackend> {}

/// Renderer.
pub trait Renderer<B: GraphicBackend, T: RenderingTarget<B>> {
    type Context: RenderingContext<B>;

    fn render(&mut self, target: &mut T, context: &Self::Context);
}
