use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, RenderPassAbstract};
use vulkano::image::{Dimensions, ImageUsage, StorageImage};

use std::sync::Arc;

pub type LayerTexture = Arc<StorageImage<Format>>;

pub fn create_layer_texture<Rp>(
    viewport: (u32, u32),
    queue: Arc<Queue>,
    render_pass: Rp,
    format: Format,
) -> (LayerTexture, Arc<Framebuffer<Rp, ((), LayerTexture)>>)
where
    Rp: RenderPassAbstract,
{
    let image = StorageImage::with_usage(
        queue.device().clone(),
        Dimensions::Dim2d {
            width: viewport.0,
            height: viewport.1,
        },
        format,
        ImageUsage {
            sampled: true,
            color_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        },
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
