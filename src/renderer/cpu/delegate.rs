// All CPU-driven graphics are presented using Vulkano backend. Fuck.

use crate::renderer::vulkano::{pipeline, surface::VulkanoSurface, VulkanoRenderingContext};
use std::sync::Arc;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::{Dimensions, ImageUsage, StorageImage};

use super::CpuImageBuffer;

use crate::renderer::RenderingSurface;

pub struct CpuDelegate {
    pub surface: VulkanoSurface<'static>,
    context: CpuDelegateContext,
}

impl CpuDelegate {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let surface = VulkanoSurface::new(event_loop);
        let context = CpuDelegateContext::new(
            surface.device.clone(),
            surface.format(),
            surface.graphical_queue.clone(),
        );

        Self { surface, context }
    }

    pub fn draw(&mut self, framebuffer: &CpuImageBuffer) {
        let mut target = self.surface.draw_begin(&self.context).unwrap();

        self.context.load_buffer(&framebuffer.rgba_buffer);

        let cmd = target
            .command_buffer
            .copy_buffer_to_image(self.context.buffer.clone(), self.context.texture.clone())
            .unwrap()
            .begin_render_pass(
                target.framebuffer.clone(),
                false,
                vec![[0.0, 0.0, 0.0, 1.0].into()],
            )
            .unwrap();

        let cmd = cmd
            .draw(
                self.context.pipeline.clone(),
                &mut self.surface.dynamic_state,
                self.context.vertex_buffer.clone(),
                self.context.sets.clone(),
                (),
            )
            .unwrap();

        target.command_buffer = cmd.end_render_pass().unwrap();

        self.surface.draw_end(target, &self.context);
    }
}

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

pub struct CpuDelegateContext {
    pub render_pass: Arc<dyn RenderPassAbstract + Sync + Send>,
    pub pipeline: Arc<
        GraphicsPipeline<
            SingleBufferDefinition<Vertex>,
            Box<dyn PipelineLayoutAbstract + Send + Sync>,
            Arc<dyn RenderPassAbstract + Sync + Send>,
        >,
    >,
    pub texture: StorageTexture,
    pub buffer: Arc<CpuAccessibleBuffer<[u8]>>,
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub sets: Arc<dyn DescriptorSet + Sync + Send>,
}

#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position);

pub type StorageTexture = Arc<StorageImage<Format>>;

pub fn create_storage_texture(
    viewport: (u32, u32),
    queue: Arc<Queue>,
    format: Format,
) -> StorageTexture {
    StorageImage::with_usage(
        queue.device().clone(),
        Dimensions::Dim2d {
            width: viewport.0,
            height: viewport.1,
        },
        format,
        ImageUsage {
            sampled: true,
            transfer_destination: true,
            ..ImageUsage::none()
        },
        Some(queue.family()),
    )
    .unwrap()
}

impl CpuDelegateContext {
    pub fn new(device: Arc<Device>, format: Format, queue: Arc<Queue>) -> Self {
        let render_pass = pipeline::create_render_pass(device.clone(), format)
            as Arc<dyn RenderPassAbstract + Sync + Send>;
        let pipeline = create_pipeline(device.clone(), render_pass.clone());

        let texture = create_storage_texture(
            (
                crate::constants::GAME_WINDOW_WIDTH,
                crate::constants::GAME_WINDOW_HEIGHT,
            ),
            queue,
            format,
        );

        let dim = crate::constants::GAME_WINDOW_WIDTH as usize
            * crate::constants::GAME_WINDOW_HEIGHT as usize;

        let buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage {
                transfer_source: true,
                transfer_destination: true,
                ..BufferUsage::none()
            },
            false,
            (0..dim * 4).map(|_| 0u8),
        )
        .expect("failed to create buffer");

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::none()
            },
            false,
            vec![
                Vertex {
                    position: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                },
            ]
            .into_iter(),
        )
        .expect("failed to create buffer");

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

        let layout = pipeline.layout().descriptor_set_layout(0).unwrap();
        let sets = Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_sampled_image(texture.clone(), sampler)
                .unwrap()
                .build()
                .unwrap(),
        );

        Self {
            render_pass,
            pipeline,
            texture,
            buffer,
            vertex_buffer,
            sets,
        }
    }

    pub fn load_buffer(&mut self, buf: &[u8]) {
        let mut lock = self.buffer.write().expect("failed to obtain write lock");
        lock.copy_from_slice(buf);
    }
}

impl VulkanoRenderingContext for CpuDelegateContext {
    fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Sync + Send> {
        &self.render_pass
    }
}

//

use vulkano::framebuffer::Subpass;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::GraphicsPipeline;

pub fn create_pipeline<Rp>(
    device: Arc<Device>,
    render_pass: Rp,
) -> Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<dyn PipelineLayoutAbstract + Send + Sync>,
        Rp,
    >,
>
where
    Rp: RenderPassAbstract,
{
    use crate::renderer::vulkano::shaders::simple::{fs, vs};

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device.clone())
            .unwrap(),
    )
}
