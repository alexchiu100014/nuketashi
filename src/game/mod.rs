pub mod scene;

use crate::renderer::vulkano::layer::LayerRenderer;
use crate::renderer::Renderer;
use crate::script::mil::command::{Command as MilCommand, RendererCommand, RuntimeCommand};

use std::sync::Arc;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;

use crate::renderer::vulkano::text::Text;

pub struct Game {
    layers: Vec<LayerRenderer>,
    // face_layer: LayerRenderer,
    text_layer: Text,
    text_update: bool,
    commands: Vec<MilCommand>,
    queue: Option<Arc<Queue>>,
    waiting: bool,
}

use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

impl Game {
    fn platform_specific_setup() {
        #[cfg(target_os = "macos")]
        unsafe {
            use crate::platform::macos;
            macos::create_menu_bar();
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self::platform_specific_setup();

        Game {
            layers: vec![],
            commands: vec![],
            text_layer: Text::new((380, 640), (900, 300)),
            text_update: false,
            waiting: false,
            queue: None,
        }
    }

    pub fn load_script(&mut self) {
        use encoding_rs::SHIFT_JIS;

        use crate::script::rio::parser::Parser;
        use crate::script::rio::transpiler::Transpiler;

        let script = std::fs::read("./testcase/02_NK_23H.TXT").unwrap();
        let (script, _, _) = SHIFT_JIS.decode(&script);

        let mut parser = Parser::from_raw_bytes(script.as_bytes());
        let script = parser.parse().unwrap();

        let tr = Transpiler::new(script);
        let script = tr.transpile();

        use crate::script::mil::pass::prefetch::PrefetchPass;
        use crate::script::mil::pass::Pass;

        let prefetch = PrefetchPass::new();
        let mut script = prefetch.process(script);

        script.reverse();

        self.commands = script;
    }

    pub fn exec_script(&mut self) {
        if self.waiting {
            return;
        }

        while let Some(cmd) = self.commands.pop() {
            match cmd {
                MilCommand::RuntimeCommand(RuntimeCommand::WaitUntilUserEvent) => {
                    self.waiting = true;
                    return;
                }
                MilCommand::RendererCommand(r) => {
                    self.visit_renderer_command(r);
                }
                MilCommand::LayerCommand { layer_no, command } => {
                    self.layers[layer_no as usize].send(command);
                }
                _ => {
                    log::debug!("skipped command: {:?}", cmd);
                }
            }
        }
    }

    fn visit_renderer_command(&mut self, command: RendererCommand) {
        match command {
            RendererCommand::Dialogue(name, dialogue) => {
                self.text_layer.write(
                    format!("{}\n{}", name.unwrap_or_default(), dialogue),
                    self.queue.clone().unwrap(),
                );

                self.text_update = true;
            }
            _ => {
                log::debug!("skipped renderer command: {:?}", command);
            }
        }
    }

    pub fn execute(mut self) {
        use crate::config;
        use crate::renderer::vulkano::surface::VulkanoSurface;
        use crate::renderer::{EventDelegate, RenderingSurface};

        self.load_script();

        let event_loop = EventLoop::new();
        let mut buf = VulkanoSurface::new(&event_loop);

        buf.set_title(config::get_game_title());

        // create layer renderer
        self.layers
            .resize_with(30, || LayerRenderer::new(buf.format()));

        use crate::renderer::vulkano::layer::LayerRenderingContext;
        use crate::renderer::vulkano::pipeline;

        let render_pass = pipeline::create_render_pass(buf.device.clone(), buf.format())
            as Arc<dyn RenderPassAbstract + Sync + Send>;
        let pipeline =
            pipeline::create_pict_layer_pipeline(buf.device.clone(), render_pass.clone());
        let pipeline_text =
            pipeline::create_text_layer_pipeline(buf.device.clone(), render_pass.clone());

        /* let (tex, fb) = crate::renderer::vulkano::layer::layer_texture::create_layer_texture(
            (1600, 900),
            buf.graphical_queue.clone(),
            render_pass.clone(),
            buf.format(),
        ); */

        self.queue = Some(buf.graphical_queue.clone());

        let ctx = LayerRenderingContext {
            render_pass,
            pipeline: pipeline.clone(),
        };

        // for benchmark

        event_loop.run(move |event, _evt_loop, control_flow| {
            buf.handle_event(&event, control_flow);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    self.waiting = false;
                }
                Event::RedrawRequested(_) => {
                    use vulkano::sync::GpuFuture;

                    self.exec_script();

                    let mut target = buf.draw_begin(&ctx).unwrap();

                    if self.text_update {
                        self.text_layer
                            .load_gpu(buf.graphical_queue.clone(), pipeline_text.clone());
                        self.text_update = false;
                    }

                    if let Some(future) = self.text_layer.take_future() {
                        target.future = Box::new(target.future.join(future));
                    }

                    target
                        .command_buffer
                        .begin_render_pass(
                            target.framebuffer.clone(),
                            false,
                            vec![[0.0, 0.0, 0.0, 1.0].into()],
                        )
                        .unwrap();

                    for l in &mut self.layers {
                        l.update(buf.graphical_queue.clone(), pipeline.clone());
                        target.future =
                            Box::new(target.future.join(l.take_future(buf.device.clone())));
                        l.render(&mut target, &ctx);
                    }

                    self.text_layer.draw(
                        &mut target.command_buffer,
                        pipeline_text.clone(),
                        &mut target.dynamic_state,
                    );

                    target.command_buffer.end_render_pass().unwrap();

                    buf.draw_end(target, &ctx);

                    buf.surface.window().request_redraw();
                }
                _ => {}
            }
        })
    }
}
