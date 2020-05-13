use crate::format::s25::{Result, S25Archive, S25Image};

use vulkano::format::Format;
use vulkano::image::Dimensions;
use vulkano::image::ImmutableImage;

use vulkano::command_buffer::{AutoCommandBuffer, CommandBufferExecFuture};
use vulkano::device::Queue;
use vulkano::sync::NowFuture;

use std::sync::Arc;

pub fn load_s25_entry(
    archive: &mut S25Archive,
    entry: usize,
    queue: Arc<Queue>,
) -> Result<(
    Arc<ImmutableImage<Format>>,
    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
)> {
    Ok(load_s25_image(archive.load_image(entry)?, queue))
}

pub fn load_s25_image(
    image: S25Image,
    queue: Arc<Queue>,
) -> (
    Arc<ImmutableImage<Format>>,
    CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
) {
    let (w, h) = (image.metadata.width as u32, image.metadata.height as u32);

    ImmutableImage::from_iter(
        image.rgba_buffer.into_iter(),
        Dimensions::Dim2d {
            width: w,
            height: h,
        },
        Format::R8G8B8A8Srgb, // unsigned, normalized
        queue,
    )
    .expect("failed to load image")
}
