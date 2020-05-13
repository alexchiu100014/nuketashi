/* use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{
    pool::standard::StandardCommandPoolAlloc, AutoCommandBuffer, AutoCommandBufferBuilder,
    CommandBufferExecFuture, DynamicState,
};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::pipeline::{vertex::VertexSource, GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::{GpuFuture, NowFuture};

use std::sync::Arc;

pub type Texture = Arc<ImmutableImage<Format>>;

pub mod renderer;

const FONT_HEIGHT: f32 = 24.0;

#[derive(Default)]
pub struct Text {
    pub wireframes: Vec<(i32, i32, i32, i32)>,
    pub offset: (i32, i32),
    pub size: (i32, i32),
    pub cursor: f32,
    pub texture: Option<Texture>,
    pub tex_future: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
}

impl Text {
    pub fn write<S: AsRef<str>>(&mut self, string: S, queue: Arc<Queue>) {
        let string = string.as_ref();

        if string.is_empty() {
            self.texture = None;
            self.tex_future = None;

            return;
        }

        let mut buf = vec![0u8; self.size.0 as usize * self.size.1 as usize];

        self.wireframes = renderer::write_text_in_box(
            renderer::create_font(),
            FONT_HEIGHT,
            string,
            (self.size.0 as usize, self.size.1 as usize),
            &mut buf,
        );

        // load to texture
        let (t, f) = ImmutableImage::from_iter(
            buf.into_iter(),
            Dimensions::Dim2d {
                width: self.size.0 as u32,
                height: self.size.1 as u32,
            },
            Format::R8Srgb, // unsigned, normalized
            queue,
        )
        .expect("failed to load text into texture");

        self.texture = Some(t);
        self.tex_future = Some(f);
    }
}
*/
