use std::sync::Arc;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{vertex::SingleBufferDefinition, GraphicsPipeline};

pub fn create_render_pass(device: Arc<Device>, format: Format) -> Arc<impl RenderPassAbstract> {
    Arc::new(
        vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    )
}

use crate::game::layer;

pub fn create_pict_layer_pipeline<Rp>(
    device: Arc<Device>,
    render_pass: Rp,
) -> Arc<
    GraphicsPipeline<
        SingleBufferDefinition<layer::Vertex>,
        Box<dyn PipelineLayoutAbstract + Send + Sync>,
        Rp,
    >,
>
where
    Rp: RenderPassAbstract,
{
    use crate::game::shaders::pict_layer::{fs, vs};

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<layer::Vertex>()
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

use crate::game::text;

pub fn create_text_layer_pipeline<Rp>(
    device: Arc<Device>,
    render_pass: Rp,
) -> Arc<
    GraphicsPipeline<
        SingleBufferDefinition<text::Vertex>,
        Box<dyn PipelineLayoutAbstract + Send + Sync>,
        Rp,
    >,
>
where
    Rp: RenderPassAbstract,
{
    use crate::game::shaders::text::{fs, vs};

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<text::Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device.clone())
            .unwrap(),
    )
}
