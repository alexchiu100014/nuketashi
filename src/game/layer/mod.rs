use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{
    pool::standard::StandardCommandPoolAlloc, AutoCommandBuffer, AutoCommandBufferBuilder,
    CommandBufferExecFuture, DynamicState,
};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::ImmutableImage;
use vulkano::pipeline::{vertex::VertexSource, GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::{GpuFuture, NowFuture};

use std::sync::Arc;

use crate::format::s25::{S25Archive, S25Image};
use crate::game::texture_loader;
pub type Texture = Arc<ImmutableImage<Format>>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum OverlayMode {
    Disabled,
    Normal,
    Reverse,
}

impl Default for OverlayMode {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position, uv);

#[derive(Default)]
pub struct PictLayer {
    pub entry_no: i32,
    pub texture: Option<Texture>,
    pub future: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
    pub offset: (i32, i32),
    pub size: (i32, i32),
    pub set: Option<Arc<dyn DescriptorSet + Sync + Send>>,
    pub vertex_buffer: Option<Arc<ImmutableBuffer<[Vertex]>>>,
    pub vtx_future:
        Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer<StandardCommandPoolAlloc>>>,
}

fn point_at(x: i32, y: i32) -> [f32; 2] {
    const W_COEF: f64 = 2.0 / (crate::constants::GAME_WINDOW_WIDTH as f64);
    const H_COEF: f64 = 2.0 / (crate::constants::GAME_WINDOW_HEIGHT as f64);

    [
        (x as f64 * W_COEF - 1.0) as f32,
        (y as f64 * H_COEF - 1.0) as f32,
    ]
}

impl PictLayer {
    pub fn empty() -> Self {
        Self {
            entry_no: -1,
            ..Self::default()
        }
    }

    // load pict-layer information onto GPU
    pub fn load_gpu<Mv, L, Rp>(
        &mut self,
        image: S25Image,
        load_queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) where
        L: PipelineLayoutAbstract,
    {
        // load image to GPU
        let device = load_queue.device().clone();

        let offset = (image.metadata.offset_x, image.metadata.offset_y);
        let size = (image.metadata.width, image.metadata.height);
        let (t, f) = texture_loader::load_s25_image(image, load_queue.clone());

        self.texture = Some(t.clone());
        self.future = Some(f);
        self.offset = offset;
        self.size = size;

        let sampler = Sampler::new(
            device,
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
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
                        position: point_at(self.offset.0, self.offset.1),
                        uv: [0.0, 0.0],
                    },
                    Vertex {
                        position: point_at(self.offset.0, self.offset.1 + self.size.1),
                        uv: [0.0, 1.0],
                    },
                    Vertex {
                        position: point_at(self.offset.0 + self.size.0, self.offset.1),
                        uv: [1.0, 0.0],
                    },
                    Vertex {
                        position: point_at(
                            self.offset.0 + self.size.0,
                            self.offset.1 + self.size.1,
                        ),
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
        self.vtx_future = Some(vtx_future);
    }

    pub fn draw<P>(
        &self,
        builder: AutoCommandBufferBuilder,
        pipeline: P,
        dyn_state: &DynamicState,
    ) -> AutoCommandBufferBuilder
    where
        P: GraphicsPipelineAbstract
            + VertexSource<Arc<ImmutableBuffer<[Vertex]>>>
            + Send
            + Sync
            + 'static
            + Clone,
    {
        builder
            .draw(
                pipeline,
                dyn_state,
                self.vertex_buffer.clone().unwrap(),
                self.set.clone().unwrap(),
                (),
            )
            .unwrap()
    }

    pub fn join_future(&mut self, device: Arc<Device>, future: impl GpuFuture) -> impl GpuFuture {
        let future = self.join_vtx_future(device.clone(), future);

        if let Some(f) = self.future.take() {
            future.join(Box::new(f) as Box<dyn GpuFuture>)
        } else {
            future.join(Box::new(vulkano::sync::now(device)) as Box<dyn GpuFuture>)
        }
    }

    fn join_vtx_future(&mut self, device: Arc<Device>, future: impl GpuFuture) -> impl GpuFuture {
        if let Some(f) = self.vtx_future.take() {
            future.join(Box::new(f) as Box<dyn GpuFuture>)
        } else {
            future.join(Box::new(vulkano::sync::now(device)) as Box<dyn GpuFuture>)
        }
    }
}

#[derive(Default)]
pub struct Layer {
    // S25 archive that corresponds to the layer
    pub s25_archive: Option<S25Archive>,
    // parameters for pict layers
    pub pict_layers: Vec<PictLayer>,
    pub overlay: Option<Texture>,
    pub overlay_future: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
    // value sent to the shader to render
    pub overlay_mode: OverlayMode,
    pub overlay_rate: f32, // [0, 1]
    pub position: (i32, i32),
    pub opacity: f32, // [0, 1]
}

impl Layer {
    pub fn load_s25(&mut self, s25: S25Archive) {
        // clear layer
        self.clear_layers();

        // replace s25 file
        self.s25_archive = Some(s25);
    }

    pub fn load_pict_layers<Mv, L, Rp>(
        &mut self,
        pict_layers: &[i32],
        load_queue: Arc<Queue>,
        pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) where
        L: PipelineLayoutAbstract,
    {
        // match the length of pict layers
        self.pict_layers
            .resize_with(pict_layers.len(), PictLayer::empty);

        if let Some(arc) = &mut self.s25_archive {
            for (i, &entry) in pict_layers.iter().enumerate() {
                let pict_layer = &mut self.pict_layers[i];
                // don't reload if the image is the same
                if pict_layer.entry_no == entry {
                    continue;
                }

                pict_layer.entry_no = entry;

                // clear the pict-layer is -1 is given
                if entry == -1 {
                    pict_layer.texture = None;
                    pict_layer.future = None;
                    continue;
                }

                let img = arc
                    .load_image(i * 100 + entry as usize)
                    .expect("failed to load the image entry");

                // load pict-layer information to GPU
                pict_layer.load_gpu(img, load_queue.clone(), pipeline.clone());
            }
        }
    }

    pub fn load_overlay(&mut self, overlay: S25Image, load_queue: Arc<Queue>) {
        let (t, f) = texture_loader::load_s25_image(overlay, load_queue);
        self.overlay = Some(t);
        self.overlay_future = Some(f);
    }

    pub fn clear_layers(&mut self) {
        self.pict_layers.clear();

        self.overlay.take();
        self.overlay_future.take();

        // make opacity and overlay_rate zero
        self.opacity = 0.0;
        self.overlay_rate = 0.0;

        // disable overlay mode
        self.overlay_mode = OverlayMode::Disabled;
    }

    pub fn draw<P>(
        &self,
        builder: AutoCommandBufferBuilder,
        pipeline: P,
        dyn_state: &DynamicState,
    ) -> AutoCommandBufferBuilder
    where
        P: GraphicsPipelineAbstract
            + VertexSource<Arc<ImmutableBuffer<[Vertex]>>>
            + Send
            + Sync
            + 'static
            + Clone,
    {
        let mut builder = builder;

        // let all the pict-layers draw
        for layer in &self.pict_layers {
            builder = layer.draw(builder, pipeline.clone(), dyn_state);
        }

        // TODO: apply overlay

        builder
    }

    pub fn join_future<'a>(
        &mut self,
        device: Arc<Device>,
        future: impl GpuFuture + 'a,
    ) -> impl GpuFuture + 'a {
        // TODO: ugh, so many boxing...

        // let all the pict-layers load
        let mut future: Box<dyn GpuFuture + 'a> = Box::new(future);
        for layer in &mut self.pict_layers {
            future = Box::new(layer.join_future(device.clone(), future));
        }

        if let Some(f) = self.overlay_future.take() {
            future.join(Box::new(f) as Box<dyn GpuFuture>)
        } else {
            future.join(Box::new(vulkano::sync::now(device)) as Box<dyn GpuFuture>)
        }
    }
}
