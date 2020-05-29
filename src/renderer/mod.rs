pub mod cpu;
pub mod vulkano;

/// Grahpic backend.
pub trait GraphicBackend {}

/// Surface.
pub trait Surface<B: GraphicBackend>: Sized {
    type Context: RenderingContext<B, Self>;

    fn resize(viewport: (u32, u32));
}

/// Resource for a renderer.
pub trait RenderingContext<B: GraphicBackend, S: Surface<B>> {}

/// Rendering target.
pub trait RenderingTarget<B: GraphicBackend, S: Surface<B>, Ctx: RenderingContext<B, S>> {}

/// Renderer
pub trait Renderer<
    B: GraphicBackend,
    S: Surface<B>,
    Ctx: RenderingContext<B, S>,
    T: RenderingTarget<B, S, Ctx>,
>
{
    type Error: std::error::Error;

    fn render(context: &mut Ctx, target: &mut T) -> Result<(), Self::Error>;
}
