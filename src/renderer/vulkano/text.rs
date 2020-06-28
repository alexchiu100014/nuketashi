// TODO: use IBO

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{
    pool::standard::StandardCommandPoolAlloc, AutoCommandBuffer, AutoCommandBufferBuilder,
    CommandBufferExecFuture, DynamicState,
};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::pipeline::{vertex::VertexSource, GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::{GpuFuture, NowFuture};

use crate::utils::viewport;
use std::sync::Arc;

pub type Texture = Arc<ImmutableImage<Format>>;

use crate::renderer::common::text as renderer;

const FONT_HEIGHT: f32 = 44.0;

// Text layer
#[derive(Default)]
pub struct Text {
    pub wireframes: Vec<(i32, i32, i32, i32)>,
    pub offset: (i32, i32),
    pub size: (i32, i32),
    pub use_cursor: bool,
    pub cursor: f32,
    pub texture: Option<Texture>,
    pub tex_future: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
    pub vertex_buffer: Option<Arc<ImmutableBuffer<[Vertex]>>>,
    pub vtx_future:
        Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer<StandardCommandPoolAlloc>>>,
    pub indices_buffer: Option<Arc<ImmutableBuffer<[u16]>>>,
    pub idc_future:
        Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer<StandardCommandPoolAlloc>>>,
    pub set: Option<Arc<dyn DescriptorSet + Sync + Send>>,
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub text_count: f32,
}

vulkano::impl_vertex!(Vertex, position, uv, text_count);

impl Text {
    pub fn new(offset: (i32, i32), size: (i32, i32)) -> Self {
        Self {
            offset,
            size,
            ..Default::default()
        }
    }

    pub fn is_cached(&self) -> bool {
        self.tex_future.is_none()
    }

    pub fn clear(&mut self) {
        self.wireframes.clear();
        self.cursor = 0.0;
        self.texture = None;
        self.tex_future = None;
        self.vertex_buffer = None;
        self.vtx_future = None;
        self.indices_buffer = None;
        self.idc_future = None;
        self.set = None;
    }

    pub fn write<S: AsRef<str>>(&mut self, string: S, queue: Arc<Queue>) {
        let string = string.as_ref();

        if string.is_empty() {
            self.texture = None;
            self.tex_future = None;
            self.vertex_buffer = None;
            self.vtx_future = None;
            self.set = None;

            return;
        }

        let mut buf = vec![0u8; self.size.0 as usize * self.size.1 as usize * 4];

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
            Format::R8G8B8A8Srgb, // unsigned, normalized
            queue,
        )
        .expect("failed to load text into texture");

        self.texture = Some(t);
        self.tex_future = Some(f);
    }

    pub fn load_gpu<Mv, L, Rp>(
        &mut self,
        load_queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) where
        L: PipelineLayoutAbstract,
    {
        // skip if not loaded
        if self.texture.is_none() {
            return;
        }

        // load image to GPU
        let device = load_queue.device().clone();

        let sampler = Sampler::new(
            device,
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .unwrap();

        // load other information to GPU
        let layout = pipeline.layout().descriptor_set_layout(0).unwrap();
        let set = PersistentDescriptorSet::start(layout.clone())
            .add_sampled_image(self.texture.clone().unwrap(), sampler)
            .unwrap()
            .build()
            .unwrap();

        self.set = Some(Arc::new(set));

        let mut vertices = vec![];
        let mut indices = vec![];

        for (i, &(x, y, w, h)) in self.wireframes.iter().enumerate() {
            vertices.push(Vertex {
                position: viewport::point_at(x, y),
                uv: viewport::point_unscaled_boxed(x, y, self.size.0, self.size.1),
                text_count: i as f32,
            });
            vertices.push(Vertex {
                position: viewport::point_at(x + w, y),
                uv: viewport::point_unscaled_boxed(x + w, y, self.size.0, self.size.1),
                text_count: i as f32 + 0.9,
            });
            vertices.push(Vertex {
                position: viewport::point_at(x, y + h),
                uv: viewport::point_unscaled_boxed(x, y + h, self.size.0, self.size.1),
                text_count: i as f32,
            });
            vertices.push(Vertex {
                position: viewport::point_at(x + w, y + h),
                uv: viewport::point_unscaled_boxed(x + w, y + h, self.size.0, self.size.1),
                text_count: i as f32 + 0.9,
            });

            let i_4 = i as u16 * 4;

            indices.push(i_4);
            indices.push(i_4 + 1);
            indices.push(i_4 + 2);

            indices.push(i_4 + 1);
            indices.push(i_4 + 2);
            indices.push(i_4 + 3);
        }

        let (vertex_buffer, vtx_future) = {
            ImmutableBuffer::from_iter(
                vertices.into_iter(),
                BufferUsage::vertex_buffer(),
                load_queue.clone(),
            )
            .unwrap()
        };

        let (indices_buffer, idc_future) = {
            ImmutableBuffer::from_iter(
                indices.into_iter(),
                BufferUsage::index_buffer(),
                load_queue.clone(),
            )
            .unwrap()
        };

        self.vertex_buffer = Some(vertex_buffer);
        self.vtx_future = Some(vtx_future);

        self.indices_buffer = Some(indices_buffer);
        self.idc_future = Some(idc_future);
    }

    pub fn draw<P>(
        &self,
        builder: &mut AutoCommandBufferBuilder,
        pipeline: P,
        dyn_state: &DynamicState,
    ) where
        P: GraphicsPipelineAbstract
            + VertexSource<Arc<ImmutableBuffer<[Vertex]>>>
            + Send
            + Sync
            + 'static
            + Clone,
    {
        // workaround for not-loaded text
        if self.vertex_buffer.is_some() {
            builder
                .draw_indexed(
                    pipeline,
                    dyn_state,
                    self.vertex_buffer.clone().unwrap(),
                    self.indices_buffer.clone().unwrap(),
                    self.set.clone().unwrap(),
                    crate::renderer::vulkano::shaders::text::vs::ty::PushConstantData {
                        offset: viewport::point_unscaled(self.offset.0, self.offset.1),
                        text_cursor: if self.use_cursor { self.cursor } else { 1000.0 },
                    },
                )
                .unwrap();
        }
    }

    pub fn take_future<'a>(&mut self) -> Option<Box<dyn GpuFuture + 'a>> {
        Some(Box::new(
            self.tex_future
                .take()?
                .join(self.vtx_future.take()?)
                .join(self.idc_future.take()?),
        ))
    }
}
