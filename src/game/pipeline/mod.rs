use std::sync::Arc;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain::Swapchain;

pub fn create_render_pass<W>(
    device: Arc<Device>,
    swapchain: &Swapchain<W>,
) -> Arc<impl RenderPassAbstract> {
    Arc::new(
        vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
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

pub fn create_pict_layer_pipeline<Rp, V, W>(
    device: Arc<Device>,
    render_pass: Rp,
) -> Arc<impl GraphicsPipelineAbstract>
where
    Rp: RenderPassAbstract,
{
    use crate::game::layer::Vertex;

    let vs = crate::game::shaders::pict_layer::vs::Shader::load(device.clone()).unwrap();
    let fs = crate::game::shaders::pict_layer::fs::Shader::load(device.clone()).unwrap();

    Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device.clone())
            .unwrap(),
    )
}
