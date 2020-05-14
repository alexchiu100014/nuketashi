use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, RenderPassAbstract};
use vulkano::image::{Dimensions, StorageImage};

use std::sync::Arc;

pub fn create_layer_texture<Rp>(
    viewport: (u32, u32),
    queue: Arc<Queue>,
    render_pass: Rp,
    format: Format,
) -> (
    Arc<StorageImage<Format>>,
    Arc<Framebuffer<Rp, ((), Arc<StorageImage<Format>>)>>,
)
where
    Rp: RenderPassAbstract,
{
    let image = StorageImage::new(
        queue.device().clone(),
        Dimensions::Dim2d {
            width: viewport.0,
            height: viewport.1,
        },
        format,
        Some(queue.family()),
    )
    .unwrap();

    let framebuffer = Arc::new(
        Framebuffer::start(render_pass)
            .add(image.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    (image, framebuffer)
}
