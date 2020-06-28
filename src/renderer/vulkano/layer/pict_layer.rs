use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::ImmutableImage;
use vulkano::pipeline::{vertex::VertexSource, GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::GpuFuture;

use std::sync::Arc;

use crate::format::s25::S25Image;
use crate::renderer::vulkano::texture_loader;
use crate::utils::viewport;

pub type Texture = Arc<ImmutableImage<Format>>;

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position, uv);

#[derive(Default)]
pub struct PictLayer {
    pub texture: Option<Texture>,
    pub future: Option<Box<dyn GpuFuture>>,
    pub set: Option<Arc<dyn DescriptorSet + Sync + Send>>,
    pub vertex_buffer: Option<Arc<ImmutableBuffer<[Vertex]>>>,
}

impl PictLayer {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_cached(&self) -> bool {
        self.future.is_none()
    }

    pub fn is_loaded(&self) -> bool {
        self.texture.is_some()
    }

    pub fn clear(&mut self) {
        self.future.take();
        self.texture.take();
        self.set.take();
        self.vertex_buffer.take();
    }

    // load pict-layer information onto GPU
    pub fn load_gpu<Mv, L, Rp>(
        &mut self,
        image: S25Image,
        load_queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
        format: Format,
    ) where
        L: PipelineLayoutAbstract,
    {
        // load image to GPU
        let device = load_queue.device().clone();

        let offset = (
            image.metadata.offset_x as f64,
            image.metadata.offset_y as f64,
        );
        let size = (image.metadata.width as f64, image.metadata.height as f64);
        let (t, f) = texture_loader::load_s25_image(image, load_queue.clone(), format);

        self.texture = Some(t.clone());

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
            .add_sampled_image(t, sampler)
            .unwrap()
            .build()
            .unwrap();

        self.set = Some(Arc::new(set));

        let (vertex_buffer, vtx_future) = {
            ImmutableBuffer::from_iter(
                [
                    Vertex {
                        position: viewport::f_point_at(offset.0, offset.1),
                        uv: [0.0, 0.0],
                    },
                    Vertex {
                        position: viewport::f_point_at(offset.0, offset.1 + size.1),
                        uv: [0.0, 1.0],
                    },
                    Vertex {
                        position: viewport::f_point_at(offset.0 + size.0, offset.1),
                        uv: [1.0, 0.0],
                    },
                    Vertex {
                        position: viewport::f_point_at(offset.0 + size.0, offset.1 + size.1),
                        uv: [1.0, 1.0],
                    },
                ]
                .iter()
                .cloned(),
                BufferUsage::vertex_buffer(),
                load_queue.clone(),
            )
            .unwrap()
        };

        self.vertex_buffer = Some(vertex_buffer);
        self.future = Some(Box::new(f.join(vtx_future)));
    }

    pub fn draw<P>(
        &self,
        builder: &mut AutoCommandBufferBuilder,
        pipeline: P,
        dyn_state: &DynamicState,
        (x, y): (f64, f64),
        opacity: f32,
        (radius_x, radius_y): (i32, i32),
    ) where
        P: GraphicsPipelineAbstract
            + VertexSource<Arc<ImmutableBuffer<[Vertex]>>>
            + Send
            + Sync
            + 'static
            + Clone,
    {
        // workaround for not-loaded pict-layers
        if self.vertex_buffer.is_some() {
            builder
                .draw(
                    pipeline,
                    dyn_state,
                    self.vertex_buffer.clone().unwrap(),
                    self.set.clone().unwrap(),
                    crate::renderer::vulkano::shaders::pict_layer::vs::ty::PushConstantData {
                        offset: viewport::f_point_unscaled(x, y),
                        opacity,
                        radius_x,
                        radius_y,
                    },
                )
                .unwrap();
        }
    }

    pub fn take_future<'a>(&mut self) -> Option<Box<dyn GpuFuture>> {
        self.future.take()
    }

    pub fn has_future(&self) -> bool {
        self.future.is_some()
    }
}
